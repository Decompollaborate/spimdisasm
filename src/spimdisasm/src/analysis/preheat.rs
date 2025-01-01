/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use rabbitizer::{vram::VramOffset, Instruction, Vram};

use crate::{
    config::GlobalConfig,
    metadata::{SegmentMetadata, SymbolType},
    rom_address::RomAddress,
    sections::{SectionDataSettings, SectionExecutableSettings},
    size::Size,
};

use super::{ReferenceWrapper, ReferencedAddress, RegisterTracker};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preheater {
    references: BTreeMap<Vram, ReferencedAddress>,
}

impl Preheater {
    pub fn new() -> Self {
        Self {
            references: BTreeMap::new(),
        }
    }

    pub(crate) fn references(&self) -> &BTreeMap<Vram, ReferencedAddress> {
        &self.references
    }

    pub fn preheat_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionExecutableSettings,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        _owned_segment: &SegmentMetadata,
    ) {
        let mut current_rom = rom;
        let mut current_vram = vram;
        let mut prev_instr: Option<Instruction> = None;
        let mut regs_tracker = RegisterTracker::new();

        for b in raw_bytes.chunks_exact(4) {
            let word = global_config.endian().word_from_bytes(b);
            let instr = Instruction::new(word, current_vram, settings.instruction_flags());
            current_rom += Size::new(4);
            current_vram += Size::new(4);

            if !instr.is_valid() {
                prev_instr = None;
                continue;
            }

            if let Some(_target_vram) = instr.get_branch_vram_generic() {
                // instr.opcode().is_branch() or instr.is_unconditional_branch()
                regs_tracker.process_branch(&instr, current_rom);
            } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
                // instr.opcode().is_jump_with_address()
                let reference = self.new_ref(target_vram);

                reference.set_sym_type(SymbolType::Function);
                reference.increment_references();
            } else if instr.is_jumptable_jump() {
                //self.process_jumptable_jump(context, regs_tracker, instr, instr_rom);
                if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(&instr) {
                    let address = Vram::new(jr_reg_data.address());

                    if jr_reg_data.branch_info().is_none() {
                        let reference = self.new_ref(address);

                        reference.set_sym_type(SymbolType::Jumptable);
                    }
                }
            } else if instr.opcode().is_jump() && instr.opcode().does_link() {
                // `jalr`. Implicit `!is_jump_with_address`
                if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(&instr) {
                    let address = Vram::new(jr_reg_data.address());

                    let reference = self.new_ref(address);
                    reference.set_sym_type(SymbolType::Function);
                }
            } else if instr.opcode().can_be_hi() {
                regs_tracker.process_hi(&instr, current_rom, prev_instr.as_ref());
            } else if instr.opcode().is_unsigned() {
                // TODO
            } else if instr.opcode().can_be_lo() {
                if let Some(pairing_info) =
                    regs_tracker.preprocess_lo_and_get_info(&instr, current_rom)
                {
                    if pairing_info.is_gp_got {
                        // TODO
                    } else if let Some(lower_half) = instr.get_processed_immediate() {
                        let address =
                            Vram::new(pairing_info.value as u32) + VramOffset::new(lower_half);

                        let reference = self.new_ref(address);

                        let access_type = instr.opcode().access_type();
                        reference.set_size(access_type.min_size());
                        reference.set_alignment(access_type.min_alignment());
                        reference.increment_references();

                        if let Some(sym_type) = SymbolType::from_access_type(access_type) {
                            reference.set_sym_type(sym_type);
                        }

                        regs_tracker.process_lo(&instr, address.inner(), current_rom);
                    }
                }
            }

            if let Some(prev) = &prev_instr {
                if prev.is_function_call() {
                    regs_tracker.unset_registers_after_func_call(&instr, prev);
                } else if prev.is_unconditional_branch()
                    || prev.is_jumptable_jump()
                    || prev.is_return()
                    || prev.opcode().is_branch_likely()
                {
                    regs_tracker.clear();
                }
            }

            prev_instr = Some(instr);
        }
    }

    pub fn preheat_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        self.common_data_preheat(global_config, settings, raw_bytes, rom, vram, owned_segment);
    }

    pub fn preheat_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        self.common_data_preheat(global_config, settings, raw_bytes, rom, vram, owned_segment);
    }

    pub fn preheat_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        _settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        let mut references_found = Vec::new();

        for word_bytes in raw_bytes.chunks_exact(4) {
            let word = global_config.endian().word_from_bytes(word_bytes);
            let word_vram = Vram::new(word);

            if owned_segment.in_vram_range(word_vram) {
                references_found.push((word_vram, Some(SymbolType::GccExceptTableLabel), true));
            }
        }

        for (v, typ, referenced) in references_found {
            let reference = self.new_ref(v);
            if let Some(typ) = typ {
                reference.set_sym_type(typ);
            }
            if referenced {
                reference.increment_references();
            }
        }
    }

    fn common_data_preheat(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        let mut remaining_string_size = 0;

        let mut prev_sym_type: Option<SymbolType> = None;

        let mut references_found = Vec::new();

        for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
            let local_offset = i * 4;

            let current_vram = vram + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);

            let a = ReferenceWrapper::find(owned_segment, self, current_vram);

            if remaining_string_size <= 0 {
                if let Some(str_sym_size) = settings.string_guesser_level().guess(
                    a,
                    current_vram,
                    &raw_bytes[local_offset..],
                    settings.encoding(),
                ) {
                    if ReferenceWrapper::find_with_addend(
                        owned_segment,
                        self,
                        current_vram + Size::new(str_sym_size as u32 - 1),
                    )
                    .is_some_and(|x| x.vram() != current_vram)
                    {
                        remaining_string_size = str_sym_size as i32;

                        references_found.push((current_vram, Some(SymbolType::CString), false));
                    }
                }
            }

            if remaining_string_size <= 0 {
                let b = ReferenceWrapper::find(owned_segment, self, b_vram);
                let c = ReferenceWrapper::find(owned_segment, self, c_vram);
                let d = ReferenceWrapper::find(owned_segment, self, d_vram);

                if b.is_none() && c.is_none() && d.is_none() {
                    // There's no symbol in between

                    let should_search_for_address = match a {
                        None => prev_sym_type,
                        Some(metadata) => metadata.sym_type(),
                    }
                    .is_none_or(|x| x.can_reference_symbols());

                    if should_search_for_address {
                        let word = global_config.endian().word_from_bytes(word_bytes);
                        let word_vram = Vram::new(word);

                        if owned_segment.in_vram_range(word_vram) {
                            references_found.push((word_vram, None, true));
                        }
                    }
                }

                for x in [a, b, c, d].into_iter().flatten() {
                    prev_sym_type = x.sym_type();
                }
            }

            remaining_string_size -= 4;
        }

        for (v, typ, referenced) in references_found {
            let reference = self.new_ref(v);
            if let Some(typ) = typ {
                reference.set_sym_type(typ);
            }
            if referenced {
                reference.increment_references();
            }
        }
    }

    fn new_ref(&mut self, vram: Vram) -> &mut ReferencedAddress {
        self.references
            .entry(vram)
            .or_insert_with(|| ReferencedAddress::new(vram))
    }
}

impl Default for Preheater {
    fn default() -> Self {
        Self::new()
    }
}
