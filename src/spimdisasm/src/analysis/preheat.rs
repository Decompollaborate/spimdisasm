/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use rabbitizer::{access_type::AccessType, Instruction};

use crate::{
    addresses::{Rom, Size, Vram, VramOffset},
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
    config::GlobalConfig,
    metadata::{SegmentMetadata, SymbolType},
    section_type::SectionType,
    sections::{SectionDataSettings, SectionExecutableSettings},
};

use super::{ReferenceWrapper, ReferencedAddress, RegisterTracker};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct Preheater {
    references: AddendedOrderedMap<Vram, ReferencedAddress>,
}

impl Preheater {
    pub const fn new() -> Self {
        Self {
            references: AddendedOrderedMap::new(),
        }
    }

    pub(crate) fn references(&self) -> &AddendedOrderedMap<Vram, ReferencedAddress> {
        &self.references
    }

    pub fn preheat_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionExecutableSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        let mut current_rom = rom;
        let mut current_vram = vram;
        let mut prev_instr: Option<Instruction> = None;
        let mut regs_tracker = RegisterTracker::new();

        for b in raw_bytes.chunks_exact(4) {
            let word = global_config.endian().word_from_bytes(b);
            let instr = Instruction::new(word, current_vram, settings.instruction_flags());

            if !instr.is_valid() {
                prev_instr = None;
                current_rom += Size::new(4);
                current_vram += Size::new(4);
                continue;
            }

            if prev_instr.is_some_and(|x| x.opcode().is_branch_likely()) {
                // We only do a single lineal analysis, no control flow at all,
                // so if we find a branch likely we skip it to avoid carrying garbage info.
                prev_instr = Some(instr);
                current_rom += Size::new(4);
                current_vram += Size::new(4);
                continue;
            }

            if let Some(_target_vram) = instr.get_branch_vram_generic() {
                // instr.opcode().is_branch() or instr.is_unconditional_branch()
                regs_tracker.process_branch(&instr, current_rom);
            } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
                // instr.opcode().is_jump_with_address()
                let reference = self.new_ref(target_vram, Some(current_vram), owned_segment);

                reference.set_sym_type(SymbolType::Function);
            } else if instr.is_jumptable_jump() {
                //self.process_jumptable_jump(context, regs_tracker, instr, instr_rom);
                if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(&instr) {
                    let address = Vram::new(jr_reg_data.address());

                    if jr_reg_data.branch_info().is_none() {
                        let reference = self.new_ref(address, None, owned_segment);

                        reference.set_sym_type(SymbolType::Jumptable);
                    }
                }
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
                regs_tracker.process_hi(&instr, current_rom);
            } else if instr.opcode().is_unsigned() {
                // TODO
            } else if instr.opcode().can_be_lo() {
                if let Some(pairing_info) =
                    regs_tracker.preprocess_lo_and_get_info(&instr, current_rom)
                {
                    if pairing_info.is_gp_got {
                        // TODO
                    } else if let Some(lower_half) = instr.get_processed_immediate() {
                        let access_type = instr.opcode().access_type();
                        let address =
                            Vram::new(pairing_info.value as u32) + VramOffset::new(lower_half);

                        let realigned_symbol_vram = match access_type {
                            // Align down the Vram
                            Some(AccessType::WORD_LEFT | AccessType::WORD_RIGHT) => {
                                Vram::new(address.inner() - (address.inner() % 4))
                            }
                            Some(AccessType::DOUBLEWORD_LEFT | AccessType::DOUBLEWORD_RIGHT) => {
                                Vram::new(address.inner() - (address.inner() % 8))
                            }
                            None | Some(_) => address,
                        };

                        let reference =
                            self.new_ref(realigned_symbol_vram, Some(current_vram), owned_segment);

                        if let Some(access_type) = instr.opcode().access_type() {
                            reference.set_access_type(access_type);
                        }

                        regs_tracker.process_lo(&instr, address.inner(), current_rom);
                    }
                }
                if let Some(address) =
                    regs_tracker.get_address_if_instr_can_set_type(&instr, current_rom)
                {
                    if let Some(access_type) = instr.opcode().access_type() {
                        let realigned_symbol_vram = match access_type {
                            // Align down the Vram
                            AccessType::WORD_LEFT | AccessType::WORD_RIGHT => {
                                Vram::new(address - (address % 4))
                            }
                            AccessType::DOUBLEWORD_LEFT | AccessType::DOUBLEWORD_RIGHT => {
                                Vram::new(address - (address % 8))
                            }
                            _ => Vram::new(address),
                        };

                        let reference = self.new_ref(realigned_symbol_vram, None, owned_segment);

                        reference.set_access_type(access_type);
                    }
                }
            }

            regs_tracker.overwrite_registers(&instr, current_rom);

