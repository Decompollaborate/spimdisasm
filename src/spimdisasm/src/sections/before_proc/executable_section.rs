/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;

use rabbitizer::{Instruction, InstructionFlags, IsaExtension};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, RomVramRange, Size, Vram, VramOffset};
use crate::analysis::{ReferenceWrapper, RegisterTracker};
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
        let owned_segment = context.find_owned_segment(&parent_segment_info)?;
        let funcs_start_data = find_functions(
            context.global_config(),
            settings,
            owned_segment,
            ranges,
            &instrs,
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

    let mut function_ended = false;
    let mut farthest_branch = VramOffset::new(0);
    let mut halt_function_searching;

    let mut index: usize = 0;
    let mut current_function_start = index * 4;
    let mut current_function_ref = owned_segment.find_reference(
        section_ranges.vram().start() + Size::new(index as u32 * 4),
        FindSettings::new(false),
    );

    let mut prev_start = index;
    let mut contains_invalid = false;
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
    let mut regs_tracker = RegisterTracker::new();
    let mut prev_instr = None;

    while index < instrs.len() {
        if !instrs[index].is_valid() {
            contains_invalid = true;
        }

        if function_ended {
            //function_ended = false;

            is_likely_handwritten = settings.is_handwritten;
            index += 1;

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

            contains_invalid = !instrs[index].is_valid();
        }

        let instr = &instrs[index];

        if instr.isa_extension() != Some(IsaExtension::RSP) && !is_likely_handwritten {
            is_likely_handwritten = instr.is_likely_handwritten();
        }

        let current_rom = section_ranges.rom().start() + Size::new(index as u32 * 4);
        run_register_tracker_start(global_config, &mut regs_tracker, instr, current_rom);

        if instr.opcode().is_branch()
            || instr.is_unconditional_branch()
            || instr.is_jumptable_jump()
        {
            (farthest_branch, halt_function_searching) = find_functions_branch_checker(
                owned_segment,
                &regs_tracker,
                section_ranges,
                index * 4,
                instr,
                &mut starts_data,
                farthest_branch,
                is_likely_handwritten,
                contains_invalid,
            );
            if halt_function_searching {
                break;
            }
        }

        (function_ended, prev_func_had_user_declared_size) = find_functions_check_function_ended(
            owned_segment,
            settings,
            instr,
            index,
            instrs,
            section_ranges.rom().start() + Size::new(index as u32 * 4),
            section_ranges.vram().start() + Size::new(index as u32 * 4),
            current_function_ref,
            farthest_branch,
            current_function_start,
        );

        run_register_tracker_end(&mut regs_tracker, instr, prev_instr, current_rom);

        if instr.is_valid() {
            prev_instr = Some(instr);
        } else {
            prev_instr = None;
        }

        index += 1;
        //farthest_branch -= 4;
        farthest_branch = VramOffset::new(farthest_branch.inner() - 4);
    }

    if prev_start != index
        && !owned_segment
            .is_vram_ignored(section_ranges.vram().start() + Size::new(prev_start as u32 * 4))
    {
        starts_data.push((prev_start, auto_pad_by));
    }

    starts_data
}

