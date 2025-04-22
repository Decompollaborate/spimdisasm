/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc};

use rabbitizer::{access_type::AccessType, registers_meta::Register, Instruction};

use crate::{
    addresses::{AddressRange, GlobalOffsetTable, Rom, RomVramRange, Size, SizedAddress, Vram},
    collections::{
        addended_ordered_map::{AddendedOrderedMap, FindSettings},
        unordered_map::UnorderedMap,
        unordered_set::UnorderedSet,
    },
    config::GlobalConfig,
    metadata::{IgnoredAddressRange, LabelMetadata, LabelType, SymbolMetadata, SymbolType},
    section_type::SectionType,
    sections::before_proc::{DataSectionSettings, ExecutableSectionSettings},
};

use super::{
    InstrOpJumptable, InstrOpLink, InstrOpPairedAddress, InstrOpTailCall, InstructionOperation,
    PreheatError, ReferenceWrapper, ReferencedAddress, ReferencedLabel, RegisterTracker,
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub(crate) struct Preheater {
    segment_name: Option<Arc<str>>,
    ranges: RomVramRange,
    references: AddendedOrderedMap<Vram, ReferencedAddress>,
    label_references: BTreeMap<Vram, ReferencedLabel>,
    preheated_sections_rom: AddendedOrderedMap<Rom, Size>,
    preheated_sections_vram: AddendedOrderedMap<Vram, (Arc<str>, Vram, Size)>,
}

impl Preheater {
    pub(crate) const fn new(segment_name: Option<Arc<str>>, ranges: RomVramRange) -> Self {
        Self {
            segment_name,
            ranges,
            references: AddendedOrderedMap::new(),
            label_references: BTreeMap::new(),
            preheated_sections_rom: AddendedOrderedMap::new(),
            preheated_sections_vram: AddendedOrderedMap::new(),
        }
    }

    pub(crate) fn references(&self) -> &AddendedOrderedMap<Vram, ReferencedAddress> {
        &self.references
    }
    pub(crate) fn references_mut(&mut self) -> &mut AddendedOrderedMap<Vram, ReferencedAddress> {
        &mut self.references
    }
    pub(crate) fn preheated_sections_rom(&self) -> &AddendedOrderedMap<Rom, Size> {
        &self.preheated_sections_rom
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn preheat_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &ExecutableSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> Result<(), PreheatError> {
        self.check_failable_preconditions(name, raw_bytes, rom, vram)?;

        let mut current_rom = rom;
        let mut current_vram = vram;
        let mut prev_instr: Option<Instruction> = None;
        let mut regs_tracker = RegisterTracker::new(
            settings.instruction_flags().abi(),
            Some(vram),
            global_config.gp_config().copied(),
            global_config.endian(),
        );
        let mut function_maybe_ended = false;
        // TODO: A bit of a hack, consider removing
        let mut pic_locals = UnorderedMap::new();
        // This hack exists to try to properly pair the address of a jumptable whose %hi is in the
        // delay slot of the `jr` instruction of another jumptable.
        // This is a pretty uncommon thing to happen, but it annoyed me enough to actually implement
        // this hack.
        let mut jumptable_silly_hack = 0;

        let mut accesses_to_remove = UnorderedSet::new();

        for b in raw_bytes.chunks_exact(4) {
            let word = global_config.endian().word_from_bytes(b);
            let instr = Instruction::new(word, current_vram, settings.instruction_flags());

            if function_maybe_ended && !instr.is_nop() {
                // We restart the register tracker _after_ we end seeing nops, because those nops
                // usually are padding of some kind, and not part of the function.
                // We do this because we want to know the actual address of each function, since
                // that's important for PIC programs.

                regs_tracker.soft_reset(settings.instruction_flags().abi(), Some(instr.vram()));
                function_maybe_ended = false;
            }

            if !instr.is_valid() {
                function_maybe_ended |= regs_tracker
                    .clear_afterwards(prev_instr.as_ref(), Some(current_vram + Size::new(4)));

                prev_instr = None;
                current_rom += Size::new(4);
                current_vram += Size::new(4);
                continue;
            }

            if prev_instr.is_some_and(|x| x.opcode().is_branch_likely()) {
                // We only do a single lineal analysis, no control flow at all,
                // so if we find a branch likely we skip it to avoid carrying garbage info.
                function_maybe_ended |= regs_tracker
                    .clear_afterwards(prev_instr.as_ref(), Some(current_vram + Size::new(4)));

                prev_instr = Some(instr);
                current_rom += Size::new(4);
                current_vram += Size::new(4);
                continue;
            }

            let instr_processed_result =
                regs_tracker.process_instruction(&instr, current_rom, global_offset_table);

            let paired_address = match instr_processed_result {
                InstructionOperation::Link { info } => match info {
                    InstrOpLink::DirectLinkingCall { target_vram } => {
                        if let Some(reference) = self.new_ref(
                            target_vram,
                            Some(current_vram),
                            user_symbols,
                            ignored_addresses,
                        ) {
                            reference.set_sym_type(SymbolType::Function);
                        }
                        None
                    }
                    InstrOpLink::LinkingBranch { target_vram } => {
                        self.new_label_ref(
                            target_vram,
                            LabelType::Branch,
                            current_vram,
                            user_labels,
                        );
                        None
                    }
                    InstrOpLink::RawRegisterLink { vram, .. }
                    | InstrOpLink::Call16RegisterLink { vram, .. }
                    | InstrOpLink::CallHiLoRegisterLink { vram, .. } => {
                        if let Some(reference) =
                            self.new_ref(vram, None, user_symbols, ignored_addresses)
                        {
                            reference.set_sym_type(SymbolType::Function);
                        }
                        None
                    }
                    InstrOpLink::DereferencedRegisterLink { .. }
                    | InstrOpLink::UnknownJumpAndLinkRegister { .. } => None,
                },

                InstructionOperation::TailCall { info } => match info {
                    InstrOpTailCall::MaybeDirectTailCall { target_vram } => {
                        self.new_label_ref(
                            target_vram,
                            LabelType::Branch,
                            current_vram,
                            user_labels,
                        );
                        None
                    }
                    InstrOpTailCall::RawRegisterTailCall { vram, .. } => {
                        if let Some(reference) =
                            self.new_ref(vram, None, user_symbols, ignored_addresses)
                        {
                            reference.set_sym_type(SymbolType::Function);
                        }
                        None
                    }
                    InstrOpTailCall::DereferencedRegisterTailCall { .. }
                    | InstrOpTailCall::UnknownRegisterJump { .. } => None,
                },

                InstructionOperation::JumptableJump {
                    jumptable_vram,
                    dereferenced_rom,
                    info,
                } => {
                    if let Some(reference) =
                        self.new_ref(jumptable_vram, None, user_symbols, ignored_addresses)
                    {
                        reference.set_sym_type(SymbolType::Jumptable);
                    }

                    match info {
                        InstrOpJumptable::Simple => {}
                        InstrOpJumptable::Pic => {
                            if let Some(got_address) = pic_locals.get(&dereferenced_rom) {
                                if let Some(reference) = self.new_ref(
                                    *got_address,
                                    None,
                                    user_symbols,
                                    ignored_addresses,
                                ) {
                                    reference.set_add_gp_to_pointed_data();
                                }
                            }
                        }
                    }

                    jumptable_silly_hack = 2;
                    None
                }

                InstructionOperation::ReturnJump => None,

                InstructionOperation::Branch { target_vram } => {
                    self.new_label_ref(target_vram, LabelType::Branch, current_vram, user_labels);
                    None
                }

                InstructionOperation::Hi { .. } => None,

                InstructionOperation::PairedAddress {
                    unaddended_vram,
                    addended_vram: _,
                    info,
                } => match info {
                    InstrOpPairedAddress::PairedLo { access_info, .. } => {
                        let mut special_case = false;

                        if let Some(lo_rs) = instr.field_rs() {
                            if instr.opcode().reads_rs() && lo_rs.is_global_pointer(instr.abi()) {
                                if let Some(lo_rt) = instr.field_rt() {
                                    if instr.opcode().modifies_rt()
                                        && lo_rt.is_global_pointer(instr.abi())
                                    {
                                        special_case = true;
                                    }
                                }
                            }
                        }

                        if special_case {
                            None
                        } else {
                            Some((unaddended_vram, Some(current_vram), access_info))
                        }
                    }
                    InstrOpPairedAddress::GpRel { access_info, .. } => {
                        Some((unaddended_vram, Some(current_vram), access_info))
                    }
                    InstrOpPairedAddress::GpGotGlobal { .. }
                    | InstrOpPairedAddress::GpGotLazyResolver { .. } => {
                        Some((unaddended_vram, Some(current_vram), None))
                    }
                    InstrOpPairedAddress::GpGotLocal { .. } => None,
                    InstrOpPairedAddress::PairedGpGotLo { access_info, .. } => {
                        pic_locals.insert(current_rom, unaddended_vram);

                        Some((unaddended_vram, Some(current_vram), access_info))
                    }
                    InstrOpPairedAddress::PairedGotLo { .. } => {
                        Some((unaddended_vram, Some(current_vram), None))
                    }
                },

                InstructionOperation::GpSet { .. } => None,
                InstructionOperation::DereferencedRawAddress {
                    original_address,
                    access_info,
                    ..
                } => Some((original_address, None, Some(access_info))),
                InstructionOperation::DanglingLo { .. } => None,
                InstructionOperation::Constant { .. } => None,
                InstructionOperation::UnpairedConstant { .. } => None,
                InstructionOperation::RegisterOperation { .. } => None,
                InstructionOperation::UnhandledOpcode { opcode: _ } => None,
                InstructionOperation::InvalidInstr {} => None,
            };

            if let Some((paired_address, referenced_by, access_info)) = paired_address {
                if let Some(reference) = self.new_ref(
                    paired_address,
                    referenced_by,
                    user_symbols,
                    ignored_addresses,
                ) {
                    if let Some((access_type, y)) = access_info {
                        if let (AccessType::DOUBLEFLOAT, true) = (access_type, y) {
                            // We want to avoid creating a symbol in the middle of the doublefloat.
                            let unaddended_vram = paired_address.align_down(8) + Size::new(0x4);
                            accesses_to_remove.insert(unaddended_vram);
                        }

                        reference.set_access_type(access_type);
                    }
                }
            }

            if jumptable_silly_hack != 1 {
                function_maybe_ended |= regs_tracker
                    .clear_afterwards(prev_instr.as_ref(), Some(current_vram + Size::new(4)));
            }

            prev_instr = Some(instr);
            current_rom += Size::new(4);
            current_vram += Size::new(4);
            jumptable_silly_hack -= 1;
        }

        if !accesses_to_remove.is_empty() {
            self.references
                .retain(|vram, _| !accesses_to_remove.contains(vram));
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn preheat_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
        _global_offset_table: Option<&GlobalOffsetTable>,
    ) -> Result<(), PreheatError> {
        self.common_data_preheat(
            global_config,
            settings,
            name,
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
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
        _global_offset_table: Option<&GlobalOffsetTable>,
    ) -> Result<(), PreheatError> {
        self.common_data_preheat(
            global_config,
            settings,
            name,
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
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
        _global_offset_table: Option<&GlobalOffsetTable>,
    ) -> Result<(), PreheatError> {
        self.check_failable_preconditions(name, raw_bytes, rom, vram)?;

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
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        user_symbols: &AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: &BTreeMap<Vram, LabelMetadata>,
        section_type: SectionType,
        ignored_addresses: &AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Result<(), PreheatError> {
        self.check_failable_preconditions(name, raw_bytes, rom, vram)?;

        if rom.inner() % 4 != 0 || vram.inner() % 4 != 0 {
            // not word-aligned, give up.
            return Ok(());
        }

        // Ensure there's a symbol at the start of the segment
        self.new_ref(vram, None, user_symbols, ignored_addresses);

        let mut remaining_string_size = 0;

        let mut prev_sym_info: Option<(Vram, Option<SymbolType>, Option<Size>, bool)> = None;
        // If true: the previous symbol made us thought we may be in late_rodata
        let mut maybe_reached_late_rodata = false;
        // If true, we are sure we are in late_rodata
        let mut reached_late_rodata = false;

        let mut float_counter = 0;
        let mut float_padding_counter = 0;

        let mut first_table_label: Option<Vram> = None;
        let mut new_ref_scheduled_due_to_jtbl_ended = false;

        let endian = global_config.endian();
        let gp_value = global_config.gp_config().map(|x| x.gp_value());

        for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
            let local_offset = i * 4;

            let current_vram = vram + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);

            let prev_sym_ended_here = if prev_sym_info
                .is_some_and(|(v, _, s, _)| s.is_some_and(|s| current_vram >= v + s))
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
                    a.is_some_and(|x| x.add_gp_to_pointed_data()),
                );
                let b_type = (
                    b.is_some(),
                    b_vram,
                    b.and_then(|x| x.sym_type()),
                    b.and_then(|x| x.user_declared_size()),
                    b.is_some_and(|x| x.add_gp_to_pointed_data()),
                );
                let c_type = (
                    c.is_some(),
                    c_vram,
                    c.and_then(|x| x.sym_type()),
                    c.and_then(|x| x.user_declared_size()),
                    c.is_some_and(|x| x.add_gp_to_pointed_data()),
                );
                let d_type = (
                    d.is_some(),
                    d_vram,
                    d.and_then(|x| x.sym_type()),
                    d.and_then(|x| x.user_declared_size()),
                    d.is_some_and(|x| x.add_gp_to_pointed_data()),
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
                        let add_gp_to_pointed_data = if let Some(a) = a {
                            a.add_gp_to_pointed_data()
                        } else {
                            prev_sym_info.is_some_and(|(_, _, _, add_gp_to_pointed_data)| {
                                add_gp_to_pointed_data
                            })
                        };

                        let word_vram =
                            if let (true, Some(gp_value)) = (add_gp_to_pointed_data, gp_value) {
                                // `as i32` should be doing a two complement conversion.
                                Vram::new(gp_value.inner().wrapping_add_signed(word as i32))
                            } else {
                                Vram::new(word)
                            };
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

                                if word == 0
                                    || ((first.inner() & mask) != (word_vram.inner() & mask))
                                    || !in_range
                                {
                                    // We are past the end of the jumptable, so we trash `prev_sym_info` to avoid
                                    // seeing the rest of the symbol as a jumptable

                                    // If the word is zero then do not add this address as a reference immediately,
                                    // so we can keep this as trailing padding into this symbol
                                    new_ref_scheduled_due_to_jtbl_ended = word == 0;

                                    if !new_ref_scheduled_due_to_jtbl_ended {
                                        self.new_ref(
                                            current_vram,
                                            prev_sym_info.map(|x| x.0),
                                            user_symbols,
                                            ignored_addresses,
                                        );
                                    }

                                    if let Some((jtbl_vram, _, _, _)) = prev_sym_info {
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
                                first_table_label = Some(word_vram);
                                true
                            };

                            if still_valid_table {
                                table_label = Some(word_vram);
                            }
                        }
                    }

                    // Only try to guess if this data is a string if we don't suspect this word may
                    // be an address.
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

                for (exists, sym_vram, sym_type, sym_size, add_gp_to_pointed_data) in
                    [a_type, b_type, c_type, d_type].into_iter()
                {
                    if exists {
                        prev_sym_info =
                            Some((sym_vram, sym_type, sym_size, add_gp_to_pointed_data));
                        new_ref_scheduled_due_to_jtbl_ended = false;
                    }
                }

                if let (Some((table_vram, _, _, _)), Some(table_label)) =
                    (prev_sym_info, table_label)
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
        name: Arc<str>,
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
                name,
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
                name,
                rom,
                vram,
                *segment_vram_range,
                vram_end,
            ))
        } else if self
            .preheated_sections_rom
            .find(&rom, FindSettings::new(true))
            .is_some()
        {
            Err(PreheatError::new_already_preheated(
                self.segment_name.clone(),
                name,
                rom,
                vram,
            ))
        } else if let Some((other_name, other_vram, other_size)) = self
            .preheated_sections_vram
            .find(&vram, FindSettings::new(true))
        {
            Err(PreheatError::new_overlaps_with_already_preheated(
                self.segment_name.clone(),
                name,
                rom,
                vram,
                other_name.clone(),
                *other_vram,
                *other_size,
            ))
        } else {
            self.preheated_sections_rom.find_mut_or_insert_with(
                rom,
                FindSettings::new(false),
                || size,
            );
            self.preheated_sections_vram.find_mut_or_insert_with(
                vram,
                FindSettings::new(false),
                || (name, vram, size),
            );
            Ok(())
        }
    }
}

impl SizedAddress for (Arc<str>, Vram, Size) {
    fn size(&self) -> Size {
        self.2
    }
}