            if let Some(prev) = &prev_instr {
                if prev.is_function_call() {
                    regs_tracker.unset_registers_after_func_call(prev);
                } else if (prev.opcode().is_jump() && !prev.opcode().does_link())
                    || prev.is_unconditional_branch()
                {
                    regs_tracker.clear();
                }
            }

            prev_instr = Some(instr);
            current_rom += Size::new(4);
            current_vram += Size::new(4);
        }
    }

    pub fn preheat_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        self.common_data_preheat(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            owned_segment,
            SectionType::Data,
        );
    }

    pub fn preheat_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        self.common_data_preheat(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            owned_segment,
            SectionType::Rodata,
        );
    }

    pub fn preheat_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        _settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        owned_segment: &SegmentMetadata,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        let mut current_vram = vram;
        let mut references_found = Vec::new();

        for word_bytes in raw_bytes.chunks_exact(4) {
            let word = global_config.endian().word_from_bytes(word_bytes);
            let word_vram = Vram::new(word);

            if owned_segment.in_vram_range(word_vram) {
                references_found.push((
                    word_vram,
                    Some(SymbolType::GccExceptTableLabel),
                    current_vram,
                ));
            }

            current_vram += Size::new(4);
        }

        for (v, typ, referenced_by) in references_found {
            let reference = self.new_ref(v, Some(referenced_by), owned_segment);
            if let Some(typ) = typ {
                reference.set_sym_type(typ);
            }
        }
    }

    // TODO
    #[allow(clippy::too_many_arguments)]
    fn common_data_preheat(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        owned_segment: &SegmentMetadata,
        section_type: SectionType,
    ) {
        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return;
        }

        // Ensure there's a symbol at the start of the segment
        self.new_ref(vram, None, owned_segment);

        let mut remaining_string_size = 0;

        let mut prev_sym_type: Option<SymbolType> = None;
        // If true: the previous symbol made us thought we may be in late_rodata
        let mut maybe_reached_late_rodata = false;
        // If true, we are sure we are in late_rodata
        let mut reached_late_rodata = false;

        let mut float_counter = 0;
        let mut float_padding_counter = 0;

        // TODO
        #[allow(clippy::type_complexity)]
        let mut references_found: Vec<(
            Vram,
            Option<SymbolType>,
            Option<Vram>,
            Option<Size>,
        )> = Vec::new();

        for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
            let local_offset = i * 4;

            let current_vram = vram + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);

            if remaining_string_size <= 0 {
                let current_ref = ReferenceWrapper::find(
                    owned_segment,
                    self,
                    current_vram,
                    FindSettings::new(true),
                );

                if current_ref.is_none_or(|x| x.vram() == current_vram) {
                    if let Some(str_size) = settings.string_guesser_level().guess(
                        current_ref,
                        current_vram,
                        &raw_bytes[local_offset..],
                        settings.encoding(),
                        maybe_reached_late_rodata || reached_late_rodata,
                    ) {
                        let str_sym_size = str_size.next_multiple_of(4);
                        let in_between_sym = ReferenceWrapper::find(
                            owned_segment,
                            self,
                            current_vram + Size::new(str_sym_size as u32 - 1),
                            FindSettings::new(true).with_reject_sizeless_addended(false),
                        );

                        if in_between_sym.is_none_or(|x| {
                            let other_sym_vram = x.vram();

                            match other_sym_vram.cmp(&current_vram) {
                                core::cmp::Ordering::Greater => false,
                                core::cmp::Ordering::Equal => true,
                                core::cmp::Ordering::Less => {
                                    if x.size().is_some_and(|x| other_sym_vram + x <= current_vram)
                                    {
                                        true
                                    } else {
                                        // Hack to try to find unreferenced strings.
                                        // We need this hack because size information for previous symbols on this section
                                        // is not known yet, because we add it lazily.
                                        // Not doing it lazily yields some weird hallucinated symbols. Maybe someday I'll
                                        // properly debug why they happen and how to avoid them, in the meantime we have
                                        // this hack.
                                        references_found.last().is_some_and(|x| {
                                            x.0 == other_sym_vram
                                                && x.3.is_some_and(|size| {
                                                    other_sym_vram + size <= current_vram
                                                })
                                        })
                                    }
                                }
                            }
                        }) {
                            remaining_string_size = str_size as i32;

                            references_found.push((
                                current_vram,
                                Some(SymbolType::CString),
                                None,
                                Some(Size::new(str_sym_size as u32)),
                            ));

                            // Next symbol should not be affected by this string.
                            prev_sym_type = None;
                        }
                    }
                }
            }

            if remaining_string_size <= 0 {
                let mut table_label = None;

                let a = ReferenceWrapper::find(
                    owned_segment,
                    self,
                    current_vram,
                    FindSettings::new(false),
                );
                let b =
                    ReferenceWrapper::find(owned_segment, self, b_vram, FindSettings::new(false));
                let c =
                    ReferenceWrapper::find(owned_segment, self, c_vram, FindSettings::new(false));
                let d =
                    ReferenceWrapper::find(owned_segment, self, d_vram, FindSettings::new(false));

                let a_type = (a.is_some(), a.and_then(|x| x.sym_type()));
                let b_type = (b.is_some(), b.and_then(|x| x.sym_type()));
                let c_type = (c.is_some(), c.and_then(|x| x.sym_type()));
                let d_type = (d.is_some(), d.and_then(|x| x.sym_type()));

                if b.is_none() && c.is_none() && d.is_none() {
                    // There's no symbol in between

                    let current_type = match a {
                        None => prev_sym_type,
                        Some(wrapper) => wrapper.sym_type(),
                    };
                    let should_search_for_address =
                        current_type.is_none_or(|x| x.can_reference_symbols());

                    let endian = global_config.endian();
                    let word = endian.word_from_bytes(word_bytes);
                    if should_search_for_address {
                        let word_vram = Vram::new(word);

                        if owned_segment.in_vram_range(word_vram) {
                            references_found.push((word_vram, None, Some(current_vram), None));

                            if current_type.is_some_and(|x| x.is_table()) {
                                table_label = Some(word_vram);
                            }
                        }
                    }

                    if maybe_reached_late_rodata
                        && matches!(
                            current_type,
                            Some(SymbolType::Float32 | SymbolType::Float64)
                        )
                        && a.is_some()
                    {
                        reached_late_rodata = true;
                    }

                    if let Some(a) = a {
                        if matches!(
                            a.sym_type(),
                            Some(SymbolType::Float32 | SymbolType::Float64)
                        ) {
                            float_counter = 1;
                            float_padding_counter = 0;
                        } else {
                            float_counter = 0;
                            float_padding_counter = 0;
                        }
                    } else if current_type == Some(SymbolType::Float32) {
                        float_counter += 1;
                        if word == 0 {
                            float_padding_counter += 1;
                        }
                    } else if current_type == Some(SymbolType::Float64) {
                        if current_vram.inner() % 8 == 0 {
                            if local_offset + 8 <= raw_bytes.len() {
                                float_counter += 1;
                                if endian
                                    .dword_from_bytes(&raw_bytes[local_offset..local_offset + 8])
                                    == 0
                                {
                                    float_padding_counter += 1;
                                }
                            } else {
                                float_counter = 0;
                                float_padding_counter = 0;
                            }
                        }
                    } else {
                        float_counter = 0;
                        float_padding_counter = 0;
                    }
                }

                for (exists, sym_type) in [a_type, b_type, c_type, d_type].into_iter() {
                    if exists {
                        prev_sym_type = sym_type;
                    }
                }

                if let Some(table_label) = table_label {
                    if let Some(current_reference_mut) = self.references.find_mut(
                        &current_vram,
                        FindSettings::new(true).with_reject_sizeless_addended(false),
                    ) {
                        current_reference_mut.add_table_label(table_label);
                    }
                }
            }

            maybe_reached_late_rodata = false;
            if !reached_late_rodata
                && section_type == SectionType::Rodata
                && prev_sym_type.is_some_and(|x| x.is_late_rodata(settings.compiler()))
            {
                if prev_sym_type == Some(SymbolType::Jumptable) {
                    reached_late_rodata = true;
                } else if float_padding_counter + 1 == float_counter {
                    // Finding a float or a double is not proof enough to say we are in late_rodata, because we
                    // can be after a const array of floats/doubles.
                    // An example of this is the libultra file `xldtob`.
                    // It is okay for late rodata floats to have padding, but if a float has non-zero padding
                    // it means it isn't a late_rodata float.
                    maybe_reached_late_rodata = true;
                }
            }
            remaining_string_size -= 4;
        }

        for (v, typ, referenced_by, size) in references_found {
            let reference = self.new_ref(v, referenced_by, owned_segment);
            if let Some(typ) = typ {
                reference.set_sym_type(typ);
            }
            if let Some(size) = size {
                reference.set_autodetected_size(size);
            }
        }
    }

    fn new_ref(
        &mut self,
        vram: Vram,
        referenced_by: Option<Vram>,
        owned_segment: &SegmentMetadata,
    ) -> &mut ReferencedAddress {
        let settings = FindSettings::new(true);

        let refer = self.references.find_mut_or_insert_with(vram, settings, || {
            if let Some(metadata) = owned_segment.find_symbol(vram, settings) {
                let vram = metadata.vram();
                let mut refer = ReferencedAddress::new_user_declared(vram);

                if let Some(typ) = metadata.user_declared_type() {
                    refer.set_user_declared_type(typ);
                }
                if let Some(size) = metadata.user_declared_size() {
                    refer.set_user_declared_size(size);
                }

                (vram, refer)
            } else {
                (vram, ReferencedAddress::new(vram))
            }
        });

        if let Some(referenced_by) = referenced_by {
            refer.add_referenced_by(referenced_by);
        }

        refer
    }
}

impl Default for Preheater {
    fn default() -> Self {
        Self::new()
    }
}
