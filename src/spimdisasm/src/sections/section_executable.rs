/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use alloc::vec;
use alloc::{collections::BTreeSet, vec::Vec};

use rabbitizer::{vram::VramOffset, Instruction, InstructionFlags, IsaExtension, Vram};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::config::Compiler;
use crate::metadata::ParentSectionMetadata;
use crate::section_type::SectionType;
use crate::symbols::symbol_function::SymbolFunctionProperties;
use crate::{
    address_range::AddressRange,
    context::Context,
    context::OwnedSegmentNotFoundError,
    metadata::segment_metadata::FindSettings,
    metadata::GeneratedBy,
    metadata::SymbolMetadata,
    parent_segment_info::ParentSegmentInfo,
    rom_address::RomAddress,
    rom_vram_range::RomVramRange,
    size::Size,
    symbols::{Symbol, SymbolFunction},
};

use super::trait_section::RomSection;
use super::Section;

#[derive(Debug, Clone, Hash, PartialEq)]
#[must_use]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionExecutable {
    name: String,

    ranges: RomVramRange,

    parent_segment_info: ParentSegmentInfo,

    // in_section_offset: u32,
    // section_type: SectionType,

    //
    functions: Vec<SymbolFunction>,

    symbol_vrams: BTreeSet<Vram>,
}