#[allow(clippy::too_many_arguments)]
fn find_functions_branch_checker(
    owned_segment: &SegmentMetadata,
    regs_tracker: &RegisterTracker,
    section_ranges: RomVramRange,
    local_offset: usize,
    instr: &Instruction,
    starts_data: &mut Vec<(usize, Option<usize>)>,
    mut farthest_branch: VramOffset,
    is_likely_handwritten: bool,
    contains_invalid: bool,
) -> (VramOffset, bool) {
    let mut halt_function_searching = false;

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
                    return (farthest_branch, halt_function_searching);
                }
            }
        }
    }

    if let Some(branch_offset) = instr.get_branch_offset_generic() {
        if branch_offset > farthest_branch {
            // Keep track of the farthest branch target
            farthest_branch = branch_offset;
        }
        if branch_offset.is_negative() {
            // Check backwards branches

            if (branch_offset.inner() + (local_offset as i32) < 0)
                && (!instr.opcode().is_jump() || instr.flags().j_as_branch())
            {
                // Whatever we are reading is not a valid instruction, it doesn't make sense for
                // an instruction to backwards-branch outside of the function.

                // Except for `j`, its behavior depends in if we are treating it as a branch or not.
                // Jumping outside of the function is fine, but branching isn't.
                halt_function_searching = true;
            } else if !is_likely_handwritten && !contains_invalid {
                let mut j = starts_data.len() as i32 - 1;
                while j >= 0 {
                    let other_func_start_offset = starts_data[j as usize].0 * 4;
                    if branch_offset.inner() + (local_offset as i32)
                        < (other_func_start_offset as i32)
                    {
                        let vram = section_ranges.vram().start() + Size::new(local_offset as u32);

                        // TODO: invert check?
                        if let Some(func_symbol) =
                            owned_segment.find_reference(vram, FindSettings::new(false))
                        {
                            // TODO
                            if func_symbol.is_trustable_function() {
                                j -= 1;
                                continue;
                            }
                        }
                        starts_data.remove(j as usize);
                    } else {
                        break;
                    }
                    j -= 1;
                }
            }
        }
    } else if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(instr) {
        // Check jumptables
        if jr_reg_data.branch_info().is_none() {
            let jumptable_address = Vram::new(jr_reg_data.address());
            if let Some(jumptable_ref) =
                owned_segment.find_reference(jumptable_address, FindSettings::new(false))
            {
                for jtbl_label_vram in jumptable_ref.table_labels() {
                    let branch_offset = *jtbl_label_vram - instr.vram();

                    if branch_offset > farthest_branch {
                        farthest_branch = branch_offset;
                    }
                }
            }
        }
    }

    (farthest_branch, halt_function_searching)
}

// returns `(function_ended, prev_func_had_user_declared_size)`
#[allow(clippy::too_many_arguments)]
fn find_functions_check_function_ended(
    owned_segment: &SegmentMetadata,
    settings: &ExecutableSectionSettings,
    instr: &Instruction,
    index: usize,
    instrs: &[Instruction],
    current_rom: Rom,
    current_vram: Vram,
    current_function_ref: Option<ReferenceWrapper>,
    farthest_branch: VramOffset,
    current_function_start: usize,
) -> (bool, bool) {
    if let Some(reference) = current_function_ref {
        if let Some(user_declared_size) = reference.user_declared_size() {
            // If the function has a size set by the user then only use that and ignore the other ways of determining function-ends
            return if (index + 2) * 4
                == current_function_start + user_declared_size.inner() as usize
            {
                (true, true)
            } else {
                (false, false)
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
                    return (true, false);
                }
            } else {
                return (true, false);
            }
        }
    }

    if farthest_branch.is_positive() {
        return (false, false);
    }
    if !instr.opcode().is_jump() {
        return (false, false);
    }

    if instr.is_return() {
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
                return (true, false);
            }
        } else {
            return (true, false);
        }
    } else if instr.is_jumptable_jump() {
        // Usually jumptables, ignore
    } else if instr.opcode().does_link() {
        // Just a function call, nothing to see here
    } else if instr.opcode().is_jump_with_address() {
        // If this instruction is a jump and it is jumping to a function then
        // we can consider this as a function end. This can happen as a
        // tail-optimization in "modern" compilers
        if settings.instruction_flags.j_as_branch() {
            return (false, false);
        } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
            if let Some(aux_ref) =
                owned_segment.find_reference(target_vram, FindSettings::new(false))
            {
                if aux_ref.is_trustable_function() && Some(aux_ref) != current_function_ref {
                    return (true, false);
                }
            }
        }
    }

    (false, false)
}

