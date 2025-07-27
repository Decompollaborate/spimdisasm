/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;

use rabbitizer::{Instruction, InstructionFlags};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, RomVramRange, Size, Vram, VramOffset};
use crate::analysis::{InstrOpTailCall, InstructionOperation, ReferenceWrapper, RegisterTracker};
use crate::collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet};
use crate::config::{Compiler, Endian, GlobalConfig};
use crate::context::Context;
use crate::metadata::{ParentSectionMetadata, SegmentMetadata, SymbolType};
use crate::parent_segment_info::ParentSegmentInfo;
use crate::relocation::RelocationInfo;
use crate::section_type::SectionType;
use crate::sections::processed::ExecutableSectionProcessed;
use crate::sections::{
    BadBytesSizeError, BadUserSymbolSizeError, EmptySectionError, RomSectionPreprocessed,
    SectionPreprocessed, UnalignedRomError, UnalignedVramError,
};
use crate::str_decoding::Encoding;
use crate::symbols::before_proc::data_sym::DataSymProperties;
use crate::symbols::before_proc::{DataSym, EitherFuncDataSym};
use crate::symbols::SymbolPreprocessed;
use crate::symbols::{
    before_proc::{function_sym::FunctionSymProperties, FunctionSym},
    Symbol,
};

use crate::sections::{
    section_post_process_error::SectionPostProcessError,
    trait_section::RomSection,
    {Section, SectionCreationError},
};

const SECTION_TYPE: SectionType = SectionType::Text;

const BYTES_PER_INSTR: u32 = 4;

#[derive(Debug, Clone)]
#[must_use]
pub struct ExecutableSection {
    name: Arc<str>,

    ranges: RomVramRange,

    parent_segment_info: ParentSegmentInfo,

    // in_section_offset: u32,
    // section_type: SectionType,

    //
    symbols: Vec<EitherFuncDataSym>,

    symbol_vrams: UnorderedSet<Vram>,
}

