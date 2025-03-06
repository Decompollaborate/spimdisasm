/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use rabbitizer::{access_type::AccessType, Instruction};

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram, VramOffset},
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
    config::GlobalConfig,
    metadata::{IgnoredAddressRange, LabelMetadata, LabelType, SymbolMetadata, SymbolType},
    section_type::SectionType,
    sections::before_proc::{DataSectionSettings, ExecutableSectionSettings},
};

use super::{PreheatError, ReferenceWrapper, ReferencedAddress, ReferencedLabel, RegisterTracker};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub(crate) struct Preheater {
    segment_name: Option<Arc<str>>,
    ranges: RomVramRange,
    references: AddendedOrderedMap<Vram, ReferencedAddress>,
    label_references: BTreeMap<Vram, ReferencedLabel>,
    preheated_sections: AddendedOrderedMap<Rom, Size>,
}

impl Preheater {
    pub(crate) const fn new(segment_name: Option<Arc<str>>, ranges: RomVramRange) -> Self {
        Self {
            segment_name,
            ranges,
            references: AddendedOrderedMap::new(),
            label_references: BTreeMap::new(),
            preheated_sections: AddendedOrderedMap::new(),
        }
    }

    pub(crate) fn references(&self) -> &AddendedOrderedMap<Vram, ReferencedAddress> {
        &self.references
    }
    pub(crate) fn references_mut(&mut self) -> &mut AddendedOrderedMap<Vram, ReferencedAddress> {
        &mut self.references
    }
    pub(crate) fn preheated_sections(&self) -> &AddendedOrderedMap<Rom, Size> {
        &self.preheated_sections
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn preheat_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &ExecutableSectionSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Result<(), PreheatError> {
        self.check_failable_preconditions(raw_bytes, rom, vram)?;

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

            if let Some(target_vram) = instr.get_branch_vram_generic() {
                // instr.opcode().is_branch() or instr.is_unconditional_branch()

                regs_tracker.process_branch(&instr, current_rom);
                self.new_label_ref(target_vram, LabelType::Branch, current_vram, user_labels);
            } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
                // instr.opcode().is_jump_with_address()
                if let Some(reference) = self.new_ref(
                    target_vram,
                    Some(current_vram),
                    user_symbols,
                    ignored_addresses,
                ) {
                    reference.set_sym_type(SymbolType::Function);
                }
            } else if instr.is_jumptable_jump() {
                //self.process_jumptable_jump(context, regs_tracker, instr, instr_rom);
                if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(&instr) {
                    let address = Vram::new(jr_reg_data.address());

                    if jr_reg_data.branch_info().is_none() {
                        if let Some(reference) =
                            self.new_ref(address, None, user_symbols, ignored_addresses)
                        {
                            reference.set_sym_type(SymbolType::Jumptable);
                        }
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

                    let reference = self.new_ref(address, None, user_symbols);
                    reference.set_sym_type(SymbolType::Function);
                }
                */
            } else if instr.opcode().can_be_hi() {
                regs_tracker.process_hi(&instr, current_rom);
            } else if instr.opcode().can_be_lo() {
                if let Some(pairing_info) =
                    regs_tracker.preprocess_lo_and_get_info(&instr, current_rom)
                {
                    if let Some(lower_half) = instr.get_processed_immediate() {
                        let address = if pairing_info.is_gp_got {
                            // TODO
                            // global_config.gp_config().is_some_and(|x| x.pic())
                            None
                        } else if pairing_info.is_gp_rel {
                            // TODO: should check for global_config.gp_config().is_some_and(|x| !x.pic())?
                            global_config.gp_config().map(|gp_config| {
                                Vram::new(
                                    gp_config.gp_value().inner().wrapping_add_signed(lower_half),
                                )
                            })
                        } else {
                            Some(Vram::new(pairing_info.value as u32) + VramOffset::new(lower_half))
                        };

                        if let Some(address) = address {
                            let access_type = instr.opcode().access_type();

                            let realigned_symbol_vram = match access_type {
                                // Align down the Vram
                                Some(AccessType::UNALIGNED_WORD) => {
                                    Vram::new(address.inner() - (address.inner() % 4))
                                }
                                Some(AccessType::UNALIGNED_DOUBLEWORD) => {
                                    Vram::new(address.inner() - (address.inner() % 8))
                                }
                                None | Some(_) => address,
                            };

                            if let (Some(reference), Some(access_type)) = (
                                self.new_ref(
                                    realigned_symbol_vram,
                                    Some(current_vram),
                                    user_symbols,
                                    ignored_addresses,
                                ),
                                instr.opcode().access_type(),
                            ) {
                                reference.set_access_type(access_type);
                            }

                            regs_tracker.process_lo(&instr, address.inner(), current_rom);
                        }
                    }
                }
            }
            if let Some(address) =
                regs_tracker.get_address_if_instr_can_set_type(&instr, current_rom)
            {
                if let Some(access_type) = instr.opcode().access_type() {
                    let realigned_symbol_vram = match access_type {
                        // Align down the Vram
                        AccessType::UNALIGNED_WORD => Vram::new(address - (address % 4)),
                        AccessType::UNALIGNED_DOUBLEWORD => Vram::new(address - (address % 8)),
                        _ => Vram::new(address),
                    };

                    if let Some(reference) =
                        self.new_ref(realigned_symbol_vram, None, user_symbols, ignored_addresses)
                    {
                        reference.set_access_type(access_type);
                    }
                }
            } else if instr.opcode().can_be_unsigned_lo() {
                // TODO
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

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn preheat_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Result<(), PreheatError> {
        self.common_data_preheat(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            user_symbols,
            user_labels,
            SectionType::Data,
            ignored_addresses,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn preheat_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Result<(), PreheatError> {
        self.common_data_preheat(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            user_symbols,
            user_labels,
            SectionType::Rodata,
            ignored_addresses,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn preheat_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        _settings: &DataSectionSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Result<(), PreheatError> {
        self.check_failable_preconditions(raw_bytes, rom, vram)?;

        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return Ok(());
        }

        // Make sure there's a table at the start of the section
        if let Some(table) = self.new_ref_no_addend(vram, None, user_symbols, ignored_addresses) {
            table.set_sym_type(SymbolType::GccExceptTable);
        }

        let mut current_vram = vram;

        for word_bytes in raw_bytes.chunks_exact(4) {
            let word = global_config.endian().word_from_bytes(word_bytes);
            let word_vram = Vram::new(word);

            if ignored_addresses
                .find(&word_vram, FindSettings::new(true))
                .is_none()
                && self.ranges.in_vram_range(word_vram)
            {
                self.new_label_ref(
                    word_vram,
                    LabelType::GccExceptTable,
                    current_vram,
                    user_labels,
                );
            }

            current_vram += Size::new(4);
        }

        Ok(())
    }

    // TODO
    #[allow(clippy::too_many_arguments)]
    fn common_data_preheat(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        section_type: SectionType,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Result<(), PreheatError> {
        self.check_failable_preconditions(raw_bytes, rom, vram)?;

        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return Ok(());
        }

        // Ensure there's a symbol at the start of the segment
        self.new_ref(vram, None, user_symbols, ignored_addresses);

        let mut remaining_string_size = 0;

        let mut prev_sym_info: Option<(Vram, Option<SymbolType>, Option<Size>)> = None;
        // If true: the previous symbol made us thought we may be in late_rodata
        let mut maybe_reached_late_rodata = false;
        // If true, we are sure we are in late_rodata
        let mut reached_late_rodata = false;

        let mut float_counter = 0;
        let mut float_padding_counter = 0;

        let mut first_table_label: Option<u32> = None;
        let mut new_ref_scheduled_due_to_jtbl_ended = false;

        let endian = global_config.endian();

        for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
            let local_offset = i * 4;

            let current_vram = vram + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);

            let prev_sym_ended_here = if prev_sym_info
                .is_some_and(|(v, _, s)| s.is_some_and(|s| current_vram >= v + s))
            {
                // If symbol has a given size then get rid of the info as soon as we pass the end of it.
                prev_sym_info = None;
                true
            } else {
                false
            };

            if remaining_string_size <= 0 {
                let mut table_label = None;

                let word = endian.word_from_bytes(word_bytes);

                if new_ref_scheduled_due_to_jtbl_ended && word != 0 {
                    self.new_ref(current_vram, None, user_symbols, ignored_addresses);
                    new_ref_scheduled_due_to_jtbl_ended = false;
                }

                let a = ReferenceWrapper::find(
                    user_symbols,
                    self,
                    current_vram,
                    FindSettings::new(false),
                );
                let b =
                    ReferenceWrapper::find(user_symbols, self, b_vram, FindSettings::new(false));
                let c =
                    ReferenceWrapper::find(user_symbols, self, c_vram, FindSettings::new(false));
                let d =
                    ReferenceWrapper::find(user_symbols, self, d_vram, FindSettings::new(false));

                let a_type = (
                    a.is_some(),
                    current_vram,
                    a.and_then(|x| x.sym_type()),
                    a.and_then(|x| x.user_declared_size()),
                );
                let b_type = (
                    b.is_some(),
                    b_vram,
                    b.and_then(|x| x.sym_type()),
                    b.and_then(|x| x.user_declared_size()),
                );
                let c_type = (
                    c.is_some(),
                    c_vram,
                    c.and_then(|x| x.sym_type()),
                    c.and_then(|x| x.user_declared_size()),
                );
                let d_type = (
                    d.is_some(),
                    d_vram,
                    d.and_then(|x| x.sym_type()),
                    d.and_then(|x| x.user_declared_size()),
                );

                if b.is_none() && c.is_none() && d.is_none() {
                    // There's no symbol in between

                    let current_type = match a {
                        None => prev_sym_info.and_then(|x| x.1),
                        Some(wrapper) => wrapper.sym_type(),
                    };

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

                    let should_search_for_address =
                        current_type.is_none_or(|x| x.can_reference_symbols());

                    let mut reference_found = false;
                    let mut reference_is_in_function = false;
                    if should_search_for_address {
                        let word_vram = Vram::new(word);
                        let is_table = current_type.is_some_and(|x| x.is_table());

                        let in_range = self.ranges.in_vram_range(word_vram);
                        if in_range {
                            let new_ref_info = if is_table {
                                let new_ref = self.new_label_ref(
                                    word_vram,
                                    SymbolType::label_for_table(current_type)
                                        .expect("Already checked this is a table type"),
                                    current_vram,
                                    user_labels,
                                );
                                Some((new_ref.vram(), false))
                            } else {
                                let new_ref = self.new_ref(
                                    word_vram,
                                    Some(current_vram),
                                    user_symbols,
                                    ignored_addresses,
                                );

                                new_ref
                                    .map(|x| (x.vram(), x.sym_type() == Some(SymbolType::Function)))
                            };

                            if let Some((new_ref_vram, is_function)) = new_ref_info {
                                new_ref_scheduled_due_to_jtbl_ended = false;
                                reference_found = true;
                                if new_ref_vram != word_vram && is_function {
                                    reference_is_in_function = !is_table;
                                }
                            }
                        }

                        if is_table {
                            let still_valid_table = if let Some(first) = first_table_label {
                                let mask = 0xFF800000;
                                if word == 0 || ((first & mask) != (word & mask)) || !in_range {
                                    // We are past the end of the jumptable, so we trash `prev_sym_info` to avoid
                                    // seeing the rest of the symbol as a jumptable

                                    // If the word is zero then do not add this address as a reference immediately,
                                    // so we can keep this as trailing padding into this symbol
                                    new_ref_scheduled_due_to_jtbl_ended = word == 0;

                                    if word != 0 {
                                        self.new_ref(
                                            current_vram,
                                            prev_sym_info.map(|x| x.0),
                                            user_symbols,
                                            ignored_addresses,
                                        );
                                    }

                                    if let Some((jtbl_vram, _, _)) = prev_sym_info {
                                        if let Some(jtbl_ref) = self.new_ref(
                                            jtbl_vram,
                                            None,
                                            user_symbols,
                                            ignored_addresses,
                                        ) {
                                            jtbl_ref.set_autodetected_size(
                                                (current_vram - jtbl_vram).try_into().unwrap(),
                                            );
                                        }
                                    }

                                    table_label = None;
                                    first_table_label = None;
                                    prev_sym_info = None;
                                    false
                                } else {
                                    true
                                }
                            } else {
                                first_table_label = Some(word);
                                true
                            };

                            if still_valid_table {
                                table_label = Some(word_vram);
                            }
                        }
                    }

                    if ignored_addresses
                        .find(&current_vram, FindSettings::new(true))
                        .is_none()
                        && (!reference_found || (reference_is_in_function && table_label.is_none()))
                    {
                        let current_ref = ReferenceWrapper::find(
                            user_symbols,
                            self,
                            current_vram,
                            FindSettings::new(true),
                        );

                        if current_ref.is_none_or(|x| x.vram() == current_vram) {
                            let guessed_size = settings.string_guesser_flags().guess(
                                current_ref,
                                current_vram,
                                &raw_bytes[local_offset..],
                                settings.encoding(),
                                settings.compiler(),
                                maybe_reached_late_rodata || reached_late_rodata,
                                prev_sym_ended_here,
                            );

                            match guessed_size {
                                Ok(str_size) => {
                                    let str_sym_size = str_size.next_multiple_of(4);
                                    let mut in_between_range = ReferenceWrapper::range(
                                        user_symbols,
                                        self,
                                        AddressRange::new(
                                            current_vram + Size::new(1),
                                            current_vram + Size::new(str_sym_size as u32),
                                        ),
                                    );

                                    if in_between_range.next().is_none() {
                                        // Check if there is already another symbol after the current one and before the end of the string,
                                        // in which case we say this symbol should not be a string

                                        remaining_string_size = str_size as i32;

                                        if let Some(reference) = self.new_ref(
                                            current_vram,
                                            None,
                                            user_symbols,
                                            ignored_addresses,
                                        ) {
                                            reference.set_sym_type(SymbolType::CString);
                                            reference.set_autodetected_size(Size::new(
                                                str_sym_size as u32,
                                            ));
                                            new_ref_scheduled_due_to_jtbl_ended = false;
                                        }
                                        // Do not create a symbol at `current_vram + Size::new(str_sym_size as u32)` here,
                                        // because it can mess the logic to merge trailing padding due to next's symbol alignment
                                        // that is done in DataSection

                                        // Next symbol should not be affected by this string.
                                        prev_sym_info = None;
                                    }
                                }

                                Err(_e) => {
                                    // For debugging
                                }
                            }
                        }
                    }
                }

                for (exists, sym_vram, sym_type, sym_size) in
                    [a_type, b_type, c_type, d_type].into_iter()
                {
                    if exists {
                        prev_sym_info = Some((sym_vram, sym_type, sym_size));
                    }
                }

                if let (Some((table_vram, _, _)), Some(table_label)) = (prev_sym_info, table_label)
                {
                    if let Some(current_reference_mut) = self
                        .references
                        .find_mut(&table_vram, FindSettings::new(false))
                    {
                        current_reference_mut.add_table_label(table_label);
                    }
                }
            }

            maybe_reached_late_rodata = false;
            if !reached_late_rodata
                && section_type == SectionType::Rodata
                && prev_sym_info
                    .is_some_and(|x| x.1.is_some_and(|x| x.is_late_rodata(settings.compiler())))
            {
                if prev_sym_info.is_some_and(|x| x.1 == Some(SymbolType::Jumptable)) {
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

        Ok(())
    }

    fn new_ref(
        &mut self,
        vram: Vram,
        referenced_by: Option<Vram>,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Option<&mut ReferencedAddress> {
        if ignored_addresses
            .find(&vram, FindSettings::new(true))
            .is_some()
        {
            None
        } else {
            let settings = FindSettings::new(true);

            Some(self.new_ref_impl(vram, referenced_by, user_symbols, settings))
        }
    }

    fn new_ref_no_addend(
        &mut self,
        vram: Vram,
        referenced_by: Option<Vram>,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Option<&mut ReferencedAddress> {
        if ignored_addresses
            .find(&vram, FindSettings::new(true))
            .is_some()
        {
            None
        } else {
            let settings = FindSettings::new(false);

            Some(self.new_ref_impl(vram, referenced_by, user_symbols, settings))
        }
    }

    fn new_ref_impl(
        &mut self,
        vram: Vram,
        referenced_by: Option<Vram>,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        settings: FindSettings,
    ) -> &mut ReferencedAddress {
        let (refer, _) = self
            .references
            .find_mut_or_insert_with_key_value(vram, settings, || {
                if let Some(metadata) = user_symbols.find(&vram, settings) {
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

    fn new_label_ref(
        &mut self,
        vram: Vram,
        label_type: LabelType,
        referenced_by: Vram,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
    ) -> &mut ReferencedLabel {
        let refer = self.label_references.entry(vram).or_insert_with(|| {
            if let Some(metadata) = user_labels.get(&vram) {
                ReferencedLabel::new_user_declared(vram, metadata.label_type())
            } else {
                ReferencedLabel::new(vram, label_type)
            }
        });

        refer.add_referenced_by(referenced_by);
        refer.set_autodetected_type(label_type);

        refer
    }

    fn check_failable_preconditions(
        &mut self,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError> {
        let size = Size::new(raw_bytes.len() as u32);
        let rom_end = rom + size;
        let vram_end = vram + size;
        let segment_rom_range = self.ranges.rom();
        let segment_vram_range = self.ranges.vram();

        if !segment_rom_range.in_range(rom) || !segment_rom_range.in_range_inclusive_end(rom_end) {
            Err(PreheatError::new_wrong_rom(
                self.segment_name.clone(),
                rom,
                vram,
                *segment_rom_range,
                rom_end,
            ))
        } else if !segment_vram_range.in_range(vram)
            || !segment_vram_range.in_range_inclusive_end(vram_end)
        {
            Err(PreheatError::new_wrong_vram(
                self.segment_name.clone(),
                rom,
                vram,
                *segment_vram_range,
                vram_end,
            ))
        } else if self
            .preheated_sections
            .find(&rom, FindSettings::new(true))
            .is_some()
        {
            Err(PreheatError::new_already_preheated(
                self.segment_name.clone(),
                rom,
                vram,
            ))
        } else {
            self.preheated_sections
                .find_mut_or_insert_with(rom, FindSettings::new(false), || size);
            Ok(())
        }
    }
}
