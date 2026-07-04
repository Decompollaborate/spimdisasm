/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};

use address_space::{AddressRange, Vram};

use crate::{
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    segments::SegBuilder,
};

use super::{
    AbsoluteSegment, AddGlobalToBuilderError, AddOverlayToBuilderError,
    BuildSegmentsCollectionError, GlobalOvlBuilder, GlobalSegBuilder, OverlayCategory,
    OverlayCategoryName, OverlaysCollection, Segment, SegmentsCollection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentsCollectionBuilder<PRESEG, PREOVL> {
    global_segments: Vec<PRESEG>,
    overlays: Vec<PREOVL>,
    segment_names: UnorderedSet<Arc<str>>,
}

impl<PRESEG, PREOVL> SegmentsCollectionBuilder<PRESEG, PREOVL> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            global_segments: Vec::new(),
            overlays: Vec::new(),
            segment_names: UnorderedSet::new(),
        }
    }

    #[must_use]
    pub fn global_segments(&self) -> &[PRESEG] {
        &self.global_segments
    }

    #[must_use]
    pub fn overlays(&self) -> &[PREOVL] {
        &self.overlays
    }

    #[must_use]
    pub fn segment_names(&self) -> &UnorderedSet<Arc<str>> {
        &self.segment_names
    }
}

impl<PRESEG, PREOVL> SegmentsCollectionBuilder<PRESEG, PREOVL>
where
    PRESEG: GlobalSegBuilder,
    PREOVL: GlobalOvlBuilder,
{
    pub fn add_global_segment(
        &mut self,
        global_segment: PRESEG,
    ) -> Result<(), AddGlobalToBuilderError> {
        // We can't let a global segment overlap with anything else
        let self_ranges = global_segment.rom_vram_range();
        let self_rom = self_ranges.rom();
        let self_vram = self_ranges.vram();

        // Check this doesn't overlap with other global segments
        for g in &self.global_segments {
            let other_ranges = g.rom_vram_range();
            let other_rom = other_ranges.rom();
            let other_vram = other_ranges.vram();

            if other_rom.overlaps(self_rom) {
                return Err(AddGlobalToBuilderError::new_global_overlapping_rom(
                    global_segment.name(),
                    *self_rom,
                    g.name(),
                    *other_rom,
                ));
            } else if other_vram.overlaps(self_vram) {
                return Err(AddGlobalToBuilderError::new_global_overlapping_vram(
                    global_segment.name(),
                    *self_vram,
                    g.name(),
                    *other_vram,
                ));
            }
        }

        // Check this doesn't overlap with overlays segments
        for o in &self.overlays {
            let other_ranges = o.rom_vram_range();
            let other_rom = other_ranges.rom();
            let other_vram = other_ranges.vram();

            if other_rom.overlaps(self_rom) {
                return Err(AddGlobalToBuilderError::new_overlay_overlapping_rom(
                    global_segment.name(),
                    *self_rom,
                    o.name(),
                    *other_rom,
                ));
            } else if other_vram.overlaps(self_vram) {
                return Err(AddGlobalToBuilderError::new_overlay_overlapping_vram(
                    global_segment.name(),
                    *self_vram,
                    o.name(),
                    *other_vram,
                ));
            }
        }

        // Segment names must be unique
        if !self.segment_names.insert(global_segment.name()) {
            Err(AddGlobalToBuilderError::new_duplicated_name(
                global_segment.name(),
            ))
        } else {
            self.global_segments.push(global_segment);
            Ok(())
        }
    }

    pub fn add_overlay(&mut self, overlay: PREOVL) -> Result<(), AddOverlayToBuilderError> {
        let self_ranges = overlay.rom_vram_range();
        let self_rom = self_ranges.rom();
        let self_vram = self_ranges.vram();

        // We let overlays overlap with other overlays.
        // but overlapping with global segments is prohibited.
        for g in &self.global_segments {
            let other_ranges = g.rom_vram_range();
            let other_rom = other_ranges.rom();
            let other_vram = other_ranges.vram();

            if other_rom.overlaps(self_rom) {
                return Err(AddOverlayToBuilderError::new_overlapping_rom(
                    overlay.name(),
                    *self_rom,
                    g.name(),
                    *other_rom,
                ));
            } else if other_vram.overlaps(self_vram) {
                return Err(AddOverlayToBuilderError::new_overlapping_vram(
                    overlay.name(),
                    *self_vram,
                    g.name(),
                    *other_vram,
                ));
            }
        }

        if !self.segment_names.insert(overlay.name()) {
            Err(AddOverlayToBuilderError::new_duplicated_name(
                overlay.name(),
            ))
        } else {
            self.overlays.push(overlay);
            Ok(())
        }
    }
}

