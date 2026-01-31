/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::BTreeMap, sync::Arc};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, GlobalOffsetTable, Rom, RomVramRange, Size, Vram},
    analysis::{PreheatError, Preheater},
    collections::addended_ordered_map::AddendedOrderedMap,
    config::GlobalConfig,
    metadata::{
        IgnoredAddressRange, LabelMetadata, OverlayCategoryName, SegmentMetadata, SymbolMetadata,
    },
    sections::before_proc::{DataSectionSettings, ExecutableSectionSettings},
};

use super::SegmentBuilderKind;

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) struct SegmentHeater {
    kind: SegmentBuilderKind,
    ranges: RomVramRange,
    prioritised_overlays: Arc<[Arc<str>]>,
    user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
    user_labels: BTreeMap<Vram, LabelMetadata>,
    ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
    global_offset_table: Option<GlobalOffsetTable>,

    preheater: Preheater,
}

impl SegmentHeater {
    fn new(
        kind: SegmentBuilderKind,
        ranges: RomVramRange,
        prioritised_overlays: Arc<[Arc<str>]>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        global_offset_table: Option<GlobalOffsetTable>,
    ) -> Self {
        Self {
            kind: kind.clone(),
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            global_offset_table,

            preheater: Preheater::new(kind.into(), ranges),
        }
    }

    pub(crate) fn name(&self) -> Arc<str> {
        self.kind.name()
    }
    pub(crate) const fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }
    pub(crate) fn prioritised_overlays(&self) -> &[Arc<str>] {
        &self.prioritised_overlays
    }

    fn preheated_sections_rom(&self) -> &AddendedOrderedMap<Rom, Size> {
        self.preheater.preheated_sections_rom()
    }
}

impl SegmentHeater {
    fn preheat_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &ExecutableSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError> {
        self.preheater.preheat_text(
            global_config,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.user_labels,
            &self.ignored_addresses,
            self.global_offset_table.as_ref(),
        )
    }

    fn preheat_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError> {
        self.preheater.preheat_data(
            global_config,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.user_labels,
            &self.ignored_addresses,
            self.global_offset_table.as_ref(),
        )
    }

    fn preheat_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError> {
        self.preheater.preheat_rodata(
            global_config,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.user_labels,
            &self.ignored_addresses,
            self.global_offset_table.as_ref(),
        )
    }

    fn preheat_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: Arc<str>,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError> {
        self.preheater.preheat_gcc_except_table(
            global_config,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.user_labels,
            &self.ignored_addresses,
            self.global_offset_table.as_ref(),
        )
    }

