/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::BTreeMap;
use rabbitizer::Vram;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
    rom_address::RomAddress,
    sections::{SectionDataSettings, SectionExecutableSettings},
};

use super::ContextBuilderFinderHeaterOverlays;

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderFinderHeater {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderFinderHeater {
    pub(crate) fn new(
        global_config: GlobalConfig,

        global_segment: SegmentMetadata,
        overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
    ) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments,
        }
    }

    pub fn preanalyze_text(
        &mut self,
        _settings: &SectionExecutableSettings,
        _raw_bytes: &[u8],
        _rom: RomAddress,
        _vram: Vram,
    ) {
    }

    pub fn preanalyze_data(
        &mut self,
        _settings: &SectionDataSettings,
        _raw_bytes: &[u8],
        _rom: RomAddress,
        _vram: Vram,
    ) {
    }

    pub fn preanalyze_rodata(
        &mut self,
        _settings: &SectionDataSettings,
        raw_bytes: &[u8],
        _rom: RomAddress,
        vram: Vram,
    ) {
        // Look for stuff that looks like addresses which point to symbols on this section
        let displacement = (4 - (vram.inner() % 4) as usize) % 4;
        for (i, _word_bytes) in raw_bytes[displacement..].chunks_exact(4).enumerate() {
            let _local_offset = i * 4 + displacement;

            // let current_rom = rom + Size::new(local_offset as u32);
        }
    }

    pub fn preanalyze_gcc_except_table(
        &mut self,
        _settings: &SectionDataSettings,
        _raw_bytes: &[u8],
        _rom: RomAddress,
        _vram: Vram,
    ) {
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
    use super::*;

    #[pymethods]
    impl ContextBuilderFinderHeater {
        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderFinderHeaterOverlays {
            self.clone().process()
        }
    }
}
