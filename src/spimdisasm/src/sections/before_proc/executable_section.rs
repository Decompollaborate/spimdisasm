/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;

use rabbitizer::{Instruction, InstructionFlags, IsaExtension};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, RomVramRange, Size, Vram, VramOffset};
use crate::analysis::{InstrOpTailCall, InstructionOperation, ReferenceWrapper, RegisterTracker};
use crate::collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet};
use crate::config::{Compiler, Endian, GlobalConfig};
use crate::context::Context;
use crate::metadata::{ParentSectionMetadata, SegmentMetadata};
use crate::parent_segment_info::ParentSegmentInfo;
use crate::relocation::RelocationInfo;
use crate::section_type::SectionType;
use crate::sections::processed::ExecutableSectionProcessed;
use crate::sections::{
    BadBytesSizeError, EmptySectionError, RomSectionPreprocessed, SectionPreprocessed,
    UnalignedRomError, UnalignedVramError,
};
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

#[derive(Debug, Clone)]
#[must_use]
pub struct ExecutableSection {
    name: Arc<str>,

    ranges: RomVramRange,

    parent_segment_info: ParentSegmentInfo,

    // in_section_offset: u32,
    // section_type: SectionType,

    //
    functions: Vec<FunctionSym>,

    symbol_vrams: UnorderedSet<Vram>,
}

