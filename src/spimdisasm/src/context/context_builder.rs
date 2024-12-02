/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    collections::btree_map::{self, BTreeMap},
    string::String,
    vec::Vec,
};
use rabbitizer::Vram;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    config::GlobalConfig,
    metadata::{GeneratedBy, OverlayCategoryName, SegmentMetadata, SymbolMetadata},
    rom_address::RomAddress,
    rom_vram_range::RomVramRange,
    sections::{SectionDataSettings, SectionTextSettings},
};

use super::{the_context::OverlayCategory, Context};

pub struct SegmentModifier<'a> {
    segment: &'a mut SegmentMetadata,
}

impl SegmentModifier<'_> {
    pub fn add_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_symbol(vram, rom, GeneratedBy::UserDeclared, None, false);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_function(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_function(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_branch_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_branch_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_jumptable(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_jumptable(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_jumptable_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_jumptable_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_gcc_except_table(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_gcc_except_table(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_gcc_except_table_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_gcc_except_table_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilder {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
}

impl ContextBuilder {
    #[must_use]
    pub fn new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
        let global_segment = SegmentMetadata::new(global_ranges, None);

        Self {
            global_config,
            global_segment,
        }
    }

    #[must_use]
    pub fn global_segment(&mut self) -> SegmentModifier {
        SegmentModifier {
            segment: &mut self.global_segment,
        }
    }

    #[must_use]
    pub fn process(self) -> ContextBuilderOverlay {
        ContextBuilderOverlay {
            global_config: self.global_config,
            global_segment: self.global_segment,
            overlay_segments: BTreeMap::new(),
        }
    }
}

pub struct OverlaysBuilder<'a> {
    name: OverlayCategoryName,
    entry: btree_map::Entry<'a, OverlayCategoryName, OverlayCategory>,
    overlays: Vec<SegmentMetadata>,
}

impl OverlaysBuilder<'_> {
    #[must_use]
    pub fn add_overlay(&mut self, ranges: RomVramRange) -> SegmentModifier {
        self.overlays
            .push(SegmentMetadata::new(ranges, Some(self.name.clone())));
        SegmentModifier {
            segment: self
                .overlays
                .last_mut()
                .expect("This shouldn't panic since we just added an element"),
        }
    }

    // TODO: proper error type
    #[allow(clippy::result_unit_err)]
    pub fn build(self) -> Result<(), ()> {
        if self.overlays.is_empty() {
            return Err(());
        }

        let mut segments = BTreeMap::new();

        let mut ranges = *self.overlays[0].rom_vram_range();

        for seg in self.overlays {
            ranges.expand_ranges(seg.rom_vram_range());
            segments.insert(seg.rom_range().start(), seg);
        }

        let placeholder_segment = SegmentMetadata::new(ranges, Some(self.name));
        self.entry
            .or_insert(OverlayCategory::new(placeholder_segment, segments));
        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderOverlay {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderOverlay {
    #[must_use]
    pub fn add_overlay_category(&mut self, category: OverlayCategoryName) -> OverlaysBuilder {
        OverlaysBuilder {
            name: category.clone(),
            entry: self.overlay_segments.entry(category),
            overlays: Vec::new(),
        }
    }

    #[must_use]
    pub fn process(self) -> ContextBuilderFinderHeater {
        ContextBuilderFinderHeater {
            global_config: self.global_config,
            global_segment: self.global_segment,
            overlay_segments: self.overlay_segments,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderFinderHeater {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderFinderHeater {
    pub fn preanalyze_text(
        &mut self,
        _settings: &SectionTextSettings,
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
        ContextBuilderFinderHeaterOverlays {
            global_config: self.global_config,
            global_segment: self.global_segment,
            overlay_segments: self.overlay_segments,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderFinderHeaterOverlays {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderFinderHeaterOverlays {
    #[must_use]
    pub fn build(self) -> Context {
        Context::new(
            self.global_config,
            self.global_segment,
            self.overlay_segments,
        )
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
            Self::new(global_config, global_ranges)
        }

        // TODO: add a way to add symbols
        // #[pyo3(name = "global_segment")]
        // pub fn py_global_segment(&mut self) -> SegmentModifier {
        //     self.global_segment()
        // }

        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderOverlay {
            // Silly clone because we can't move from a Python instance
            self.clone().process()
        }
    }

    #[pymethods]
    impl ContextBuilderOverlay {
        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderFinderHeater {
            self.clone().process()
        }
    }

    #[pymethods]
    impl ContextBuilderFinderHeater {
        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderFinderHeaterOverlays {
            self.clone().process()
        }
    }

    #[pymethods]
    impl ContextBuilderFinderHeaterOverlays {
        #[pyo3(name = "build")]
        pub fn py_build(&self) -> Context {
            self.clone().build()
        }
    }
}
