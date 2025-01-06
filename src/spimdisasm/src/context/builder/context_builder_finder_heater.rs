/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Vram},
    analysis::Preheater,
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
    sections::{SectionDataSettings, SectionExecutableSettings},
};

use super::ContextBuilderFinderHeaterOverlays;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderFinderHeater {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,

    preheater: Preheater,
}

impl ContextBuilderFinderHeater {
    pub(crate) fn new(
        global_config: GlobalConfig,

        global_segment: SegmentMetadata,
        overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
    ) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments,

            preheater: Preheater::new(),
        }
    }

    pub fn preanalyze_text(
        &mut self,
        settings: &SectionExecutableSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_text(
            &self.global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.global_segment,
        );
    }

    pub fn preanalyze_data(
        &mut self,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_data(
            &self.global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.global_segment,
        );
    }

    pub fn preanalyze_rodata(
        &mut self,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_rodata(
            &self.global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.global_segment,
        );
    }

    pub fn preanalyze_gcc_except_table(
        &mut self,
        settings: &SectionDataSettings,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
    ) {
        self.preheater.preheat_gcc_except_table(
            &self.global_config,
            settings,
            raw_bytes,
            rom,
            vram,
            &self.global_segment,
        );
    }

    #[must_use]
    pub fn process(mut self) -> ContextBuilderFinderHeaterOverlays {
        // TODO: remove
        #[cfg(feature = "std")]
        {
            use std::{
                fs::File,
                io::{BufWriter, Write},
            };

            use crate::{addresses::Size, collections::addended_ordered_map::FindSettings};

            let mut buf = BufWriter::new(File::create("gathered_global_references.csv").unwrap());
            buf.write_all(
                "vram,type,size,alignment,reference_counter,referenced_by,issues\n".as_bytes(),
            )
            .unwrap();
            for (vram, reference) in self.preheater.references().iter() {
                assert_eq!(*vram, reference.vram());
                let line = format!(
                    "0x{},{:?},{:?},{:?},{},\"{:?}\",",
                    vram,
                    reference.sym_type(),
                    reference.size(),
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
                        .find(&aux_vram, FindSettings::default().with_allow_addend(true));
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

        *self.global_segment.preheater_mut() = self.preheater;

        ContextBuilderFinderHeaterOverlays::new(
            self.global_config,
            self.global_segment,
            self.overlay_segments,
        )
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use std::borrow::Cow;

    use super::*;

    #[pymethods]
    impl ContextBuilderFinderHeater {
        #[pyo3(name = "preanalyze_text")]
        pub fn py_preanalyze_text(
            &mut self,
            settings: &SectionExecutableSettings,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_text(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "preanalyze_data")]
        pub fn py_preanalyze_data(
            &mut self,
            settings: &SectionDataSettings,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_data(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "preanalyze_rodata")]
        pub fn py_preanalyze_rodata(
            &mut self,
            settings: &SectionDataSettings,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_rodata(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "preanalyze_gcc_except_table")]
        pub fn py_preanalyze_gcc_except_table(
            &mut self,
            settings: &SectionDataSettings,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_gcc_except_table(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderFinderHeaterOverlays {
            self.clone().process()
        }
    }
}