impl<PRESEG, PREOVL, SEG> SegmentsCollectionBuilder<PRESEG, PREOVL>
where
    PRESEG: GlobalSegBuilder<Finished = SEG>,
    PREOVL: GlobalOvlBuilder<Finished = SEG>,
    SEG: Segment,
{
    pub fn build<ASEG>(
        self,
        absolute_segment: ASEG,
        unknown_segment: SEG,
    ) -> Result<SegmentsCollection<SEG, ASEG>, BuildSegmentsCollectionError>
    where
        ASEG: AbsoluteSegment,
    {
        let Self {
            global_segments,
            overlays,
            segment_names: _,
        } = self;

        if global_segments.is_empty() {
            return Err(BuildSegmentsCollectionError::new_zero_global_segments());
        }

        let global_segments = Self::build_globals(global_segments, &overlays)?;
        let overlay_segments = Self::build_overlays(overlays)?;

        let segments_collections = SegmentsCollection::new(
            absolute_segment,
            global_segments,
            overlay_segments,
            unknown_segment,
        );

        Ok(segments_collections)
    }

    fn build_globals(
        global_segments: Vec<PRESEG>,
        overlays: &[PREOVL],
    ) -> Result<Vec<SEG>, BuildSegmentsCollectionError> {
        global_segments
            .into_iter()
            .map(|seg| {
                Self::get_visible_vram_ranges_for_segment(seg.name(), &seg, overlays)
                    .map(|visible_ranges_for_global| seg.finish(visible_ranges_for_global))
            })
            .collect()
    }

    fn build_overlays(
        overlays: Vec<PREOVL>,
    ) -> Result<OverlaysCollection<SEG>, BuildSegmentsCollectionError> {
        let temp: Result<Vec<_>, _> = overlays
            .iter()
            .map(|overlay| {
                Self::get_visible_vram_ranges_for_segment(overlay.name(), overlay, &overlays)
            })
            .collect();
        let visible_ranges_for_overlays = temp?;

        let mut grouped_segments: UnorderedMap<OverlayCategoryName, Vec<SEG>> = UnorderedMap::new();
        for (overlay, visible_ranges) in overlays.into_iter().zip(visible_ranges_for_overlays) {
            grouped_segments
                .entry(overlay.category_name())
                .or_default()
                .push(overlay.finish(visible_ranges));
        }

        let mut overlay_segments = UnorderedMap::new();
        for (name, overlays) in grouped_segments {
            // TODO: move the body of this loop to OverlayCategory::new?
            let mut segments = UnorderedMap::new();
            let mut ranges = *overlays[0].rom_vram_range();

            for seg in overlays {
                ranges.expand_ranges(seg.rom_vram_range());
                segments.insert(seg.rom_vram_range().rom().start(), seg);
            }

            overlay_segments.insert(name.clone(), OverlayCategory::new(name, ranges, segments));
        }

        Ok(OverlaysCollection::new(overlay_segments))
    }

    fn get_visible_vram_ranges_for_segment(
        segment_name: Arc<str>,
        segment: &dyn SegBuilder,
        overlays: &[PREOVL],
    ) -> Result<Vec<AddressRange<Vram>>, BuildSegmentsCollectionError> {
        // We need to know all the Vram ranges this overlay is able to see
        let mut all_ranges = vec![*segment.rom_vram_range().vram()];
        for other_name in segment.prioritised_overlays() {
            if let Some(other_overlay) = overlays
                .iter()
                .find(|other_overlay| &other_overlay.name() == other_name)
            {
                all_ranges.push(*other_overlay.rom_vram_range().vram());
            } else {
                return Err(
                    BuildSegmentsCollectionError::new_missing_prioritised_overlay(
                        segment_name,
                        other_name.clone(),
                    ),
                );
            }
        }

        let mut overlapping_ranges = UnorderedSet::new();
        for (i, x) in all_ranges.iter().enumerate() {
            if x.overlaps(segment.rom_vram_range().vram()) {
                overlapping_ranges.insert(*x);
                continue;
            }
            for (j, y) in all_ranges.iter().enumerate() {
                if i == j {
                    continue;
                }
                if x.overlaps(y) {
                    overlapping_ranges.insert(*x);
                    overlapping_ranges.insert(*y);
                }
            }
        }

        let mut non_overlapping_ranges = Vec::new();
        for x in all_ranges {
            if !overlapping_ranges.contains(&x) {
                non_overlapping_ranges.push(x);
            }
        }

        Ok(non_overlapping_ranges)
    }
}

impl<PRESEG, PREOVL> Default for SegmentsCollectionBuilder<PRESEG, PREOVL> {
    fn default() -> Self {
        Self::new()
    }
}
