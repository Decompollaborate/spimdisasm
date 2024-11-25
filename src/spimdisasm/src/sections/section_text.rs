/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use alloc::vec;
use alloc::{collections::BTreeSet, vec::Vec};

use rabbitizer::{vram::VramOffset, Instruction, InstructionFlags, IsaExtension, Vram};

use crate::context::OwnedSegmentNotFoundError;
use crate::metadata::segment_metadata::FindSettings;
use crate::metadata::GeneratedBy;
use crate::parent_segment_info::ParentSegmentInfo;
use crate::size::Size;
use crate::{
    context::Context,
    metadata::SymbolMetadata,
    rom_address::RomAddress,
    symbols::{Symbol, SymbolFunction},
};

use super::{Section, SectionBase};

pub struct SectionTextSettings {
    instruction_flags: InstructionFlags,
    is_handwritten: bool,
}

impl SectionTextSettings {
    pub fn new(instruction_flags: InstructionFlags) -> Self {
        Self {
            instruction_flags,
            is_handwritten: false,
        }
    }
}

pub struct SectionText {
    section_base: SectionBase,

    functions: Vec<SymbolFunction>,

    symbol_vrams: BTreeSet<Vram>,
}

impl SectionText {
    pub fn new(
        context: &mut Context,
        settings: SectionTextSettings,
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
            vram.inner() % 4 == 0,
            "Vram address must be aligned to 4 bytes"
        );
        assert!(
            rom.inner() % 4 == 0,
            "Rom address must be aligned to 4 bytes"
        );

        let mut section_base = SectionBase::new(name, Some(rom), vram, parent_segment_info);
        let instrs = instrs_from_bytes(&settings, context, raw_bytes, vram);
        let funcs_start_data = find_functions(&settings, context, &mut section_base, &instrs)?;

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
            let vram = section_base.vram_offset(local_offset);
            let rom = section_base.rom_offset(local_offset);

            symbol_vrams.insert(vram);

            // TODO: get rid of unwrap?
            let /*mut*/ func = SymbolFunction::new(context, instrs[*start..end].into(), rom.unwrap(), vram, local_offset, section_base.parent_segment_info())?;

            functions.push(func);
        }

        Ok(Self {
            section_base,
            functions,
            symbol_vrams,
        })
    }

    pub fn name(&self) -> &str {
        self.section_base.name()
    }

    // TODO: remove
    pub fn functions(&self) -> &[SymbolFunction] {
        &self.functions
    }
}

/*
impl SectionText<'_, '_> {
}
*/

impl Section for SectionText {
    fn section_base(&self) -> &SectionBase {
        &self.section_base
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.functions
    }

    fn symbols_vrams(&self) -> &BTreeSet<Vram> {
        &self.symbol_vrams
    }
}

fn instrs_from_bytes(
    settings: &SectionTextSettings,
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
    settings: &SectionTextSettings,
    context: &mut Context,
    section_base: &mut SectionBase,
    instrs: &[Instruction],
) -> Result<Vec<(usize, bool)>, OwnedSegmentNotFoundError> {
    if instrs.is_empty() {
        return Ok(vec![(0, false)]);
    }

    let owned_segment = context.find_owned_segment(section_base.parent_segment_info())?;
    let mut starts_data = Vec::new();

    let mut function_ended = false;
    let mut farthest_branch = VramOffset::new(0);
    let mut halt_function_searching;

    let mut local_offset = 0;
    let mut current_function_start = local_offset;
    let mut current_function_sym = owned_segment.find_symbol(
        section_base.vram_offset(local_offset),
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
                section_base.vram_offset(local_offset),
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
                .find_owned_segment(section_base.parent_segment_info())?
                .find_symbol(
                    section_base.vram_offset(local_offset),
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
                    .find_owned_segment(section_base.parent_segment_info())?
                    .find_symbol(
                        section_base.vram_offset(local_offset),
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
                    .find_owned_segment_mut(section_base.parent_segment_info())?
                    .add_function(
                        section_base.vram_offset(local_offset),
                        section_base.rom_offset(local_offset),
                        GeneratedBy::Autogenerated,
                    );
                aux_sym.set_autocreated_from_other_sized_sym();
                // TODO: figure out a way to avoid having to search the symbol we just createdm, hopefully by reusing the above `aux_sym`.
                current_function_sym = context
                    .find_owned_segment(section_base.parent_segment_info())?
                    .find_symbol(
                        section_base.vram_offset(local_offset),
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
                section_base,
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
            section_base,
            local_offset,
            instr,
            index,
            instrs,
            section_base.rom_offset(local_offset).unwrap(), // TODO: avoid unwrap
            section_base.vram_offset(local_offset),
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

fn find_functions_branch_checker(
    context: &Context,
    section_base: &SectionBase,
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
                .find_owned_segment(section_base.parent_segment_info())?
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
        if branch_offset < VramOffset::new(0) {
            // TODO: branch_offset.is_negative()
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
                let owned_segment =
                    context.find_owned_segment(section_base.parent_segment_info())?;
                let mut j = starts_data.len() as i32 - 1;
                while j >= 0 {
                    let other_func_start_offset = starts_data[j as usize].0 * 4;
                    if branch_offset.inner() + (local_offset as i32)
                        < (other_func_start_offset as i32)
                    {
                        let vram = section_base.vram_offset(local_offset);

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

fn find_functions_check_function_ended(
    context: &Context,
    settings: &SectionTextSettings,
    section_base: &SectionBase,
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

    let owned_segment = context.find_owned_segment(section_base.parent_segment_info())?;
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

    if !(farthest_branch.inner() > 0) && instr.opcode().is_jump() {
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