impl ExecutableSection {
    pub(crate) fn new(
        context: &mut Context,
        settings: &ExecutableSectionSettings,
        name: Arc<str>,
        raw_bytes: Vec<u8>,
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<Self, SectionCreationError> {
        if raw_bytes.is_empty() {
            return Err(EmptySectionError::new(name, vram).into());
        }
        if raw_bytes.len() % BYTES_PER_INSTR as usize != 0 {
            return Err(
                BadBytesSizeError::new(name, raw_bytes.len(), BYTES_PER_INSTR as usize).into(),
            );
        }
        if vram.inner() % BYTES_PER_INSTR != 0 {
            return Err(UnalignedVramError::new(name, vram, BYTES_PER_INSTR as usize).into());
        }
        if rom.inner() % BYTES_PER_INSTR != 0 {
            return Err(UnalignedRomError::new(name, rom, BYTES_PER_INSTR as usize).into());
        }

        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let instrs =
            instrs_from_bytes(settings, context.global_config().endian(), &raw_bytes, vram);
        debug_assert!(!instrs.is_empty(), "{name}, {vram:?}, {rom:?}");

        let owned_segment = context.find_owned_segment(&parent_segment_info)?;
        let boundaries = find_functions(
            context.global_config(),
            settings,
            owned_segment,
            ranges,
            &instrs,
        )?;

        debug_assert!(!boundaries.is_empty(), "{name}, {vram:?}, {rom:?}");

        let mut symbols = Vec::new();
        let mut symbol_vrams = UnorderedSet::new();

        let mut bytes_iter = raw_bytes.into_iter();
        let mut instrs_iter = instrs.into_iter();

        for boundary in boundaries {
            let local_offset = boundary.start as u32 * BYTES_PER_INSTR;
            let s = Size::new(local_offset);
            let current_vram = vram + s;
            let current_rom = rom + s;

            symbol_vrams.insert(vram);

            let parent_metadata =
                ParentSectionMetadata::new(name.clone(), vram, parent_segment_info.clone());

            let word_count = boundary.end - boundary.start;

            let sym = if let Some(data_type) = boundary.data_type {
                let sym_bytes: Arc<[u8]> = bytes_iter
                    .by_ref()
                    .take(word_count * BYTES_PER_INSTR as usize)
                    .collect();
                debug_assert_eq!(sym_bytes.len(), word_count * BYTES_PER_INSTR as usize);
                // Advance the other buffer too
                let _count = instrs_iter.by_ref().take(word_count).count();
                debug_assert_eq!(_count, word_count);

                let properties = DataSymProperties::new(
                    parent_metadata,
                    settings.compiler,
                    boundary.auto_pad_by,
                    Some(data_type),
                    Encoding::default(),
                );
                let data = DataSym::new(
                    context,
                    sym_bytes,
                    current_rom,
                    current_vram,
                    parent_segment_info.clone(),
                    SECTION_TYPE,
                    properties,
                )?;

                EitherFuncDataSym::Data(data)
            } else {
                let sym_instrs: Arc<[Instruction]> =
                    instrs_iter.by_ref().take(word_count).collect();
                debug_assert_eq!(sym_instrs.len(), word_count);
                // Advance the other buffer too
                let _count = bytes_iter
                    .by_ref()
                    .take(word_count * BYTES_PER_INSTR as usize)
                    .count();
                debug_assert_eq!(
                    _count,
                    word_count * BYTES_PER_INSTR as usize,
                    "{} {}",
                    boundary.start,
                    boundary.end
                );

                let properties = FunctionSymProperties {
                    parent_metadata,
                    compiler: settings.compiler,
                    auto_pad_by: boundary.auto_pad_by,
                };
                let func = FunctionSym::new(
                    context,
                    sym_instrs,
                    current_rom,
                    current_vram,
                    parent_segment_info.clone(),
                    properties,
                )?;

                EitherFuncDataSym::Func(func)
            };

            symbols.push(sym);
        }

        Ok(Self {
            name,
            ranges,
            parent_segment_info,
            symbols,
            symbol_vrams,
        })
    }
}

impl ExecutableSection {
    pub fn symbols(&self) -> &[EitherFuncDataSym] {
        &self.symbols
    }
}

impl ExecutableSection {
    pub fn post_process(
        self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<ExecutableSectionProcessed, SectionPostProcessError> {
        let ExecutableSection {
            name,
            ranges,
            parent_segment_info,
            symbols,
            symbol_vrams,
        } = self;

        ExecutableSectionProcessed::new(
            context,
            name,
            ranges,
            parent_segment_info,
            symbols,
            symbol_vrams,
            user_relocs,
        )
    }
}

impl Section for ExecutableSection {
    fn name(&self) -> Arc<str> {
        self.name.clone()
    }

