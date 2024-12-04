/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::{self, BTreeMap};
use alloc::vec::Vec;

#[cfg(not(feature = "nightly"))]
use ::polonius_the_crab::prelude::*;

#[cfg(feature = "nightly")]
use core::ops::Bound;

use rabbitizer::Vram;

use crate::rom_vram_range::RomVramRange;
use crate::size::Size;
use crate::{address_range::AddressRange, rom_address::RomAddress, section_type::SectionType};

use super::{symbol_metadata::GeneratedBy, OverlayCategoryName};
use super::{SymbolMetadata, SymbolType};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct SegmentMetadata {
    ranges: RomVramRange,

    category_name: Option<OverlayCategoryName>,

    symbols: BTreeMap<Vram, SymbolMetadata>,
    // constants: BTreeMap<Vram, SymbolMetadata>,

    //
    /// Stuff that looks like pointers. Found referenced by data.
    new_pointer_in_data: BTreeMap<Vram, Vec<RomAddress>>,
    //
    // is_the_unknown_segment: bool,
}

impl SegmentMetadata {
    pub(crate) const fn new(
        ranges: RomVramRange,
        category_name: Option<OverlayCategoryName>,
    ) -> Self {
        Self {
            ranges,
            category_name,

            symbols: BTreeMap::new(),
            new_pointer_in_data: BTreeMap::new(),
        }
    }

    pub const fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }

    pub const fn rom_range(&self) -> &AddressRange<RomAddress> {
        self.ranges.rom()
    }
    /*
    pub(crate) fn rom_range_mut(&mut self) -> &mut AddressRange<RomAddress> {
        &mut self.rom_range
    }
    */
    pub fn in_rom_range(&self, rom: RomAddress) -> bool {
        self.ranges.rom().in_range(rom)
    }

    pub const fn vram_range(&self) -> &AddressRange<Vram> {
        self.ranges.vram()
    }
    /*
    pub(crate) fn vram_range_mut(&mut self) -> &mut AddressRange<Vram> {
        &mut self.vram_range
    }
    */
    pub fn in_vram_range(&self, vram: Vram) -> bool {
        self.ranges.vram().in_range(vram)
    }

    pub const fn rom_size(&self) -> Size {
        self.ranges.rom().size()
    }

    pub fn vram_size(&self) -> Size {
        self.ranges.vram().size()
    }

    /*
    pub fn vram_from_rom(&self, rom: RomAddress) -> Option<Vram> {
        if let Some(rom_range) = self.rom_range {
            let offset = VramOffset::new((rom.inner() as i32) - (rom_range.start().inner() as i32));

            Some(self.vram_range.start() + offset)
        } else {
            None
        }
    }
    */

    pub const fn category_name(&self) -> Option<&OverlayCategoryName> {
        self.category_name.as_ref()
    }

    pub const fn symbols(&self) -> &BTreeMap<Vram, SymbolMetadata> {
        &self.symbols
    }
}

#[cfg(feature = "nightly")]
fn into_prev_and_next<'a, K, V>(
    mut cursor: btree_map::CursorMut<'a, K, V>,
) -> (Option<(&'a K, &'a mut V)>, Option<(&'a K, &'a mut V)>) {
    let prev: Option<(&'a K, &'a mut V)> = cursor.peek_prev().map(|(k, v)| {
        let ptr_k = k as *const K;
        let ptr_v = v as *mut V;
        unsafe { (&*ptr_k, &mut *ptr_v) }
    });
    let next: Option<(&'a K, &'a mut V)> = cursor.peek_next().map(|(k, v)| {
        let ptr_k = k as *const K;
        let ptr_v = v as *mut V;
        unsafe { (&*ptr_k, &mut *ptr_v) }
    });

    (prev, next)
}

#[cfg(not(feature = "nightly"))]
fn add_symbol_impl(
    mut slf: &mut SegmentMetadata,
    vram: Vram,
    generated_by: GeneratedBy,
    allow_sym_with_addend: bool,
) -> &mut SymbolMetadata {
    // TODO: get rid of the polonius stuff when the new borrow checker has been released.

    polonius!(|slf| -> &'polonius mut SymbolMetadata {
        if let Some(x) = slf.find_symbol_mut(
            vram,
            FindSettings::new().with_allow_addend(allow_sym_with_addend),
        ) {
            polonius_return!(x);
        }
    });

    let entry = slf.symbols.entry(vram);

    entry.or_insert(SymbolMetadata::new(generated_by, vram))
}

