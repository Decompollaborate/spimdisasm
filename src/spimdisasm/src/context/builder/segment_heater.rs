/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{string::String, vec::Vec};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Vram},
    analysis::Preheater,
    collections::addended_ordered_map::AddendedOrderedMap,
    config::GlobalConfig,
    metadata::{IgnoredAddressRange, OverlayCategoryName, SegmentMetadata, SymbolMetadata},
    sections::{SectionDataSettings, SectionExecutableSettings},
};

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) struct SegmentHeater {
    ranges: RomVramRange,
    prioritised_overlays: Vec<String>,
    user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
    ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,

    preheater: Preheater,
}

impl SegmentHeater {
    const fn new(
        ranges: RomVramRange,
        prioritised_overlays: Vec<String>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Self {
        Self {
            ranges,
            prioritised_overlays,
            user_symbols,
            ignored_addresses,

            preheater: Preheater::new(ranges),
        }
    }

    pub(crate) const fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }
    pub(crate) fn prioritised_overlays(&self) -> &[String] {
        &self.prioritised_overlays
    }
}

impl SegmentHeater {
    fn preanalyze_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionExecutableSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_text(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.ignored_addresses,
        );
    }

    fn preanalyze_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_data(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.ignored_addresses,
        );
    }

    fn preanalyze_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_rodata(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.ignored_addresses,
        );
    }

    fn preanalyze_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_gcc_except_table(
            global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.user_symbols,
            &self.ignored_addresses,
        );
    }

    fn dump_info(&self, segment_name: Option<&str>) {
        let _avoid_unused_warning = segment_name;

        // TODO: remove
        #[cfg(feature = "std")]
        {
            use std::{
                fs::File,
                io::{BufWriter, Write},
            };

            use crate::{addresses::Size, collections::addended_ordered_map::FindSettings};

            let mut buf = BufWriter::new(
                File::create(format!(
                    "gathered_{}_references.csv",
                    segment_name.unwrap_or("global")
                ))
                .unwrap(),
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
                    if maybe_overlapped_sym.is_none() {
                        buf.write_all("what?".as_bytes()).unwrap();
                    } else if maybe_overlapped_sym.unwrap().vram() != *vram {
                        buf.write_all(
                            format!(
                                "The size of this symbol overlaps with address 0x{}",
                                maybe_overlapped_sym.unwrap().vram()
                            )
                            .as_bytes(),
                        )
                        .unwrap();
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
    pub(crate) const fn new(
        ranges: RomVramRange,
        prioritised_overlays: Vec<String>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
    ) -> Self {
        Self {
            inner: SegmentHeater::new(
                ranges,
                prioritised_overlays,
                user_symbols,
                ignored_addresses,
            ),
        }
    }

    pub(crate) const fn inner(&self) -> &SegmentHeater {
        &self.inner
    }

    pub fn preanalyze_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionExecutableSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_text(global_config, settings, raw_bytes, rom, vram);
    }

    pub fn preanalyze_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_data(global_config, settings, raw_bytes, rom, vram);
    }

    pub fn preanalyze_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_rodata(global_config, settings, raw_bytes, rom, vram);
    }

    pub fn preanalyze_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_gcc_except_table(global_config, settings, raw_bytes, rom, vram);
    }

    #[must_use]
    pub(crate) fn finish(self, visible_overlay_ranges: Vec<AddressRange<Vram>>) -> SegmentMetadata {
        self.inner.dump_info(None);

        SegmentMetadata::new_global(
            self.inner.ranges,
            self.inner.prioritised_overlays,
            self.inner.user_symbols,
            self.inner.ignored_addresses,
            self.inner.preheater,
            visible_overlay_ranges,
        )
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlaySegmentHeater {
    inner: SegmentHeater,
    name: String,
    category_name: OverlayCategoryName,
}

impl OverlaySegmentHeater {
    pub(crate) const fn new(
        ranges: RomVramRange,
        name: String,
        prioritised_overlays: Vec<String>,
        user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
        ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
        category_name: OverlayCategoryName,
    ) -> Self {
        Self {
            inner: SegmentHeater::new(
                ranges,
                prioritised_overlays,
                user_symbols,
                ignored_addresses,
            ),
            name,
            category_name,
        }
    }

    pub(crate) const fn inner(&self) -> &SegmentHeater {
        &self.inner
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn category_name(&self) -> &OverlayCategoryName {
        &self.category_name
    }
    pub(crate) const fn ranges(&self) -> &RomVramRange {
        self.inner.ranges()
    }
    pub(crate) fn prioritised_overlays(&self) -> &[String] {
        self.inner.prioritised_overlays()
    }
    pub(crate) const fn preheater(&self) -> &Preheater {
        &self.inner.preheater
    }
    pub(crate) const fn preheater_mut(&mut self) -> &mut Preheater {
        &mut self.inner.preheater
    }

    pub fn preanalyze_text(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionExecutableSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_text(global_config, settings, raw_bytes, rom, vram);
    }

    pub fn preanalyze_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_data(global_config, settings, raw_bytes, rom, vram);
    }

    pub fn preanalyze_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_rodata(global_config, settings, raw_bytes, rom, vram);
    }

    pub fn preanalyze_gcc_except_table(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.inner
            .preanalyze_gcc_except_table(global_config, settings, raw_bytes, rom, vram);
    }

    #[must_use]
    pub(crate) fn finish(self, visible_overlay_ranges: Vec<AddressRange<Vram>>) -> SegmentMetadata {
        self.inner.dump_info(Some(&self.name));

        SegmentMetadata::new_overlay(
            self.inner.ranges,
            self.inner.prioritised_overlays,
            self.inner.user_symbols,
            self.inner.ignored_addresses,
            self.inner.preheater,
            visible_overlay_ranges,
            self.category_name,
            self.name,
        )
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl GlobalSegmentHeater {
        #[pyo3(name = "preanalyze_text")]
        pub fn py_preanalyze_text(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionExecutableSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_text(global_config, settings, raw_bytes, rom, vram);
        }

        #[pyo3(name = "preanalyze_data")]
        pub fn py_preanalyze_data(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionDataSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_data(global_config, settings, raw_bytes, rom, vram);
        }

        #[pyo3(name = "preanalyze_rodata")]
        pub fn py_preanalyze_rodata(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionDataSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_rodata(global_config, settings, raw_bytes, rom, vram);
        }

        #[pyo3(name = "preanalyze_gcc_except_table")]
        pub fn py_preanalyze_gcc_except_table(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionDataSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_gcc_except_table(global_config, settings, raw_bytes, rom, vram);
        }
    }

    #[pymethods]
    impl OverlaySegmentHeater {
        #[pyo3(name = "preanalyze_text")]
        pub fn py_preanalyze_text(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionExecutableSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_text(global_config, settings, raw_bytes, rom, vram);
        }

        #[pyo3(name = "preanalyze_data")]
        pub fn py_preanalyze_data(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionDataSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_data(global_config, settings, raw_bytes, rom, vram);
        }

        #[pyo3(name = "preanalyze_rodata")]
        pub fn py_preanalyze_rodata(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionDataSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_rodata(global_config, settings, raw_bytes, rom, vram);
        }

        #[pyo3(name = "preanalyze_gcc_except_table")]
        pub fn py_preanalyze_gcc_except_table(
            &mut self,
            global_config: &GlobalConfig,
            settings: &SectionDataSettings,
            raw_bytes: &[u8],
            rom: Rom,
            vram: Vram,
        ) {
            self.preanalyze_gcc_except_table(global_config, settings, raw_bytes, rom, vram);
        }
    }
}
