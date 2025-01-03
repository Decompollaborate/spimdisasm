/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;

use crate::{
    addresses::RomVramRange,
    collections::unordered_map::{self, UnorderedMap},
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
};

use super::SegmentModifier;

pub struct OverlaysBuilder<'map_entry> {
    name: OverlayCategoryName,
    entry: unordered_map::Entry<'map_entry, OverlayCategoryName, OverlayCategory>,
    overlays: Vec<SegmentMetadata>,
}

impl<'map_entry> OverlaysBuilder<'map_entry> {
    pub(crate) fn new(
        category: OverlayCategoryName,
        overlay_segments: &'map_entry mut UnorderedMap<OverlayCategoryName, OverlayCategory>,
    ) -> Self {
        Self {
            name: category.clone(),
            entry: overlay_segments.entry(category),
            overlays: Vec::new(),
        }
    }
}

impl OverlaysBuilder<'_> {
    #[must_use]
    pub fn add_overlay(&mut self, ranges: RomVramRange) -> SegmentModifier {
        // TODO: ensure an overlay doesn't overlap the global segment.
        self.overlays
            .push(SegmentMetadata::new(ranges, Some(self.name.clone())));
        SegmentModifier::new(
            self.overlays
                .last_mut()
                .expect("This shouldn't panic since we just added an element"),
        )
    }

    // TODO: proper error type
    #[allow(clippy::result_unit_err)]
    pub fn build(self) -> Result<(), ()> {
        if self.overlays.is_empty() {
            return Err(());
        }

        let mut segments = UnorderedMap::new();

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