fn run_register_tracker_start(
    global_config: &GlobalConfig,
    regs_tracker: &mut RegisterTracker,
    instr: &Instruction,
    current_rom: Rom,
) {
    if !instr.is_valid() {
        return;
    }

    if let Some(_target_vram) = instr.get_branch_vram_generic() {
        // instr.opcode().is_branch() or instr.is_unconditional_branch()
        regs_tracker.process_branch(instr, current_rom);
    } else if let Some(_target_vram) = instr.get_instr_index_as_vram() {
        // instr.opcode().is_jump_with_address()
    } else if instr.is_jumptable_jump() {
        //self.process_jumptable_jump(context, regs_tracker, instr, instr_rom);
        if let Some(_jr_reg_data) = regs_tracker.get_jr_reg_data(instr) {}
    } else if instr.opcode().is_jump() && instr.opcode().does_link() {
        // `jalr`. Implicit `!is_jump_with_address`
        // We can only mark the referenced address as a function if that address was not dereferenced.
        // i.e. `la $t9, some_func; jalr $t9`.
        // Dereferenced symbols are usually some kind of callback, like an array of functions.
        // Currently `get_jr_reg_data` only returns `Some` if the register was dereferenced, so we can't really use it here.
        /*
        if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(&instr) {
            let address = Vram::new(jr_reg_data.address());

            let reference = self.new_ref(address, None, owned_segment);
            reference.set_sym_type(SymbolType::Function);
        }
        */
    } else if instr.opcode().can_be_hi() {
        regs_tracker.process_hi(instr, current_rom);
    } else if instr.opcode().can_be_lo() {
        if let Some(pairing_info) = regs_tracker.preprocess_lo_and_get_info(instr, current_rom) {
            if pairing_info.is_gp_got {
                // TODO
            } else if let Some(lower_half) = instr.get_processed_immediate() {
                let address = if pairing_info.is_gp_got {
                    None
                } else if pairing_info.is_gp_rel {
                    // TODO: should check for global_config.gp_config().is_some_and(|x| !x.pic())?
                    global_config.gp_config().map(|gp_config| {
                        Vram::new(gp_config.gp_value().inner().wrapping_add_signed(lower_half))
                    })
                } else {
                    Some(Vram::new(pairing_info.value as u32) + VramOffset::new(lower_half))
                };

                if let Some(address) = address {
                    regs_tracker.process_lo(instr, address.inner(), current_rom);
                }
            }
        }
    } else if instr.opcode().can_be_unsigned_lo() {
        // TODO
    }
}

fn run_register_tracker_end(
    regs_tracker: &mut RegisterTracker,
    instr: &Instruction,
    prev_instr: Option<&Instruction>,
    current_rom: Rom,
) {
    if instr.is_valid() {
        regs_tracker.overwrite_registers(instr, current_rom);
    }

    if let Some(prev) = &prev_instr {
        if prev.is_function_call() {
            regs_tracker.unset_registers_after_func_call(prev);
        } else if (prev.opcode().is_jump() && !prev.opcode().does_link())
            || prev.is_unconditional_branch()
        {
            regs_tracker.clear();
        }
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
}

impl ExecutableSectionSettings {
    pub fn new(compiler: Option<Compiler>, instruction_flags: InstructionFlags) -> Self {
        Self {
            compiler,
            instruction_flags,
            is_handwritten: false,
            detect_redundant_end: false,
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
    pub fn detect_redundant_end(&self) -> bool {
        // TODO: move hardcoded IDO check to a Compiler function.
        self.compiler
            .is_some_and(|x| x == Compiler::IDO && self.detect_redundant_end)
    }

    pub fn set_is_handwritten(&mut self, is_handwritten: bool) {
        self.is_handwritten = is_handwritten;
    }

    /// Tries to detect one or more redundants and unreferenced function ends and merge them to the previous function.
    /// This option is ignored if the compiler is not set to IDO.
    pub fn set_detect_redundant_end(&mut self, detect_redundant_end: bool) {
        self.detect_redundant_end = detect_redundant_end;
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