impl ExecutableSection {
    pub(crate) fn new(
        context: &mut Context,
        settings: &ExecutableSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<Self, SectionCreationError> {
        if raw_bytes.is_empty() {
            return Err(EmptySectionError::new(name, vram).into());
        }
        if raw_bytes.len() % 4 != 0 {
            return Err(BadBytesSizeError::new(name, raw_bytes.len(), 4).into());
        }
        if vram.inner() % 4 != 0 {
            return Err(UnalignedVramError::new(name, vram, 4).into());
        }
        if rom.inner() % 4 != 0 {
            return Err(UnalignedRomError::new(name, rom, 4).into());
        }

        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let instrs = instrs_from_bytes(settings, context.global_config().endian(), raw_bytes, vram);
        debug_assert!(!instrs.is_empty(), "{}, {:?}, {:?}", name, vram, rom);

        let owned_segment = context.find_owned_segment(&parent_segment_info)?;
        let funcs_start_data = find_functions(
            context.global_config(),
            settings,
            owned_segment,
            ranges,
            &instrs,
        );

        debug_assert!(
            !funcs_start_data.is_empty(),
            "{}, {:?}, {:?}",
            name,
            vram,
            rom
        );

        let mut functions = Vec::new();
        let mut symbol_vrams = UnorderedSet::new();

        for (i, (start, auto_pad_by)) in funcs_start_data.iter().enumerate() {
            let end = if i + 1 < funcs_start_data.len() {
                funcs_start_data[i + 1].0
            } else {
                instrs.len()
            };
            debug_assert!(*start < end, "{:?} {} {} {}", rom, vram, *start, end);

            let local_offset = start * 4;
            let s = Size::new(local_offset as u32);
            let current_vram = vram + s;
            let current_rom = rom + s;

            symbol_vrams.insert(vram);

            let properties = FunctionSymProperties {
                parent_metadata: ParentSectionMetadata::new(
                    name.clone(),
                    vram,
                    parent_segment_info.clone(),
                ),
                compiler: settings.compiler,
                auto_pad_by: auto_pad_by.map(|x| ranges.vram().start() + Size::new(x as u32)),
            };
            let /*mut*/ func = FunctionSym::new(context, instrs[*start..end].into(), current_rom, current_vram, local_offset, parent_segment_info.clone(), properties)?;

            functions.push(func);
        }

        Ok(Self {
            name,
            ranges,
            parent_segment_info,
            functions,
            symbol_vrams,
        })
    }
}

impl ExecutableSection {
    pub fn functions(&self) -> &[FunctionSym] {
        &self.functions
    }
}

impl ExecutableSection {
    pub fn post_process(
        self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<ExecutableSectionProcessed, SectionPostProcessError> {
        ExecutableSectionProcessed::new(
            context,
            self.name,
            self.ranges,
            self.parent_segment_info,
            self.functions,
            self.symbol_vrams,
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

    #[must_use]
    fn section_type(&self) -> SectionType {
        SectionType::Text
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.functions
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
        &self.functions
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

    for b in raw_bytes.chunks_exact(4) {
        let word = endian.word_from_bytes(b);

        instrs.push(Instruction::new(word, vram, settings.instruction_flags));
        vram += VramOffset::new(4);
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
        self.farthest = VramOffset::new(self.farthest.inner() - 4);
    }
}

fn find_functions(
    global_config: &GlobalConfig,
    settings: &ExecutableSectionSettings,
    owned_segment: &SegmentMetadata,
    section_ranges: RomVramRange,
    instrs: &[Instruction],
) -> Vec<(usize, Option<usize>)> {
    if instrs.is_empty() {
        return vec![(0, None)];
    }

    let mut starts_data = Vec::new();

    let mut function_ended = FunctionEndedState::No;
    let mut farthest_branch = FarthestBranch::new(section_ranges.rom().start());

    let mut index: usize = 0;
    let mut current_function_start = index * 4;
    let mut current_function_ref = owned_segment.find_reference(
        section_ranges.vram().start() + Size::new(index as u32 * 4),
        FindSettings::new(false),
    );

    let mut prev_start = index;
    let mut is_likely_handwritten = settings.is_handwritten;

    let mut prev_func_had_user_declared_size = false;

    if instrs[0].is_nop() {
        // Loop over until we find a instruction that isn't a nop
        while index < instrs.len() {
            if current_function_ref.is_some() {
                break;
            }

            if !instrs[index].is_nop() {
                break;
            }

            index += 1;
            current_function_start = index * 4;
            current_function_ref = owned_segment.find_reference(
                section_ranges.vram().start() + Size::new(index as u32 * 4),
                FindSettings::new(false),
            );
        }

        if index != 0 {
            starts_data.push((prev_start, None));
            prev_start = index;
        }
    }

    let mut auto_pad_by = None;
    let mut regs_tracker = RegisterTracker::new(
        settings.instruction_flags().abi(),
        Some(section_ranges.vram().start() + Size::new(index as u32 * 4)),
        global_config.gp_config().copied(),
        global_config.endian(),
    );
    let mut prev_instr = None;

    let global_offset_table = owned_segment.global_offset_table();

    while index < instrs.len() {
        if function_ended != FunctionEndedState::No {
            is_likely_handwritten = settings.is_handwritten;

            if function_ended == FunctionEndedState::WithDelaySlot {
                index += 1;
            }

            let mut aux_ref = owned_segment.find_reference(
                section_ranges.vram().start() + Size::new(index as u32 * 4),
                FindSettings::new(false),
            );

            // Loop over until we find a instruction that isn't a nop
            while index < instrs.len() {
                if aux_ref.is_some() {
                    break;
                }

                let instr = &instrs[index];
                if !instr.is_nop() {
                    break;
                }

                index += 1;

                aux_ref = owned_segment.find_reference(
                    section_ranges.vram().start() + Size::new(index as u32 * 4),
                    FindSettings::new(false),
                );
            }

            current_function_start = index * 4;
            current_function_ref = aux_ref;

            if !owned_segment
                .is_vram_ignored(section_ranges.vram().start() + Size::new(prev_start as u32 * 4))
            {
                starts_data.push((prev_start, auto_pad_by));
                auto_pad_by = if prev_func_had_user_declared_size {
                    Some(prev_start)
                } else {
                    None
                };
            }

            prev_start = index;

            if index >= instrs.len() {
                return starts_data;
            }

            prev_instr = None;
            regs_tracker.soft_reset(
                instrs[index].abi(),
                Some(section_ranges.vram().start() + Size::new(index as u32 * 4)),
            );
        }

        let instr = &instrs[index];

        if instr.isa_extension() != Some(IsaExtension::RSP) && !is_likely_handwritten {
            is_likely_handwritten = instr.is_likely_handwritten();
        }

        let current_rom = section_ranges.rom().start() + Size::new(index as u32 * 4);
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
                index * 4,
                instr,
                &mut farthest_branch,
            );
        }

        (function_ended, prev_func_had_user_declared_size) = find_functions_check_function_ended(
            owned_segment,
            &instr_processed_result,
            settings,
            index,
            instrs,
            section_ranges.rom().start() + Size::new(index as u32 * 4),
            section_ranges.vram().start() + Size::new(index as u32 * 4),
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

    if prev_start != index
        && !owned_segment
            .is_vram_ignored(section_ranges.vram().start() + Size::new(prev_start as u32 * 4))
    {
        starts_data.push((prev_start, auto_pad_by));
    }

    starts_data
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
    No,
    WithDelaySlot,
    ByException,
}

// returns `(function_ended, prev_func_had_user_declared_size)`
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
) -> (FunctionEndedState, bool) {
    let instr = &instrs[index];
    let opcode = instr.opcode();

    if let Some(reference) = current_function_ref {
        if let Some(user_declared_size) = reference.user_declared_size() {
            // If the function has a size set by the user then only use that and ignore the other ways of determining function-ends
            return if (index + 2) * 4
                == current_function_start + user_declared_size.inner() as usize
            {
                (FunctionEndedState::WithDelaySlot, true)
            } else {
                (FunctionEndedState::No, false)
            };
        }
    }

    if let Some(reference) =
        owned_segment.find_reference(current_vram + VramOffset::new(8), FindSettings::new(false))
    {
        // If there's another function after this then the current function has ended
        if reference.is_trustable_function() {
            if let Some(sym_rom) = reference.rom() {
                if current_rom + Size::new(8) == sym_rom {
                    return (FunctionEndedState::WithDelaySlot, false);
                }
            } else {
                return (FunctionEndedState::WithDelaySlot, false);
            }
        }
    }

    if farthest_branch.farthest().is_positive() {
        // We still have a branch that branched even farther than where we currently are, so we
        // must still be inside the same function.
        return (FunctionEndedState::No, false);
    }

    if opcode.causes_unconditional_exception() && !opcode.causes_returnable_exception() {
        return (FunctionEndedState::ByException, false);
    }

    if settings.negative_branch_as_end() && instr.is_unconditional_branch() {
        if let Some(target_vram) = instr.get_branch_vram_generic() {
            if target_vram < current_vram {
                return (FunctionEndedState::WithDelaySlot, false);
            }
        }
    }

    if !opcode.is_jump() {
        return (FunctionEndedState::No, false);
    }

    match instr_processed_result {
        InstructionOperation::Link { .. } => {
            debug_assert!(opcode.does_link(), "{:?} {:?}", current_rom, opcode);
            (FunctionEndedState::No, false)
        }
        InstructionOperation::TailCall { info } => match info {
            InstrOpTailCall::MaybeDirectTailCall { .. } => {
                debug_assert!(
                    opcode.is_jump_with_address(),
                    "{:?} {:?}",
                    current_rom,
                    opcode
                );
                debug_assert!(
                    !settings.instruction_flags.j_as_branch(),
                    "{:?} {:?}",
                    current_rom,
                    opcode
                );

                (FunctionEndedState::WithDelaySlot, false)
            }
            InstrOpTailCall::RawRegisterTailCall { .. }
            | InstrOpTailCall::DereferencedRegisterTailCall { .. } => {
                (FunctionEndedState::WithDelaySlot, false)
            }
            InstrOpTailCall::UnknownRegisterJump { .. } => {
                (FunctionEndedState::WithDelaySlot, false)
            }
        },

        InstructionOperation::JumptableJump { .. } => {
            debug_assert!(instr.is_jumptable_jump(), "{:?} {:?}", current_rom, opcode);
            (FunctionEndedState::No, false)
        }

        InstructionOperation::ReturnJump => {
            debug_assert!(instr.is_return(), "{:?} {:?}", current_rom, opcode);

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
                    (FunctionEndedState::WithDelaySlot, false)
                } else {
                    (FunctionEndedState::No, false)
                }
            } else {
                (FunctionEndedState::WithDelaySlot, false)
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
        | InstructionOperation::InvalidInstr {} => (FunctionEndedState::No, false),
    }
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
