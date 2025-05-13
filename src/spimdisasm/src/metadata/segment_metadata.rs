/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    collections::btree_map::{self, BTreeMap},
    sync::Arc,
};
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, GlobalOffsetTable, Rom, RomVramRange, Size, Vram};
use crate::analysis::{reference_wrapper, Preheater, ReferenceWrapper};
use crate::collections::addended_ordered_map::{AddendedOrderedMap, FindSettings};
use crate::section_type::SectionType;

use super::{symbol_metadata::GeneratedBy, OverlayCategoryName, SymbolNameGenerationSettings};
use super::{
    AddLabelError, IgnoredAddressRange, LabelMetadata, LabelType, OwnerSegmentKind, ReferrerInfo,
    SymbolMetadata, SymbolType,
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct SegmentMetadata {
    ranges: RomVramRange,

    category_name: Option<OverlayCategoryName>,
    name: Option<Arc<str>>,

    prioritised_overlays: Arc<[Arc<str>]>,
    visible_overlay_ranges: Arc<[AddressRange<Vram>]>,

    symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
    labels: BTreeMap<Vram, LabelMetadata>,
    // constants: BTreeMap<Vram, SymbolMetadata>,
    ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
    global_offset_table: Option<GlobalOffsetTable>,

    preheater: Preheater,

    is_the_unknown_segment: bool,
}

impl SegmentMetadata {
    #[allow(clippy::too_many_arguments)]
    fn new(
        ranges: RomVramRange,
        prioritised_overlays: Arc<[Arc<str>]>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        preheater: Preheater,
        visible_overlay_ranges: Arc<[AddressRange<Vram>]>,
        global_offset_table: Option<GlobalOffsetTable>,
        category_name: Option<OverlayCategoryName>,
        name: Option<Arc<str>>,
    ) -> Self {
        Self {
            ranges,
            category_name,
            name,

            prioritised_overlays,
            visible_overlay_ranges,

            symbols: user_symbols,
            labels: user_labels,
            ignored_addresses,
            global_offset_table,

            preheater,

            is_the_unknown_segment: false,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new_global(
        ranges: RomVramRange,
        prioritised_overlays: Arc<[Arc<str>]>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        preheater: Preheater,
        visible_overlay_ranges: Arc<[AddressRange<Vram>]>,
        global_offset_table: Option<GlobalOffsetTable>,
    ) -> Self {
        Self::new(
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            preheater,
            visible_overlay_ranges,
            global_offset_table,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_overlay(
        ranges: RomVramRange,
        prioritised_overlays: Arc<[Arc<str>]>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        preheater: Preheater,
        visible_overlay_ranges: Arc<[AddressRange<Vram>]>,
        global_offset_table: Option<GlobalOffsetTable>,
        category_name: OverlayCategoryName,
        name: Arc<str>,
    ) -> Self {
        Self::new(
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            preheater,
            visible_overlay_ranges,
            global_offset_table,
            Some(category_name),
            Some(name),
        )
    }

    pub(crate) fn new_unknown_segment() -> Self {
        let rom_range = AddressRange::new(Rom::new(0x00000000), Rom::new(0xFFFFFFFF));
        let vram_range = AddressRange::new(Vram::new(0x00000000), Vram::new(0xFFFFFFFF));
        let ranges = RomVramRange::new(rom_range, vram_range);
        Self {
            is_the_unknown_segment: true,
            ..Self::new(
                ranges,
                Arc::new([]),
                AddendedOrderedMap::new(),
                BTreeMap::new(),
                AddendedOrderedMap::new(),
                Preheater::new(None, ranges),
                Arc::new([]),
                None,
                None,
                None,
            )
        }
    }

    pub fn name(&self) -> Option<Arc<str>> {
        self.name.clone()
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

    pub(crate) fn prioritised_overlays(&self) -> &[Arc<str>] {
        &self.prioritised_overlays
    }

    // TODO: actually use
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn is_vram_in_visible_overlay(&self, vram: Vram) -> bool {
        self.visible_overlay_ranges.iter().any(|x| x.in_range(vram))
    }

    #[must_use]
    pub(crate) fn is_vram_ignored(&self, vram: Vram) -> bool {
        self.ignored_addresses
            .find(&vram, FindSettings::new(true))
            .is_some()
    }

    pub const fn symbols(&self) -> &AddendedOrderedMap<Vram, SymbolMetadata> {
        &self.symbols
    }
    pub const fn labels(&self) -> &BTreeMap<Vram, LabelMetadata> {
        &self.labels
    }

    pub const fn global_offset_table(&self) -> Option<&GlobalOffsetTable> {
        self.global_offset_table.as_ref()
    }
}

impl SegmentMetadata {
    pub(crate) fn add_symbol(
        &mut self,
        vram: Vram,
        allow_sym_with_addend: bool, // false
        symbol_name_generation_settings: SymbolNameGenerationSettings,
    ) -> Result<&mut SymbolMetadata, AddSymbolError> {
        if self.in_vram_range(vram) {
            let (sym, _) = self.symbols.find_mut_or_insert_with(
                vram,
                FindSettings::new(allow_sym_with_addend),
                || {
                    let owner_segment_kind = if self.is_the_unknown_segment {
                        OwnerSegmentKind::Unknown
                    } else if let Some(name) = &self.name {
                        OwnerSegmentKind::Overlay(name.clone())
                    } else {
                        OwnerSegmentKind::Global
                    };

                    SymbolMetadata::new(
                        GeneratedBy::Autogenerated,
                        vram,
                        owner_segment_kind,
                        symbol_name_generation_settings,
                    )
                },
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

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn add_self_symbol<F>(
        &mut self,
        vram: Vram,
        rom: Option<Rom>,
        size: Size,
        section_type: SectionType,
        sym_type: Option<SymbolType>,
        trailing_size_callback: F,
        symbol_name_generation_settings: SymbolNameGenerationSettings,
    ) -> Result<&mut SymbolMetadata, AddSymbolError>
    where
        F: Fn(&SymbolMetadata) -> Size,
    {
        // Remove every other symbols that may overlap this one
        if size.inner() > 1 {
            let start = vram + Size::new(1);
            let end = vram + size;

            // `retain` can be pretty slow, so we only do it if we know for a
            // fact that there's at least one overlapping symbol.
            if self.symbols.range(start..end).next().is_some() {
                let overlapping_range = start..end;
                self.symbols.retain(|k, _| !overlapping_range.contains(k));
            }
        }

        let metadata = self.add_symbol(vram, false, symbol_name_generation_settings)?;
        metadata.set_defined();
        *metadata.rom_mut() = rom;
        *metadata.autodetected_size_mut() = Some(size);
        *metadata.section_type_mut() = Some(section_type);

        metadata.set_trailing_padding_size(trailing_size_callback(metadata));

        if let Some(sym_type) = sym_type {
            metadata.set_type(sym_type, GeneratedBy::Autogenerated);
        }

        Ok(metadata)
    }

    pub(crate) fn add_label(
        &mut self,
        vram: Vram,
        label_type: LabelType,
        creator: ReferrerInfo,
    ) -> Result<&mut LabelMetadata, AddLabelError> {
        if self.in_vram_range(vram) {
            let label = self.labels.entry(vram).or_insert_with(|| {
                let owner_segment_kind = if self.is_the_unknown_segment {
                    OwnerSegmentKind::Unknown
                } else if let Some(name) = &self.name {
                    OwnerSegmentKind::Overlay(name.clone())
                } else {
                    OwnerSegmentKind::Global
                };
                LabelMetadata::new(vram, owner_segment_kind, label_type)
            });

            label.set_autodetected_type(label_type);
            label.add_creator(creator);

            Ok(label)
        } else {
            Err(AddLabelError::new_vram_out_of_range(
                vram,
                label_type,
                self.name.clone(),
                *self.vram_range(),
            ))
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct AddSymbolError {
    vram: Vram,
    segment_ranges: AddressRange<Vram>,
    name: Option<Arc<str>>,
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

    #[must_use]
    pub(crate) fn find_label(&self, vram: Vram) -> Option<&LabelMetadata> {
        self.labels.get(&vram)
    }

    #[must_use]
    pub(crate) fn find_label_mut(&mut self, vram: Vram) -> Option<&mut LabelMetadata> {
        self.labels.get_mut(&vram)
    }

    pub(crate) fn find_label_range_mut(
        &mut self,
        vram_range: AddressRange<Vram>,
    ) -> btree_map::RangeMut<Vram, LabelMetadata> {
        self.labels.range_mut(vram_range)
    }
}

impl SegmentMetadata {
    #[must_use]
    pub(crate) fn find_reference(
        &self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<ReferenceWrapper> {
        ReferenceWrapper::find(&self.symbols, &self.preheater, vram, settings)
    }

    pub(crate) fn find_references_range(
        &self,
        vram_range: AddressRange<Vram>,
    ) -> reference_wrapper::Range {
        ReferenceWrapper::range(&self.symbols, &self.preheater, vram_range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn check_symbol_bounds() {
        let symbol_name_generation_settings = SymbolNameGenerationSettings::new();
        let rom_range = AddressRange::new(Rom::new(0), Rom::new(0x1400));
        let vram_range = AddressRange::new(Vram::new(0), Vram::new(0x1800));
        let ranges = RomVramRange::new(rom_range, vram_range);
        let mut segment = SegmentMetadata::new_global(
            ranges,
            Arc::new([]),
            AddendedOrderedMap::new(),
            BTreeMap::new(),
            AddendedOrderedMap::new(),
            Preheater::new(None, ranges),
            Arc::new([]),
            None,
        );

        segment
            .add_symbol(
                Vram::new(0x100C),
                true,
                symbol_name_generation_settings.clone(),
            )
            .unwrap();
        *segment
            .add_symbol(
                Vram::new(0x1000),
                true,
                symbol_name_generation_settings.clone(),
            )
            .unwrap()
            .user_declared_size_mut() = Some(Size::new(4));
        *segment
            .add_symbol(Vram::new(0x1004), true, symbol_name_generation_settings)
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
                .find_symbol(Vram::new(0x1008), FindSettings::new(true))
                .map(|sym| sym.vram()),
            None
        );
    }
}