#[cfg(feature = "nightly")]
fn add_symbol_impl(
    slf: &mut SegmentMetadata,
    vram: Vram,
    generated_by: GeneratedBy,
    allow_sym_with_addend: bool,
) -> &mut SymbolMetadata {
    let mut cursor = slf.symbols.upper_bound_mut(Bound::Included(&vram));

    let must_insert_new = if let Some((sym_vram, sym)) = cursor.peek_prev() {
        if vram == *sym_vram {
            false
        } else if !allow_sym_with_addend {
            true
        } else {
            vram >= *sym_vram + sym.size()
        }
    } else {
        true
    };

    if must_insert_new {
        let sym = SymbolMetadata::new(generated_by, vram);

        cursor
            .insert_before(vram, sym)
            .expect("This should not be able to panic");
    }

    //let sym = unsafe { &mut *(cursor.peek_prev().unwrap().1 as *mut SymbolMetadata) };
    into_prev_and_next(cursor).0.unwrap().1
}

impl SegmentMetadata {
    pub(crate) fn add_symbol(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
        section_type: Option<SectionType>,
        allow_sym_with_addend: bool, // false
    ) -> &mut SymbolMetadata {
        let sym = add_symbol_impl(self, vram, generated_by, allow_sym_with_addend);
        if let Some(rom) = rom {
            *sym.rom_mut() = Some(rom);
        }
        if let Some(section_type) = section_type {
            *sym.section_type_mut() = Some(section_type);
        }
        sym
    }

    pub(crate) fn add_function(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
    ) -> &mut SymbolMetadata {
        let sym = self.add_symbol(vram, rom, generated_by, Some(SectionType::Text), false);
        sym.set_type(SymbolType::Function, generated_by);
        sym
    }

    pub(crate) fn add_branch_label(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
    ) -> &mut SymbolMetadata {
        let sym = self.add_symbol(vram, rom, generated_by, Some(SectionType::Text), false);
        match sym.sym_type() {
            Some(SymbolType::Function)
            | Some(SymbolType::JumptableLabel)
            | Some(SymbolType::GccExceptTableLabel) => {
                // Functions, jumptable labels and gccexcepttable labels take precedence over plain labels.

                if generated_by == GeneratedBy::Autogenerated {
                    // This type may come from the user, so if we haven't set what type we detected this type to be then set it anyways.
                    // Setting it doesn't really matter since user-declread info takes precendence anyways
                    match sym.autodetected_type() {
                        Some(SymbolType::Function)
                        | Some(SymbolType::JumptableLabel)
                        | Some(SymbolType::GccExceptTableLabel) => {}
                        _ => sym.set_type(SymbolType::BranchLabel, generated_by),
                    }
                }
            }
            _ => sym.set_type(SymbolType::BranchLabel, generated_by),
        }
        sym
    }

    pub(crate) fn add_jumptable(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
    ) -> &mut SymbolMetadata {
        let sym = self.add_symbol(vram, rom, generated_by, Some(SectionType::Rodata), false);
        sym.set_type(SymbolType::Jumptable, generated_by);
        sym
    }

    pub(crate) fn add_jumptable_label(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
    ) -> &mut SymbolMetadata {
        let sym = self.add_symbol(vram, rom, generated_by, Some(SectionType::Text), false);
        match sym.sym_type() {
            Some(SymbolType::Function) | Some(SymbolType::GccExceptTableLabel) => {
                // Functions and gccexcepttable labels take precedence over jumptable labels.

                if generated_by == GeneratedBy::Autogenerated {
                    // This type may come from the user, so if we haven't set what type we detected this type to be then set it anyways.
                    // Setting it doesn't really matter since user-declread info takes precendence anyways
                    match sym.autodetected_type() {
                        Some(SymbolType::Function) | Some(SymbolType::GccExceptTableLabel) => {}
                        _ => sym.set_type(SymbolType::JumptableLabel, generated_by),
                    }
                }
            }
            _ => sym.set_type(SymbolType::JumptableLabel, generated_by),
        }
        sym
    }

    pub(crate) fn add_gcc_except_table(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
    ) -> &mut SymbolMetadata {
        let sym = self.add_symbol(
            vram,
            rom,
            generated_by,
            Some(SectionType::GccExceptTable),
            false,
        );
        sym.set_type(SymbolType::GccExceptTable, generated_by);
        sym
    }

