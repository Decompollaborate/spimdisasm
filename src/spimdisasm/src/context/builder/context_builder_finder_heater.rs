/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    analysis::Preheater,
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
    rom_address::RomAddress,
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
        rom: RomAddress,
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
        rom: RomAddress,
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
        rom: RomAddress,
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
        rom: RomAddress,
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
    pub fn process(self) -> ContextBuilderFinderHeaterOverlays {
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
            rom: RomAddress,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_text(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "preanalyze_data")]
        pub fn py_preanalyze_data(
            &mut self,
            settings: &SectionDataSettings,
            raw_bytes: Cow<[u8]>,
            rom: RomAddress,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_data(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "preanalyze_rodata")]
        pub fn py_preanalyze_rodata(
            &mut self,
            settings: &SectionDataSettings,
            raw_bytes: Cow<[u8]>,
            rom: RomAddress,
            vram: u32, // Vram, // TODO
        ) {
            self.preanalyze_rodata(settings, &raw_bytes, rom, Vram::new(vram));
        }

        #[pyo3(name = "preanalyze_gcc_except_table")]
        pub fn py_preanalyze_gcc_except_table(
            &mut self,
            settings: &SectionDataSettings,
            raw_bytes: Cow<[u8]>,
            rom: RomAddress,
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