    fn dump_info(&self) {
        // TODO: remove
        #[cfg(feature = "std")]
        {
            use std::{
                fs::File,
                io::{BufWriter, Write},
            };

            use crate::{addresses::Size, collections::addended_ordered_map::FindSettings};
            let segment_name = self.kind.name();

            let mut buf = BufWriter::new(
                File::create(format!("gathered_{segment_name}_references.csv",)).unwrap(),
            );
            buf.write_all(
                "vram,type,size,user_declared_size,autodetected_size,alignment,reference_counter,referenced_by,issues\n".as_bytes(),
            )
            .unwrap();
            for (vram, reference) in self.preheater.references().iter() {
                assert_eq!(*vram, reference.vram());
                let line = format!(
                    "0x{},{:?},{:?},{:?},{:?},{:?},{},\"{:?}\",",
                    vram,
                    reference.sym_type(),
                    reference.size(),
                    reference.user_declared_size(),
                    reference.autodetected_size(),
                    reference.alignment(),
                    reference.reference_counter(),
                    reference.referenced_by(),
                );
                buf.write_all(line.as_bytes()).unwrap();

                if let Some(size) = reference.size() {
                    let aux_vram = *vram + Size::new(size.inner() - 1);

                    let maybe_overlapped_sym = self
                        .preheater
                        .references()
                        .find(&aux_vram, FindSettings::new(true));

                    if let Some(maybe_overlapped_sym) = maybe_overlapped_sym {
                        let maybe_overlapped_vram = maybe_overlapped_sym.vram();
                        if maybe_overlapped_vram != *vram {
                            buf.write_all(
                                format!(
                                    "The size of this symbol overlaps with address 0x{maybe_overlapped_vram}"
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                        }
                    } else {
                        buf.write_all("what?".as_bytes()).unwrap();
                    }
                }

                buf.write_all(";".as_bytes()).unwrap();

                if let Some(alignment) = reference.alignment() {
                    if (vram.inner() % alignment as u32) != 0 {
                        buf.write_all("Alignment doesn't make sense".as_bytes())
                            .unwrap();
                    }
                }

                buf.write_all("\n".as_bytes()).unwrap();
            }
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GlobalSegmentHeater {
    inner: SegmentHeater,
}

impl GlobalSegmentHeater {
    pub(crate) fn new(
        kind: SegmentBuilderKind,
        ranges: RomVramRange,
        prioritised_overlays: Arc<[Arc<str>]>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        global_offset_table: Option<GlobalOffsetTable>,
    ) -> Self {
        Self {
            inner: SegmentHeater::new(
                kind,
                ranges,
                prioritised_overlays,
                user_symbols,
                user_labels,
                ignored_addresses,
                global_offset_table,
            ),
        }
    }

    pub(crate) const fn inner(&self) -> &SegmentHeater {
        &self.inner
    }

    pub(crate) fn name(&self) -> Arc<str> {
        self.inner.name()
    }
    pub(crate) const fn ranges(&self) -> &RomVramRange {
        self.inner.ranges()
    }

    pub(crate) fn preheated_sections_rom(&self) -> &AddendedOrderedMap<Rom, Size> {
        self.inner.preheated_sections_rom()
    }

    pub fn preheat_text<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &ExecutableSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .preheat_text(global_config, settings, name.into(), raw_bytes, rom, vram)
    }

    pub fn preheat_data<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .preheat_data(global_config, settings, name.into(), raw_bytes, rom, vram)
    }

    pub fn preheat_rodata<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .preheat_rodata(global_config, settings, name.into(), raw_bytes, rom, vram)
    }

    pub fn preheat_gcc_except_table<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner.preheat_gcc_except_table(
            global_config,
            settings,
            name.into(),
            raw_bytes,
            rom,
            vram,
        )
    }

    #[must_use]
    pub(crate) fn finish(
        self,
        visible_overlay_ranges: Arc<[AddressRange<Vram>]>,
    ) -> SegmentMetadata {
        self.inner.dump_info();

        let SegmentHeater {
            kind,
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            global_offset_table,
            preheater,
        } = self.inner;

        SegmentMetadata::new_global(
            kind.into_name(),
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            preheater,
            visible_overlay_ranges,
            global_offset_table,
        )
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlaySegmentHeater {
    inner: SegmentHeater,
    category_name: OverlayCategoryName,
}

impl OverlaySegmentHeater {
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new(
        kind: SegmentBuilderKind,
        ranges: RomVramRange,
        prioritised_overlays: Arc<[Arc<str>]>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        user_labels: BTreeMap<Vram, LabelMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        global_offset_table: Option<GlobalOffsetTable>,
        category_name: OverlayCategoryName,
    ) -> Self {
        Self {
            inner: SegmentHeater::new(
                kind,
                ranges,
                prioritised_overlays,
                user_symbols,
                user_labels,
                ignored_addresses,
                global_offset_table,
            ),
            category_name,
        }
    }

    pub(crate) const fn inner(&self) -> &SegmentHeater {
        &self.inner
    }

    pub(crate) fn name(&self) -> Arc<str> {
        self.inner.name()
    }
    pub(crate) fn category_name(&self) -> &OverlayCategoryName {
        &self.category_name
    }
    pub(crate) const fn ranges(&self) -> &RomVramRange {
        self.inner.ranges()
    }
    pub(crate) fn prioritised_overlays(&self) -> &[Arc<str>] {
        self.inner.prioritised_overlays()
    }
    pub(crate) const fn preheater(&self) -> &Preheater {
        &self.inner.preheater
    }
    pub(crate) const fn preheater_mut(&mut self) -> &mut Preheater {
        &mut self.inner.preheater
    }

    pub(crate) fn preheated_sections_rom(&self) -> &AddendedOrderedMap<Rom, Size> {
        self.inner.preheated_sections_rom()
    }

    pub fn preheat_text<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &ExecutableSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .preheat_text(global_config, settings, name.into(), raw_bytes, rom, vram)
    }

    pub fn preheat_data<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .preheat_data(global_config, settings, name.into(), raw_bytes, rom, vram)
    }

    pub fn preheat_rodata<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .preheat_rodata(global_config, settings, name.into(), raw_bytes, rom, vram)
    }