    pub(crate) fn add_gcc_except_table_label(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
        generated_by: GeneratedBy,
    ) -> &mut SymbolMetadata {
        let sym = self.add_symbol(vram, rom, generated_by, Some(SectionType::Text), false);
        match sym.sym_type() {
            Some(SymbolType::Function) => {
                // Functions take precedence over gccexcepttable labels.

                if generated_by == GeneratedBy::Autogenerated {
                    // This type may come from the user, so if we haven't set what type we detected this type to be then set it anyways.
                    // Setting it doesn't really matter since user-declread info takes precendence anyways
                    match sym.autodetected_type() {
                        Some(SymbolType::Function) => {}
                        _ => sym.set_type(SymbolType::GccExceptTableLabel, generated_by),
                    }
                }
            }
            _ => sym.set_type(SymbolType::GccExceptTableLabel, generated_by),
        }
        sym
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FindSettings {
    allow_addend: bool,
    check_upper_limit: bool,
}

impl FindSettings {
    pub const fn default() -> Self {
        Self {
            allow_addend: true,
            check_upper_limit: true,
        }
    }

    pub const fn new() -> Self {
        Self::default()
    }

    pub const fn with_allow_addend(self, allow_addend: bool) -> Self {
        Self {
            allow_addend,
            ..self
        }
    }

    pub const fn with_check_upper_limit(self, check_upper_limit: bool) -> Self {
        Self {
            check_upper_limit,
            ..self
        }
    }
}

impl SegmentMetadata {
    #[must_use]
    pub(crate) fn find_symbol(
        &self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<&SymbolMetadata> {
        if !settings.allow_addend {
            self.symbols.get(&vram)
        } else {
            let mut range = self.symbols.range(..=vram);

            if let Some((sym_vram, sym)) = range.next_back() {
                if settings.check_upper_limit && vram >= *sym_vram + sym.size() {
                    None
                } else {
                    Some(sym)
                }
            } else {
                None
            }
        }
    }

    #[must_use]
    pub(crate) fn find_symbol_mut(
        &mut self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<&mut SymbolMetadata> {
        if !settings.allow_addend {
            self.symbols.get_mut(&vram)
        } else {
            let mut range = self.symbols.range_mut(..=vram);

            if let Some((sym_vram, sym)) = range.next_back() {
                if settings.check_upper_limit && vram >= *sym_vram + sym.size() {
                    None
                } else {
                    Some(sym)
                }
            } else {
                None
            }
        }
    }

    pub(crate) fn find_symbols_range(
        &self,
        vram_start: Vram,
        vram_end: Vram,
    ) -> btree_map::Range<'_, Vram, SymbolMetadata> {
        self.symbols.range(vram_start..vram_end)
    }
}

impl SegmentMetadata {
    pub(crate) fn add_possible_pointer_in_data(
        &mut self,
        possible_pointer: Vram,
        rom_address_referencing_pointer: RomAddress,
    ) {
        self.new_pointer_in_data
            .entry(possible_pointer)
            .or_default()
            .push(rom_address_referencing_pointer);
    }
    pub(crate) fn is_vram_a_possible_pointer_in_data(&self, vram: Vram) -> bool {
        self.new_pointer_in_data.contains_key(&vram)
    }
}

#[cfg(test)]
mod tests {
    use rabbitizer::Vram;

    use super::*;

    #[test]
    fn check_symbol_bounds() {
        let rom_range = AddressRange::new(RomAddress::new(0), RomAddress::new(0x100));
        let vram_range = AddressRange::new(Vram::new(0), Vram::new(0x180));
        let ranges = RomVramRange::new(rom_range, vram_range);
        let mut segment = SegmentMetadata::new(ranges, None);

        segment.add_symbol(
            Vram::new(0x100C),
            None,
            GeneratedBy::Autogenerated,
            None,
            true,
        );
        segment.add_symbol(
            Vram::new(0x1000),
            None,
            GeneratedBy::Autogenerated,
            None,
            true,
        );
        segment.add_symbol(
            Vram::new(0x1004),
            None,
            GeneratedBy::Autogenerated,
            None,
            true,
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1000), FindSettings::new())
                .map(|sym| sym.vram()),
            Some(Vram::new(0x1000))
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1002), FindSettings::new())
                .map(|sym| sym.vram()),
            Some(Vram::new(0x1000))
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x0F00), FindSettings::new())
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x2000), FindSettings::new())
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(
                    Vram::new(0x1002),
                    FindSettings::new().with_allow_addend(false)
                )
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(
                    Vram::new(0x1100),
                    FindSettings::new().with_check_upper_limit(false)
                )
                .map(|sym| sym.vram()),
            Some(Vram::new(0x100C))
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1008), FindSettings::new())
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(
                    Vram::new(0x1008),
                    FindSettings::new().with_check_upper_limit(false)
                )
                .map(|sym| sym.vram()),
            Some(Vram::new(0x1004))
        );
    }
}
