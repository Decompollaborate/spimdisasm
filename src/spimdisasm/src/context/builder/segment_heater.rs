/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Vram},
    analysis::Preheater,
    config::GlobalConfig,
    metadata::SegmentMetadata,
    sections::{SectionDataSettings, SectionExecutableSettings},
};

#[derive(Debug, Clone, PartialEq)]
struct SegmentHeater {
    segment: SegmentMetadata,

    preheater: Preheater,
}

impl SegmentHeater {
    const fn new(segment: SegmentMetadata) -> Self {
        Self {
            segment,
            preheater: Preheater::new(),
        }
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
        self.preheater
            .preheat_text(global_config, settings, raw_bytes, rom, vram, &self.segment);
    }

    fn preanalyze_data(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater
            .preheat_data(global_config, settings, raw_bytes, rom, vram, &self.segment);
    }

    fn preanalyze_rodata(
        &mut self,
        global_config: &GlobalConfig,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater
            .preheat_rodata(global_config, settings, raw_bytes, rom, vram, &self.segment);
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
            &self.segment,
        );
    }

    #[must_use]
    fn finish(self) -> SegmentMetadata {
        self.dump_info();

        let mut segment = self.segment;
        *segment.preheater_mut() = self.preheater;

        segment
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

            let mut buf = BufWriter::new(
                File::create(format!(
                    "gathered_{}_references.csv",
                    self.segment.name().unwrap_or("global")
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GlobalSegmentHeater {
    inner: SegmentHeater,
}

impl GlobalSegmentHeater {
    pub(crate) const fn new(segment: SegmentMetadata) -> Self {
        Self {
            inner: SegmentHeater::new(segment),
        }
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
    pub(crate) fn finish(self) -> SegmentMetadata {
        self.inner.finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlaySegmentHeater {
    inner: SegmentHeater,
}

impl OverlaySegmentHeater {
    pub(crate) const fn new(segment: SegmentMetadata) -> Self {
        Self {
            inner: SegmentHeater::new(segment),
        }
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
    pub(crate) fn finish(self) -> SegmentMetadata {
        self.inner.finish()
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
