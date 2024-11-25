/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    collections::btree_map::{self, BTreeMap},
    vec::Vec,
};
use rabbitizer::Vram;

use crate::{
    address_range::AddressRange,
    metadata::{GeneratedBy, OverlayCategoryName, SegmentMetadata, SymbolMetadata},
    rom_address::RomAddress,
};

use super::{context::OverlayCategory, Context, GlobalConfig};

pub struct SegmentModifier<'a> {
    segment: &'a mut SegmentMetadata,
}

impl<'a> SegmentModifier<'a> {
    pub fn add_symbol(&mut self, vram: Vram, rom: Option<RomAddress>) -> &mut SymbolMetadata {
        self.segment
            .add_symbol(vram, rom, GeneratedBy::UserDeclared, None, false)
    }
    pub fn add_global_function(
        &mut self,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        self.segment
            .add_function(vram, rom, GeneratedBy::UserDeclared)
    }
    //TODO: the rest
}

pub struct ContextBuilder {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
}

impl ContextBuilder {
    pub fn new(
        global_config: GlobalConfig,
        global_rom_range: AddressRange<RomAddress>,
        global_vram_range: AddressRange<Vram>,
    ) -> Self {
        let global_segment = SegmentMetadata::new(global_rom_range, global_vram_range, None);

        Self {
            global_config,
            global_segment,
        }
    }

    pub fn global_segment(&mut self) -> SegmentModifier {
        SegmentModifier {
            segment: &mut self.global_segment,
        }
    }

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

impl<'a> OverlaysBuilder<'a> {
    pub fn add_overlay(
        &mut self,
        rom_range: AddressRange<RomAddress>,
        vram_range: AddressRange<Vram>,
    ) -> SegmentModifier {
        self.overlays.push(SegmentMetadata::new(
            rom_range,
            vram_range,
            Some(self.name.clone()),
        ));
        SegmentModifier {
            segment: self
                .overlays
                .last_mut()
                .expect("This shouldn't panic since we just added an element"),
        }
    }

    pub fn build(self) {
        let mut segments = BTreeMap::new();

        let mut rom_range = *self.overlays[0].rom_range();
        let mut vram_range = *self.overlays[0].vram_range();

        for seg in self.overlays {
            rom_range.expand_range(seg.rom_range());
            vram_range.expand_range(seg.vram_range());
            segments.insert(seg.rom_range().start(), seg);
        }

        let placeholder_segment = SegmentMetadata::new(rom_range, vram_range, Some(self.name));
        self.entry
            .or_insert(OverlayCategory::new(placeholder_segment, segments));
    }
}

pub struct ContextBuilderOverlay {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderOverlay {
    /*
    pub fn add_overlay(&mut self, category: OverlayCategoryName,
        rom_range: AddressRange<RomAddress>,
        vram_range: AddressRange<Vram>,) -> SegmentModifier {

        let segment = self.overlay_segments
            .entry(
                category.clone()
            )
            .or_insert_with(
                || OverlayCategory::new(category, rom_range, vram_range)
            );

        SegmentModifier {
            segment
        }
    }
    */
    pub fn add_overlay_category(&mut self, category: OverlayCategoryName) -> OverlaysBuilder {
        OverlaysBuilder {
            name: category.clone(),
            entry: self.overlay_segments.entry(category),
            overlays: Vec::new(),
        }
    }

    pub fn process(self) -> ContextBuilderFinderHeater {
        ContextBuilderFinderHeater {
            global_config: self.global_config,
            global_segment: self.global_segment,
            overlay_segments: self.overlay_segments,
        }
    }
}

pub struct ContextBuilderFinderHeater {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderFinderHeater {
    pub fn process(self) -> ContextBuilderFinderHeaterOverlays {
        ContextBuilderFinderHeaterOverlays {
            global_config: self.global_config,
            global_segment: self.global_segment,
            overlay_segments: self.overlay_segments,
        }
    }
}

pub struct ContextBuilderFinderHeaterOverlays {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderFinderHeaterOverlays {
    pub fn build(self) -> Context {
        Context::new(
            self.global_config,
            self.global_segment,
            self.overlay_segments,
        )
    }
}