    fn vram_range(&self) -> &AddressRange<Vram> {
        self.ranges.vram()
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    fn section_type(&self) -> SectionType {
        SECTION_TYPE
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.symbols
    }

    fn symbols_vrams(&self) -> &UnorderedSet<Vram> {
        &self.symbol_vrams
    }
}
impl RomSection for ExecutableSection {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SectionPreprocessed for ExecutableSection {
    fn symbol_list(&self) -> &[impl SymbolPreprocessed] {
        &self.symbols
    }
}
impl RomSectionPreprocessed for ExecutableSection {}

impl hash::Hash for ExecutableSection {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for ExecutableSection {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for ExecutableSection {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // Compare segment info first, so symbols get sorted by segment
        match self
            .parent_segment_info
            .partial_cmp(&other.parent_segment_info)
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.ranges.partial_cmp(&other.ranges)
    }
}

fn instrs_from_bytes(
    settings: &ExecutableSectionSettings,
    endian: Endian,
    raw_bytes: &[u8],
    mut vram: Vram,
) -> Vec<Instruction> {
    let mut instrs = Vec::new();

    for b in raw_bytes.chunks_exact(BYTES_PER_INSTR as usize) {
        let word = endian.word_from_bytes(b);

        instrs.push(Instruction::new(word, vram, settings.instruction_flags));
        vram += VramOffset::new(BYTES_PER_INSTR as i32);
    }

    instrs
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct FarthestBranch {
    farthest: VramOffset,
    /// Mainly just for debugging
    last_setter_rom: Rom,
}

impl FarthestBranch {
    fn new(current_rom: Rom) -> Self {
        Self {
            farthest: VramOffset::new(0),
            last_setter_rom: current_rom,
        }
    }

    fn farthest(&self) -> VramOffset {
        self.farthest
    }

    fn update(&mut self, current_rom: Rom, new_farthest: VramOffset) {
        if new_farthest > self.farthest {
            self.farthest = new_farthest;
            self.last_setter_rom = current_rom;
        }
    }

    fn decrease(&mut self) {
        self.farthest = VramOffset::new(self.farthest.inner() - BYTES_PER_INSTR as i32);
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
struct GatheredFunctionBoundary {
    start: usize,
    end: usize,
    auto_pad_by: Option<Vram>,
    data_type: Option<SymbolType>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
struct TempFunctionBoundary {
    start: usize,
    auto_pad_by: Option<Vram>,
    data_type: Option<SymbolType>,
}

impl TempFunctionBoundary {
    const fn new_func(start: usize, auto_pad_by: Option<Vram>) -> Self {
        Self {
            start,
            auto_pad_by,
            data_type: None,
        }
    }

    const fn new_data(start: usize, auto_pad_by: Option<Vram>, data_type: SymbolType) -> Self {
        Self {
            start,
            auto_pad_by,
            data_type: Some(data_type),
        }
    }

    const fn finish(self, end: usize) -> GatheredFunctionBoundary {
        let Self {
            start,
            auto_pad_by,
            data_type,
        } = self;

        GatheredFunctionBoundary {
            start,
            end,
            auto_pad_by,
            data_type,
        }
    }
}

fn find_functions(
    global_config: &GlobalConfig,
    settings: &ExecutableSectionSettings,
    owned_segment: &SegmentMetadata,
    section_ranges: RomVramRange,
    instrs: &[Instruction],
) -> Result<Vec<GatheredFunctionBoundary>, SectionCreationError> {
    let boundaries = find_functions_impl(
        global_config,
        settings,
        owned_segment,
        section_ranges,
        instrs,
    )?;

    let rom = section_ranges.rom().start();
    let vram = section_ranges.vram().start();

    let mut ends: Vec<usize> = boundaries.iter().skip(1).map(|x| x.start).collect();
    ends.push(instrs.len());
    debug_assert!(boundaries.len() == ends.len());

    Ok(boundaries
        .into_iter()
        .zip(ends)
        .map(|(boundary, end)| {
            debug_assert!(
                boundary.start < end,
                "{:?} {} {} {}",
                rom,
                vram,
                boundary.start,
                end
            );

            boundary.finish(end)
        })
        .collect())
}

fn find_functions_impl(
    global_config: &GlobalConfig,
    settings: &ExecutableSectionSettings,
    owned_segment: &SegmentMetadata,
    section_ranges: RomVramRange,
    instrs: &[Instruction],
) -> Result<Vec<TempFunctionBoundary>, SectionCreationError> {
    let mut starts_data = vec![TempFunctionBoundary::new_func(0, None)];

    let mut function_ended = None;
    let mut farthest_branch = FarthestBranch::new(section_ranges.rom().start());

    let mut index: usize = 0;

    let (mut current_function_start, mut current_function_ref) = match find_current_start(
        owned_segment,
        section_ranges.vram().start(),
        instrs,
        &mut index,
        None,
        &mut starts_data,
        true,
    )? {
        NextStartResult::FunctionStart(current_function_start, current_function_ref) => {
            (current_function_start, current_function_ref)
        }
        NextStartResult::SectionEnded => return Ok(starts_data),
    };

    let mut regs_tracker = RegisterTracker::new(
        settings.instruction_flags().abi(),
        Some(section_ranges.vram().start() + Size::new(index as u32 * BYTES_PER_INSTR)),
        global_config.gp_config().copied(),
        global_config.endian(),
    );
    let mut prev_instr = None;

    let global_offset_table = owned_segment.global_offset_table();

    while index < instrs.len() {
        if let Some(function_ended) = function_ended {
            match function_ended {
                FunctionEndedState::WithDelaySlot => {
                    index += 1;
                }
                FunctionEndedState::ByException => {}
            }

            (current_function_start, current_function_ref) = match find_current_start(
                owned_segment,
                section_ranges.vram().start(),
                instrs,
                &mut index,
                current_function_ref.as_ref(),
                &mut starts_data,
                false,
            )? {
                NextStartResult::FunctionStart(current_function_start, current_function_ref) => {
                    (current_function_start, current_function_ref)
                }
                NextStartResult::SectionEnded => return Ok(starts_data),
            };

            prev_instr = None;
            regs_tracker.soft_reset(
                instrs[index].abi(),
                Some(section_ranges.vram().start() + Size::new(index as u32 * BYTES_PER_INSTR)),
            );
            farthest_branch = FarthestBranch::new(
                section_ranges.rom().start() + Size::new(index as u32 * BYTES_PER_INSTR),
            );
        }

        let instr = &instrs[index];

        let current_rom = section_ranges.rom().start() + Size::new(index as u32 * BYTES_PER_INSTR);
        let instr_processed_result =
            regs_tracker.process_instruction(instr, current_rom, global_offset_table);

        if instr.opcode().is_branch()
            || instr.is_unconditional_branch()
            || instr.is_jumptable_jump()
        {
            find_functions_branch_checker(
                owned_segment,
                &instr_processed_result,
                section_ranges,
                index * BYTES_PER_INSTR as usize,
                instr,
                &mut farthest_branch,
            );
        }

        function_ended = find_functions_check_function_ended(
            owned_segment,
            &instr_processed_result,
            settings,
            index,
            instrs,
            section_ranges.rom().start() + Size::new(index as u32 * BYTES_PER_INSTR),
            section_ranges.vram().start() + Size::new(index as u32 * BYTES_PER_INSTR),
            current_function_ref,
            &farthest_branch,
            current_function_start,
        );

        regs_tracker.clear_afterwards(prev_instr, None);

        if instr.is_valid() {
            prev_instr = Some(instr);
        } else {
            prev_instr = None;
        }

        index += 1;
        farthest_branch.decrease();
    }

    Ok(starts_data)
}

fn find_functions_branch_checker(
    owned_segment: &SegmentMetadata,
    instr_processed_result: &InstructionOperation,
    section_ranges: RomVramRange,
    local_offset: usize,
    instr: &Instruction,
    farthest_branch: &mut FarthestBranch,
) {
    if instr.opcode().is_jump_with_address() {
        // If this instruction is a jump and it is jumping to a function then
        // don't treat it as a branch, it is probably actually being used as
        // a jump

        // TODO
        if let Some(target_vram) = instr.get_instr_index_as_vram() {
            if let Some(aux_ref) =
                owned_segment.find_reference(target_vram, FindSettings::new(false))
            {
                if aux_ref.is_trustable_function() {
                    return;
                }
            }
        }
    }

    match instr_processed_result {
        InstructionOperation::Branch { target_vram } => {
            let branch_offset = *target_vram - instr.vram();
            let current_rom = section_ranges.rom().start() + Size::new(local_offset as u32);

            farthest_branch.update(current_rom, branch_offset);
        }
        InstructionOperation::JumptableJump { jumptable_vram, .. } => {
            // Check jumptables
            if let Some(jumptable_ref) =
                owned_segment.find_reference(*jumptable_vram, FindSettings::new(false))
            {
                for jtbl_label_vram in jumptable_ref.table_labels() {
                    let branch_offset = *jtbl_label_vram - instr.vram();
                    let current_rom = section_ranges.rom().start() + Size::new(local_offset as u32);

                    farthest_branch.update(current_rom, branch_offset);
                }
            }
        }

        InstructionOperation::Link { .. }
        | InstructionOperation::TailCall { .. }
        | InstructionOperation::ReturnJump
        | InstructionOperation::Hi { .. }
        | InstructionOperation::PairedAddress { .. }
        | InstructionOperation::GpSet { .. }
        | InstructionOperation::DereferencedRawAddress { .. }
        | InstructionOperation::DanglingLo { .. }
        | InstructionOperation::Constant { .. }
        | InstructionOperation::UnpairedConstant { .. }
        | InstructionOperation::RegisterOperation { .. }
        | InstructionOperation::UnhandledOpcode { .. }
        | InstructionOperation::InvalidInstr { .. } => {}
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum FunctionEndedState {
    WithDelaySlot,
    ByException,
}

#[expect(clippy::too_many_arguments)]
fn find_functions_check_function_ended(
    owned_segment: &SegmentMetadata,
    instr_processed_result: &InstructionOperation,
    settings: &ExecutableSectionSettings,
    index: usize,
    instrs: &[Instruction],
    current_rom: Rom,
    current_vram: Vram,
    current_function_ref: Option<ReferenceWrapper>,
    farthest_branch: &FarthestBranch,
    current_function_start: usize,
) -> Option<FunctionEndedState> {
    let instr = &instrs[index];
    let opcode = instr.opcode();

    if let Some(reference) = current_function_ref {
        if let Some(user_declared_size) = reference.user_declared_size() {
            // If the function has a size set by the user then only use that and ignore the other ways of determining function-ends
            return if (index + 2) * BYTES_PER_INSTR as usize
                == current_function_start + user_declared_size.inner() as usize
            {
                Some(FunctionEndedState::WithDelaySlot)
            } else {
                None
            };
        }
    }

    if let Some(reference) = owned_segment.find_reference(
        current_vram + VramOffset::new(2 * BYTES_PER_INSTR as i32),
        FindSettings::new(false),
    ) {
        // If there's another function after this then the current function has ended
        if reference.is_trustable_function() {
            if let Some(sym_rom) = reference.rom() {
                if current_rom + Size::new(2 * BYTES_PER_INSTR) == sym_rom {
                    return Some(FunctionEndedState::WithDelaySlot);
                }
            } else {
                return Some(FunctionEndedState::WithDelaySlot);
            }
        }
    }

    if farthest_branch.farthest().is_positive() {
        // We still have a branch that branched even farther than where we currently are, so we
        // must still be inside the same function.
        return None;
    }

    if opcode.causes_unconditional_exception() && !opcode.causes_returnable_exception() {
        return Some(FunctionEndedState::ByException);
    }

    if settings.negative_branch_as_end() && instr.is_unconditional_branch() {
        if let Some(target_vram) = instr.get_branch_vram_generic() {
            if target_vram < current_vram {
                return Some(FunctionEndedState::WithDelaySlot);
            }
        }
    }

    // if !opcode.is_jump() {
    //     return None;
    // }

    match instr_processed_result {
        InstructionOperation::Link { .. } => {
            debug_assert!(opcode.does_link(), "{current_rom:?} {opcode:?}");
            None
        }
        InstructionOperation::TailCall { info } => match info {
            InstrOpTailCall::MaybeDirectTailCall { .. } => {
                debug_assert!(opcode.is_jump_with_address(), "{current_rom:?} {opcode:?}");
                debug_assert!(
                    !settings.instruction_flags.j_as_branch(),
                    "{current_rom:?} {opcode:?}"
                );

                Some(FunctionEndedState::WithDelaySlot)
            }
            InstrOpTailCall::RawRegisterTailCall { .. }
            | InstrOpTailCall::DereferencedRegisterTailCall { .. } => {
                Some(FunctionEndedState::WithDelaySlot)
            }
            InstrOpTailCall::UnknownRegisterJump { .. } => Some(FunctionEndedState::WithDelaySlot),
        },

        InstructionOperation::JumptableJump { .. } => {
            debug_assert!(instr.is_jumptable_jump(), "{current_rom:?} {opcode:?}");
            None
        }

        InstructionOperation::ReturnJump => {
            debug_assert!(instr.is_return(), "{current_rom:?} {opcode:?}");

            // Found a jr $ra and there are no branches outside of this function
            if settings.detect_redundant_end() {
                // The IDO compiler may generate a a redundant and unused `jr $ra; nop` at the end of the functions the
                // flags `-g`, `-g1` or `-g2` are used.
                // In normal conditions this would be detected as its own separate empty function, which may cause
                // issues on a decompilation project.
                // In other words, we try to detect the following pattern:
                // ```
                // jr         $ra
                //  nop
                // jr         $ra
                //  nop
                // ```
                // where the last two instructions do not belong to being an already existing function (either
                // referenced by code or user-declared).
                let mut redundant_pattern_detected = false;
                if index + 3 < instrs.len() {
                    let instr1 = instrs[index + 1];
                    let instr2 = instrs[index + 2];
                    let instr3 = instrs[index + 3];
                    // We already checked if there is a function in the previous block, so we don't need to check it again.
                    if instr1.is_nop() && instr2.is_return() && instr3.is_nop() {
                        redundant_pattern_detected = true;
                    }
                }
                if !redundant_pattern_detected {
                    Some(FunctionEndedState::WithDelaySlot)
                } else {
                    None
                }
            } else {
                Some(FunctionEndedState::WithDelaySlot)
            }
        }

        InstructionOperation::Branch { .. }
        | InstructionOperation::Hi { .. }
        | InstructionOperation::PairedAddress { .. }
        | InstructionOperation::GpSet { .. }
        | InstructionOperation::DereferencedRawAddress { .. }
        | InstructionOperation::DanglingLo { .. }
        | InstructionOperation::Constant { .. }
        | InstructionOperation::UnpairedConstant { .. }
        | InstructionOperation::RegisterOperation { .. }
        | InstructionOperation::UnhandledOpcode { .. }
        | InstructionOperation::InvalidInstr {} => None,
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd)]
#[must_use]
enum NextStartResult<'segment> {
    FunctionStart(usize, Option<ReferenceWrapper<'segment, 'segment>>),
    SectionEnded,
}

fn find_current_start<'segment>(
    owned_segment: &'segment SegmentMetadata,
    section_vram: Vram,
    instrs: &[Instruction],
    index: &mut usize,
    prev_function_ref: Option<&ReferenceWrapper<'segment, 'segment>>,
    starts_data: &mut Vec<TempFunctionBoundary>,
    is_first: bool,
) -> Result<NextStartResult<'segment>, SectionCreationError> {
    let current_function_ref = if is_first && !instrs[0].is_nop() {
        owned_segment.find_reference(
            section_vram + Size::new(*index as u32 * BYTES_PER_INSTR),
            FindSettings::new(false),
        )
    } else {
        find_current_start_advance_nops(owned_segment, section_vram, instrs, index)
    };

    // We ran out of functions
    if *index >= instrs.len() {
        Ok(NextStartResult::SectionEnded)
    } else {
        // Add this function to our vec of functions

        let auto_pad_by = prev_function_ref
            .filter(|x| x.user_declared_size().is_some())
            .map(|x| x.vram());

        // Disassemble as data instead of as a function if the user set a non-function type and a size.
        let boundary = if let Some(data_type) = current_function_ref.and_then(|x| {
            x.user_declared_type()
                .filter(|t| *t != SymbolType::Function && x.user_declared_size().is_some())
        }) {
            TempFunctionBoundary::new_data(*index, auto_pad_by, data_type)
        } else {
            TempFunctionBoundary::new_func(*index, auto_pad_by)
        };

        if is_first {
            if *index == 0 {
                starts_data[0] = boundary;
            } else {
                starts_data.push(boundary);
            }
        } else if !owned_segment
            .is_vram_ignored(section_vram + Size::new(*index as u32 * BYTES_PER_INSTR))
        {
            starts_data.push(boundary);
        }

        // Decide if we need to keep checking
        if let Some((func_sym, user_size)) =
            current_function_ref.and_then(|x| x.user_declared_size().map(|y| (x, y)))
        {
            // If the user told us about the size of this function then we should blindly trust it (as long as it is valid)

            // This whole section assumes every symbol is a multiple of a word. We shouldn't let the user break that assumption.
            if user_size.inner() % BYTES_PER_INSTR != 0 {
                return Err(BadUserSymbolSizeError::new(
                    "".into(),
                    func_sym.vram(),
                    user_size,
                    SECTION_TYPE,
                    BYTES_PER_INSTR,
                )
                .into());
            }

            *index += (user_size.inner() / BYTES_PER_INSTR) as usize;
            find_current_start(
                owned_segment,
                section_vram,
                instrs,
                index,
                current_function_ref.as_ref(),
                starts_data,
                false,
            )
        } else if owned_segment
            .find_reference(
                section_vram + Size::new((*index as u32 + 1) * BYTES_PER_INSTR),
                FindSettings::new(false),
            )
            .is_some()
        {
            // Check for 1 instruction functions
            *index += 1;
            find_current_start(
                owned_segment,
                section_vram,
                instrs,
                index,
                current_function_ref.as_ref(),
                starts_data,
                false,
            )
        } else {
            Ok(NextStartResult::FunctionStart(
                *index * BYTES_PER_INSTR as usize,
                current_function_ref,
            ))
        }
    }
}

fn find_current_start_advance_nops<'segment>(
    owned_segment: &'segment SegmentMetadata,
    section_vram: Vram,
    instrs: &[Instruction],
    index: &mut usize,
) -> Option<ReferenceWrapper<'segment, 'segment>> {
    let mut current_function_ref = None;

    // Loop over until we find a instruction that isn't a nop or a referenced function
    while *index < instrs.len() {
        current_function_ref = owned_segment.find_reference(
            section_vram + Size::new(*index as u32 * BYTES_PER_INSTR),
            FindSettings::new(false),
        );

        if current_function_ref.is_some() {
            break;
        }

        let instr = &instrs[*index];
        if !instr.is_nop() {
            break;
        }

        *index += 1;
    }

    current_function_ref
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ExecutableSectionSettings {
    compiler: Option<Compiler>,
    instruction_flags: InstructionFlags,
    is_handwritten: bool,

    /// Tries to detect one or more redundants and unreferenced function ends and merge them to the previous function.
    /// This option is ignored if the compiler is not set to IDO.
    detect_redundant_end: bool,

    /// Allow considering a negative unconditional branch as a possible function end.
    /// It is disabled by default since many compilers may emit the function's epilogue even if it
    /// is unreachable.
    negative_branch_as_end: bool,
}

impl ExecutableSectionSettings {
    pub fn new(compiler: Option<Compiler>, instruction_flags: InstructionFlags) -> Self {
        Self {
            compiler,
            instruction_flags,
            is_handwritten: false,
            detect_redundant_end: false,
            negative_branch_as_end: false,
        }
    }

    pub fn compiler(&self) -> Option<Compiler> {
        self.compiler
    }
    pub fn instruction_flags(&self) -> InstructionFlags {
        self.instruction_flags
    }

    pub fn is_handwritten(&self) -> bool {
        self.is_handwritten
    }
    pub fn set_is_handwritten(&mut self, is_handwritten: bool) {
        self.is_handwritten = is_handwritten;
    }
    pub fn with_is_handwritten(self, is_handwritten: bool) -> Self {
        Self {
            is_handwritten,
            ..self
        }
    }

    pub fn detect_redundant_end(&self) -> bool {
        // TODO: move hardcoded IDO check to a Compiler function.
        self.compiler
            .is_some_and(|x| x == Compiler::IDO && self.detect_redundant_end)
    }
    /// Tries to detect one or more redundants and unreferenced function ends and merge them to the previous function.
    /// This option is ignored if the compiler is not set to IDO.
    pub fn set_detect_redundant_end(&mut self, detect_redundant_end: bool) {
        self.detect_redundant_end = detect_redundant_end;
    }
    pub fn with_detect_redundant_end(self, detect_redundant_end: bool) -> Self {
        Self {
            detect_redundant_end,
            ..self
        }
    }

    pub fn negative_branch_as_end(&self) -> bool {
        self.negative_branch_as_end
    }
    pub fn set_negative_branch_as_end(&mut self, negative_branch_as_end: bool) {
        self.negative_branch_as_end = negative_branch_as_end;
    }
    /// Allow considering a negative unconditional branch as a possible function end.
    pub fn with_negative_branch_as_end(self, negative_branch_as_end: bool) -> Self {
        Self {
            negative_branch_as_end,
            ..self
        }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl ExecutableSectionSettings {
        #[new]
        #[pyo3(signature = (compiler, instruction_flags))]
        pub fn py_new(compiler: Option<Compiler>, instruction_flags: InstructionFlags) -> Self {
            Self::new(compiler, instruction_flags)
        }

        #[pyo3(name = "set_is_handwritten")]
        pub fn py_set_is_handwritten(&mut self, is_handwritten: bool) {
            self.set_is_handwritten(is_handwritten)
        }
        #[pyo3(name = "set_detect_redundant_end")]
        pub fn py_set_detect_redundant_end(&mut self, detect_redundant_end: bool) {
            self.set_detect_redundant_end(detect_redundant_end)
        }
    }
}