impl SectionExecutable {
    pub(crate) fn new(
        context: &mut Context,
        settings: &SectionExecutableSettings,
        name: String,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<Self, OwnedSegmentNotFoundError> {
        assert!(
            !raw_bytes.is_empty(),
            "Can't initialize a section with empty bytes. {:?} {:?}",
            rom,
            vram
        );
        assert!(
            raw_bytes.len() % 4 == 0,
            "Bytes length must be a multiple of 4. {:?} {:?}",
            rom,
            vram
        );
        assert!(
            vram.inner() % 4 == 0,
            "Vram address must be aligned to 4 bytes"
        );
        assert!(
            rom.inner() % 4 == 0,
            "Rom address must be aligned to 4 bytes"
        );
        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let instrs = instrs_from_bytes(settings, context, raw_bytes, vram);
        let funcs_start_data =
            find_functions(settings, context, &parent_segment_info, ranges, &instrs)?;

        let mut functions = Vec::new();
        let mut symbol_vrams = BTreeSet::new();

        for (i, (start, _contains_invalid)) in funcs_start_data.iter().enumerate() {
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

            let properties = SymbolFunctionProperties {
                parent_metadata: ParentSectionMetadata::new(
                    name.clone(),
                    vram,
                    parent_segment_info.clone(),
                ),
                compiler: settings.compiler,
                auto_pad_by: None, // TODO
            };
            let /*mut*/ func = SymbolFunction::new(context, instrs[*start..end].into(), current_rom, current_vram, local_offset, parent_segment_info.clone(), properties)?;

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

    pub fn functions(&self) -> &[SymbolFunction] {
        &self.functions
    }
}

/*
impl SectionExecutable<'_, '_> {
}
*/

impl Section for SectionExecutable {
    fn name(&self) -> &str {
        &self.name
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

    fn symbols_vrams(&self) -> &BTreeSet<Vram> {
        &self.symbol_vrams
    }
}

impl RomSection for SectionExecutable {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}

fn instrs_from_bytes(
    settings: &SectionExecutableSettings,
    context: &Context,
    raw_bytes: &[u8],
    mut vram: Vram,
) -> Vec<Instruction> {
    let mut instrs = Vec::new();
    let endian = context.global_config().endian();

    for b in raw_bytes.chunks_exact(4) {
        let word = endian.word_from_bytes(b);

        instrs.push(Instruction::new(word, vram, settings.instruction_flags));
        vram += VramOffset::new(4);
    }

    instrs
}

fn find_functions(
    settings: &SectionExecutableSettings,
    context: &mut Context,
    parent_segment_info: &ParentSegmentInfo,
    section_ranges: RomVramRange,
    instrs: &[Instruction],
) -> Result<Vec<(usize, bool)>, OwnedSegmentNotFoundError> {
    if instrs.is_empty() {
        return Ok(vec![(0, false)]);
    }

    let owned_segment = context.find_owned_segment(parent_segment_info)?;
    let mut starts_data = Vec::new();

    let mut function_ended = false;
    let mut farthest_branch = VramOffset::new(0);
    let mut halt_function_searching;

    let mut local_offset = 0;
    let mut current_function_start = local_offset;
    let mut current_function_sym = owned_segment.find_symbol(
        section_ranges.vram().start() + Size::new(local_offset as u32),
        FindSettings::new().with_allow_addend(false),
    );
    let mut index = 0;

    let mut prev_start = index;
    let mut contains_invalid = false;
    let mut is_likely_handwritten = settings.is_handwritten;

    let mut prev_func_had_user_declared_size = false;

    if instrs[0].is_nop() {
        // Loop over until we find a instruction that isn't a nop
        while index < instrs.len() {
            if current_function_sym.is_some() {
                break;
            }

            if !instrs[index].is_nop() {
                break;
            }

            index += 1;
            local_offset += 4;
            current_function_start = local_offset;
            current_function_sym = owned_segment.find_symbol(
                section_ranges.vram().start() + Size::new(local_offset as u32),
                FindSettings::new().with_allow_addend(false),
            );
        }

        if index != 0 {
            starts_data.push((prev_start, contains_invalid));
            prev_start = index;
        }
    }

    while index < instrs.len() {
        if !instrs[index].is_valid() {
            contains_invalid = false;
        }

        if function_ended {
            //function_ended = false;

            is_likely_handwritten = settings.is_handwritten;
            index += 1;
            local_offset += 4;

            let mut aux_sym = context
                .find_owned_segment(parent_segment_info)?
                .find_symbol(
                    section_ranges.vram().start() + Size::new(local_offset as u32),
                    FindSettings::new().with_allow_addend(false),
                );

            // Loop over until we find a instruction that isn't a nop
            while index < instrs.len() {
                if aux_sym.is_some() {
                    break;
                }

                let instr = &instrs[index];
                if !instr.is_nop() {
                    break;
                }

                index += 1;
                local_offset += 4;

                aux_sym = context
                    .find_owned_segment(parent_segment_info)?
                    .find_symbol(
                        section_ranges.vram().start() + Size::new(local_offset as u32),
                        FindSettings::new().with_allow_addend(false),
                    );
            }

            current_function_start = local_offset;
            current_function_sym = aux_sym;

            starts_data.push((prev_start, contains_invalid));
            prev_start = index;

            if index >= instrs.len() {
                return Ok(starts_data);
            }

            if prev_func_had_user_declared_size {
                let aux_sym = context
                    .find_owned_segment_mut(parent_segment_info)?
                    .add_function(
                        section_ranges.vram().start() + Size::new(local_offset as u32),
                        Some(section_ranges.rom().start() + Size::new(local_offset as u32)),
                        GeneratedBy::Autogenerated,
                    );
                aux_sym.set_auto_created_pad_by(section_ranges.vram().start());
                // TODO: figure out a way to avoid having to search the symbol we just created, hopefully by reusing the above `aux_sym`.
                current_function_sym = context
                    .find_owned_segment(parent_segment_info)?
                    .find_symbol(
                        section_ranges.vram().start() + Size::new(local_offset as u32),
                        FindSettings::new().with_allow_addend(false),
                    );
            }

            // prev_func_had_user_declared_size = false;

            contains_invalid = !instrs[index].is_valid();
        }

        let instr = &instrs[index];

        if instr.isa_extension() != IsaExtension::RSP && !is_likely_handwritten {
            is_likely_handwritten = instr.is_likely_handwritten();
        }

        if instr.opcode().is_branch() || instr.is_unconditional_branch() {
            (farthest_branch, halt_function_searching) = find_functions_branch_checker(
                context,
                parent_segment_info,
                section_ranges,
                local_offset,
                instr,
                &mut starts_data,
                farthest_branch,
                is_likely_handwritten,
                contains_invalid,
            )?;
            if halt_function_searching {
                break;
            }
        }

        (function_ended, prev_func_had_user_declared_size) = find_functions_check_function_ended(
            context,
            settings,
            parent_segment_info,
            local_offset,
            instr,
            index,
            instrs,
            section_ranges.rom().start() + Size::new(local_offset as u32),
            section_ranges.vram().start() + Size::new(local_offset as u32),
            current_function_sym,
            farthest_branch,
            current_function_start,
            is_likely_handwritten,
        )?;

        index += 1;
        //farthest_branch -= 4;
        farthest_branch = VramOffset::new(farthest_branch.inner() - 4);
        local_offset += 4;
    }

    if prev_start != index {
        starts_data.push((prev_start, contains_invalid));
    }

    Ok(starts_data)
}

#[allow(clippy::too_many_arguments)]
fn find_functions_branch_checker(
    context: &Context,
    parent_segment_info: &ParentSegmentInfo,
    section_ranges: RomVramRange,
    local_offset: usize,
    instr: &Instruction,
    starts_data: &mut Vec<(usize, bool)>,
    mut farthest_branch: VramOffset,
    is_likely_handwritten: bool,
    contains_invalid: bool,
) -> Result<(VramOffset, bool), OwnedSegmentNotFoundError> {
    let mut halt_function_searching = false;

    if instr.opcode().is_jump_with_address() {
        // If this instruction is a jump and it is jumping to a function then
        // don't treat it as a branch, it is probably actually being used as
        // a jump

        // TODO
        if let Some(target_vram) = instr.get_instr_index_as_vram() {
            if let Some(aux_sym) = context
                .find_owned_segment(parent_segment_info)?
                .find_symbol(
                    target_vram,
                    FindSettings::new()
                        .with_allow_addend(false)
                        .with_check_upper_limit(false),
                )
            {
                if aux_sym.is_trustable_function() {
                    return Ok((farthest_branch, halt_function_searching));
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
                let owned_segment = context.find_owned_segment(parent_segment_info)?;
                let mut j = starts_data.len() as i32 - 1;
                while j >= 0 {
                    let other_func_start_offset = starts_data[j as usize].0 * 4;
                    if branch_offset.inner() + (local_offset as i32)
                        < (other_func_start_offset as i32)
                    {
                        let vram = section_ranges.vram().start() + Size::new(local_offset as u32);

                        // TODO: invert check?
                        if let Some(func_symbol) = owned_segment
                            .find_symbol(vram, FindSettings::new().with_allow_addend(false))
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
    }

    Ok((farthest_branch, halt_function_searching))
}

#[allow(clippy::too_many_arguments)]
fn find_functions_check_function_ended(
    context: &Context,
    settings: &SectionExecutableSettings,
    parent_segment_info: &ParentSegmentInfo,
    local_offset: usize,
    instr: &Instruction,
    _index: usize,
    _instrs: &[Instruction],
    current_rom: RomAddress,
    current_vram: Vram,
    current_function_sym: Option<&SymbolMetadata>,
    farthest_branch: VramOffset,
    current_function_start: usize,
    _is_likely_handwritten: bool,
) -> Result<(bool, bool), OwnedSegmentNotFoundError> {
    let mut function_ended = false;
    let mut prev_func_had_user_declared_size = false;
    // TODO

    if let Some(sym) = current_function_sym {
        if let Some(user_declared_size) = sym.user_declared_size() {
            // If the function has a size set by the user then only use that and ignore the other ways of determining function-ends
            if local_offset + 8 == current_function_start + user_declared_size.inner() as usize {
                function_ended = true;
                prev_func_had_user_declared_size = true;
            }
            return Ok((function_ended, prev_func_had_user_declared_size));
        }
    }

    let owned_segment = context.find_owned_segment(parent_segment_info)?;
    let func_sym = owned_segment.find_symbol(
        current_vram + VramOffset::new(8),
        FindSettings::new().with_allow_addend(false),
    );
    if let Some(sym) = func_sym {
        // # If there's another function after this then the current function has ended
        if sym.is_trustable_function() {
            if let Some(sym_rom) = sym.rom() {
                if current_rom + Size::new(8) == sym_rom {
                    return Ok((true, false));
                }
            } else {
                return Ok((true, false));
            }
        }
    }

    if !farthest_branch.is_positive() && instr.opcode().is_jump() {
        if instr.is_return() {
            // Found a jr $ra and there are no branches outside of this function
            if false { // redundant function end detection
                 // TODO
            } else {
                return Ok((true, false));
            }
        } else if instr.is_jumptable_jump() {
            // Usually jumptables, ignore
        } else if instr.opcode().does_link() {
            // Just a function call, nothing to see here
        } else if instr.opcode().is_jump_with_address() {
            // If this instruction is a jump and it is jumping to a function then
            // we can consider this as a function end. This can happen as a
            // tail-optimization in "modern" compilers
            if !settings.instruction_flags.j_as_branch() {
                return Ok((true, false));
            } else {
                // j is a jump, but it could be jumping to a function
                if let Some(target_vram) = instr.get_instr_index_as_vram() {
                    if let Some(aux_sym) = owned_segment
                        .find_symbol(target_vram, FindSettings::new().with_allow_addend(false))
                    {
                        if aux_sym.is_trustable_function() && Some(aux_sym) != current_function_sym
                        {
                            return Ok((true, false));
                        }
                    }
                }
            }
        }
    }

    Ok((function_ended, prev_func_had_user_declared_size))
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionExecutableSettings {
    compiler: Option<Compiler>,
    instruction_flags: InstructionFlags,
    is_handwritten: bool,
}

impl SectionExecutableSettings {
    pub fn new(compiler: Option<Compiler>, instruction_flags: InstructionFlags) -> Self {
        Self {
            compiler,
            instruction_flags,
            is_handwritten: false,
        }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use crate::symbols::display::{FunctionDisplaySettings, SymDisplayError};

    use super::*;

    #[pymethods]
    impl SectionExecutableSettings {
        #[new]
        #[pyo3(signature = (compiler))]
        pub fn py_new(compiler: Option<Compiler>, /*instruction_flags: InstructionFlags*/) -> Self {
            Self::new(compiler, InstructionFlags::default())
        }
    }

    #[pymethods]
    impl SectionExecutable {
        #[pyo3(name = "sym_count")]
        pub fn py_sym_count(&self) -> usize {
            self.functions.len()
        }

        #[pyo3(name = "display_sym")]
        pub fn py_display_sym(
            &self,
            context: &Context,
            index: usize,
            settings: &FunctionDisplaySettings,
        ) -> Result<Option<String>, SymDisplayError> {
            let sym = self.functions.get(index);

            Ok(if let Some(sym) = sym {
                Some(sym.display(context, settings)?.to_string())
            } else {
                None
            })
        }
    }
}