    pub fn preheat_gcc_except_table<T>(
        &mut self,
        global_config: &GlobalConfig,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) -> Result<(), PreheatError>
    where
        T: Into<Arc<str>>,
    {
        self.inner.preheat_gcc_except_table(
            global_config,
            settings,
            name.into(),
            raw_bytes,
            rom,
            vram,
        )
    }

    #[must_use]
    pub(crate) fn finish(
        self,
        visible_overlay_ranges: Arc<[AddressRange<Vram>]>,
    ) -> SegmentMetadata {
        self.inner.dump_info();

        let SegmentHeater {
            kind,
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            global_offset_table,
            preheater,
        } = self.inner;

        SegmentMetadata::new_overlay(
            kind.into_name(),
            ranges,
            prioritised_overlays,
            user_symbols,
            user_labels,
            ignored_addresses,
            preheater,
            visible_overlay_ranges,
            global_offset_table,
            self.category_name,
        )
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl GlobalSegmentHeater {
        #[pyo3(name = "preheat_text")]
        pub fn py_preheat_text(
            &mut self,
            global_config: &GlobalConfig,
            settings: &ExecutableSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_text(global_config, settings, name, raw_bytes, rom, vram)
        }

        #[pyo3(name = "preheat_data")]
        pub fn py_preheat_data(
            &mut self,
            global_config: &GlobalConfig,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_data(global_config, settings, name, raw_bytes, rom, vram)
        }

        #[pyo3(name = "preheat_rodata")]
        pub fn py_preheat_rodata(
            &mut self,
            global_config: &GlobalConfig,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_rodata(global_config, settings, name, raw_bytes, rom, vram)
        }

        #[pyo3(name = "preheat_gcc_except_table")]
        pub fn py_preheat_gcc_except_table(
            &mut self,
            global_config: &GlobalConfig,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_gcc_except_table(global_config, settings, name, raw_bytes, rom, vram)
        }
    }

    #[pymethods]
    impl OverlaySegmentHeater {
        #[pyo3(name = "preheat_text")]
        pub fn py_preheat_text(
            &mut self,
            global_config: &GlobalConfig,
            settings: &ExecutableSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_text(global_config, settings, name, raw_bytes, rom, vram)
        }

        #[pyo3(name = "preheat_data")]
        pub fn py_preheat_data(
            &mut self,
            global_config: &GlobalConfig,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_data(global_config, settings, name, raw_bytes, rom, vram)
        }

        #[pyo3(name = "preheat_rodata")]
        pub fn py_preheat_rodata(
            &mut self,
            global_config: &GlobalConfig,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_rodata(global_config, settings, name, raw_bytes, rom, vram)
        }

        #[pyo3(name = "preheat_gcc_except_table")]
        pub fn py_preheat_gcc_except_table(
            &mut self,
            global_config: &GlobalConfig,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) -> Result<(), PreheatError> {
            self.preheat_gcc_except_table(global_config, settings, name, raw_bytes, rom, vram)
        }
    }
}
