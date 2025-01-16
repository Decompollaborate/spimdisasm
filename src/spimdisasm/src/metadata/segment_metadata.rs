/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, RomVramRange, Size, Vram};
use crate::analysis::{reference_wrapper, Preheater, ReferenceWrapper};
use crate::collections::addended_ordered_map::{self, AddendedOrderedMap, FindSettings};

use super::SymbolMetadata;
use super::{symbol_metadata::GeneratedBy, OverlayCategoryName};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct SegmentMetadata {
    ranges: RomVramRange,

    category_name: Option<OverlayCategoryName>,
    name: Option<String>,

    symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
    // constants: BTreeMap<Vram, SymbolMetadata>,

    //
    /// Stuff that looks like pointers. Found referenced by data.
    new_pointer_in_data: BTreeMap<Vram, Vec<Rom>>,
    //
    // is_the_unknown_segment: bool,
    preheater: Preheater,
}

impl SegmentMetadata {
    const fn new(
        ranges: RomVramRange,
        category_name: Option<OverlayCategoryName>,
        name: Option<String>,
    ) -> Self {
        Self {
            ranges,
            category_name,
            name,

            symbols: AddendedOrderedMap::new(),
            new_pointer_in_data: BTreeMap::new(),

            preheater: Preheater::new(),
        }
    }

    pub(crate) const fn new_global(ranges: RomVramRange) -> Self {
        Self::new(ranges, None, None)
    }

    pub(crate) const fn new_overlay(
        ranges: RomVramRange,
        category_name: OverlayCategoryName,
        name: String,
    ) -> Self {
        Self::new(ranges, Some(category_name), Some(name))
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub const fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }

    pub const fn rom_range(&self) -> &AddressRange<Rom> {
        self.ranges.rom()
    }
    /*
    pub(crate) fn rom_range_mut(&mut self) -> &mut AddressRange<Rom> {
        &mut self.rom_range
    }
    */
    pub fn in_rom_range(&self, rom: Rom) -> bool {
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
    pub fn vram_from_rom(&self, rom: Rom) -> Option<Vram> {
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

    pub const fn symbols(&self) -> &AddendedOrderedMap<Vram, SymbolMetadata> {
        &self.symbols
    }

    pub(crate) fn preheater_mut(&mut self) -> &mut Preheater {
        &mut self.preheater
    }
}

impl SegmentMetadata {
    pub(crate) fn add_symbol(
        &mut self,
        vram: Vram,
        generated_by: GeneratedBy,
        allow_sym_with_addend: bool, // false
    ) -> Result<&mut SymbolMetadata, AddSymbolError> {
        if self.in_vram_range(vram) {
            let sym = self.symbols.find_mut_or_insert_with(
                vram,
                FindSettings::new(allow_sym_with_addend),
                || (vram, SymbolMetadata::new(generated_by, vram)),
            );

            Ok(sym)
        } else {
            Err(AddSymbolError {
                vram,
                segment_ranges: *self.vram_range(),
                name: self.name.clone(),
            })
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct AddSymbolError {
    vram: Vram,
    segment_ranges: AddressRange<Vram>,
    name: Option<String>,
}
impl fmt::Display for AddSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when trying to add symbol to ")?;
        if let Some(name) = &self.name {
            write!(f, "overlay segment '{}'", name)?;
        } else {
            write!(f, "global segment")?;
        }
        write!(f, ": ")?;
        write!(
            f,
            "Symbol's vram 0x{} is out of the segment's range `{}`",
            self.vram, self.segment_ranges
        )
    }
}
impl error::Error for AddSymbolError {}

impl SegmentMetadata {
    #[must_use]
    pub(crate) fn find_symbol(
        &self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<&SymbolMetadata> {
        self.symbols.find(&vram, settings)
    }

    #[must_use]
    pub(crate) fn find_symbol_mut(
        &mut self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<&mut SymbolMetadata> {
        self.symbols.find_mut(&vram, settings)
    }

    pub(crate) fn find_symbol_ranges_mut(
        &mut self,
        vram_range: AddressRange<Vram>,
    ) -> addended_ordered_map::RangeMut<Vram, SymbolMetadata> {
        self.symbols.range_mut(vram_range)
    }
}

impl SegmentMetadata {
    #[must_use]
    pub(crate) fn find_reference(
        &self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<ReferenceWrapper> {
        ReferenceWrapper::find(self, &self.preheater, vram, settings)
    }

    pub(crate) fn find_references_range(
        &self,
        vram_range: AddressRange<Vram>,
    ) -> reference_wrapper::Range {
        ReferenceWrapper::range(self, &self.preheater, vram_range)
    }
}

impl SegmentMetadata {
    pub(crate) fn add_possible_pointer_in_data(
        &mut self,
        possible_pointer: Vram,
        rom_address_referencing_pointer: Rom,
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
    use super::*;

    #[test]
    fn check_symbol_bounds() {
        let rom_range = AddressRange::new(Rom::new(0), Rom::new(0x1400));
        let vram_range = AddressRange::new(Vram::new(0), Vram::new(0x1800));
        let ranges = RomVramRange::new(rom_range, vram_range);
        let mut segment = SegmentMetadata::new_global(ranges);

        segment
            .add_symbol(Vram::new(0x100C), GeneratedBy::Autogenerated, true)
            .unwrap();
        *segment
            .add_symbol(Vram::new(0x1000), GeneratedBy::Autogenerated, true)
            .unwrap()
            .user_declared_size_mut() = Some(Size::new(4));
        *segment
            .add_symbol(Vram::new(0x1004), GeneratedBy::Autogenerated, true)
            .unwrap()
            .user_declared_size_mut() = Some(Size::new(4));

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1000), FindSettings::new(true))
                .map(|sym| sym.vram()),
            Some(Vram::new(0x1000))
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1002), FindSettings::new(true))
                .map(|sym| sym.vram()),
            Some(Vram::new(0x1000))
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x0F00), FindSettings::new(true))
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x2000), FindSettings::new(true))
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1002), FindSettings::new(false))
                .map(|sym| sym.vram()),
            None
        );

        assert_eq!(
            segment
                .find_symbol(
                    Vram::new(0x1100),
                    FindSettings::new(true).with_reject_sizeless_addended(false)
                )
                .map(|sym| sym.vram()),
            Some(Vram::new(0x100C))
        );

        assert_eq!(
            segment
                .find_symbol(Vram::new(0x1008), FindSettings::new(true))
                .map(|sym| sym.vram()),
            None
        );
    }
}
