/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Vram},
    analysis::ReferencedAddress,
    collections::{
        addended_ordered_map::FindSettings, unordered_map::UnorderedMap,
        unordered_set::UnorderedSet,
    },
    config::GlobalConfig,
    context::Context,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
};

use super::{
    AddGlobalToBuilderError, AddOverlayToBuilderError, BuildContextError, GlobalSegmentHeater,
    OverlaySegmentHeater, SegmentHeater, UserSegmentBuilder,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct ContextBuilder {
    global_segments: Vec<GlobalSegmentHeater>,
    overlays: Vec<OverlaySegmentHeater>,
    segment_names: UnorderedSet<Arc<str>>,
}

impl ContextBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            global_segments: Vec::new(),
            overlays: Vec::new(),
            segment_names: UnorderedSet::new(),
        }
    }

    pub fn add_global_segment(
        &mut self,
        global_segment: GlobalSegmentHeater,
    ) -> Result<(), AddGlobalToBuilderError> {
        // We can't let a global segment overlap with anything else
        let self_ranges = global_segment.ranges();
        let self_rom = self_ranges.rom();
        let self_vram = self_ranges.vram();

        for g in &self.global_segments {
            let other_ranges = g.ranges();
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

        for o in &self.overlays {
            let other_ranges = o.ranges();
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

        if !self.segment_names.insert(global_segment.name()) {
            Err(AddGlobalToBuilderError::new_duplicated_name(
                global_segment.name(),
            ))
        } else {
            self.global_segments.push(global_segment);
            Ok(())
        }
    }

    pub fn add_overlay(
        &mut self,
        overlay: OverlaySegmentHeater,
    ) -> Result<(), AddOverlayToBuilderError> {
        let self_ranges = overlay.ranges();
        let self_rom = self_ranges.rom();
        let self_vram = self_ranges.vram();

        // We let overlays overlap with other overlays.
        // but overlapping with global segments is prohibited.
        for g in &self.global_segments {
            let other_ranges = g.ranges();
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

    fn get_visible_vram_ranges_for_segment(
        segment_name: Arc<str>,
        segment: &SegmentHeater,
        overlays: &[OverlaySegmentHeater],
    ) -> Result<Vec<AddressRange<Vram>>, BuildContextError> {
        // We need to know all the Vram ranges this overlay is able to see
        let mut all_ranges = vec![*segment.ranges().vram()];
        for other_name in segment.prioritised_overlays() {
            if let Some(other_overlay) = overlays
                .iter()
                .find(|other_overlay| &other_overlay.name() == other_name)
            {
                all_ranges.push(*other_overlay.ranges().vram());
            } else {
                return Err(BuildContextError::new_missing_prioritised_overlay(
                    segment_name,
                    other_name.clone(),
                ));
            }
        }

        let mut overlapping_ranges = UnorderedSet::new();
        for (i, x) in all_ranges.iter().enumerate() {
            if x.overlaps(segment.ranges().vram()) {
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

    fn build_overlays(
        overlays: Vec<OverlaySegmentHeater>,
    ) -> Result<UnorderedMap<OverlayCategoryName, OverlayCategory>, BuildContextError> {
        let mut visible_ranges_for_overlays = Vec::new();
        for overlay in &overlays {
            visible_ranges_for_overlays.push(Self::get_visible_vram_ranges_for_segment(
                overlay.name(),
                overlay.inner(),
                &overlays,
            )?);
        }

        let mut overlays: Vec<(OverlaySegmentHeater, Vec<AddressRange<Vram>>)> = overlays
            .into_iter()
            .zip(visible_ranges_for_overlays)
            .collect();

        let mut new_references: UnorderedMap<Arc<str>, Vec<ReferencedAddress>> =
            UnorderedMap::new();
        for (overlay, visible_ranges) in &overlays {
            for (vram, reference) in overlay.preheater().references() {
                if visible_ranges.iter().any(|x| x.in_range(*vram)) {
                    let mut found = false;
                    for other_name in overlay.prioritised_overlays() {
                        for (other_overlay, _) in &overlays {
                            let ovl_name = other_overlay.name();
                            if ovl_name == *other_name
                                && other_overlay.ranges().in_vram_range(*vram)
                            {
                                new_references
                                    .entry(ovl_name)
                                    .or_default()
                                    .push(reference.clone());
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            }
        }
        for (overlay, _) in &mut overlays {
            if let Some(references_for_this_overlay) = new_references.remove(&overlay.name()) {
                let references = overlay.preheater_mut().references_mut();
                for reference in references_for_this_overlay {
                    let reference_vram = reference.vram();
                    let (new_reference, _) = references.find_mut_or_insert_with(
                        reference_vram,
                        FindSettings::new(true),
                        || {
                            if reference.user_declared() {
                                ReferencedAddress::new_user_declared(reference_vram)
                            } else {
                                ReferencedAddress::new(reference_vram)
                            }
                        },
                    );

                    new_reference.set_from_other_reference(reference);
                }
            }
        }

        let mut grouped_segments: UnorderedMap<OverlayCategoryName, Vec<SegmentMetadata>> =
            UnorderedMap::new();

        for (overlay, visible_ranges) in overlays {
            grouped_segments
                .entry(overlay.category_name().clone())
                .or_default()
                .push(overlay.finish(visible_ranges.into()));
        }

        let mut overlay_segments = UnorderedMap::new();
        for (name, overlays) in grouped_segments {
            // TODO: move the body of this loop to OverlayCategory::new?
            let mut segments = UnorderedMap::new();
            let mut ranges = *overlays[0].rom_vram_range();

            for seg in overlays {
                ranges.expand_ranges(seg.rom_vram_range());
                segments.insert(seg.rom_range().start(), seg);
            }

            overlay_segments.insert(name.clone(), OverlayCategory::new(name, ranges, segments));
        }

        Ok(overlay_segments)
    }

    pub fn build(
        self,
        global_config: GlobalConfig,
        user_segment: UserSegmentBuilder,
    ) -> Result<Context, BuildContextError> {
        if self.global_segments.is_empty() {
            return Err(BuildContextError::new_zero_global_segments());
        }

        let mut preheated_sections = UnorderedMap::new();
        for global_segment in &self.global_segments {
            for (rom, _) in global_segment.preheated_sections_rom() {
                preheated_sections.insert(*rom, false);
            }
        }
        for overlay in &self.overlays {
            for (rom, _) in overlay.preheated_sections_rom() {
                preheated_sections.insert(*rom, false);
            }
        }

        let temp: Result<Vec<SegmentMetadata>, BuildContextError> = self
            .global_segments
            .into_iter()
            .map(|seg| {
                let visible_ranges_for_global = Self::get_visible_vram_ranges_for_segment(
                    seg.name(),
                    seg.inner(),
                    &self.overlays,
                )?;
                Ok(seg.finish(visible_ranges_for_global.into()))
            })
            .collect();
        let global_segments = temp?;

        let overlay_segments = Self::build_overlays(self.overlays)?;

        Ok(Context::new(
            global_config,
            user_segment.build(),
            global_segments,
            overlay_segments,
            preheated_sections,
        ))
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new() -> Self {
            Self::new()
        }

        #[pyo3(name = "add_global_segment")]
        pub fn py_add_global_segment(
            &mut self,
            global_segment: GlobalSegmentHeater,
        ) -> Result<(), AddGlobalToBuilderError> {
            self.add_global_segment(global_segment)
        }

        #[pyo3(name = "add_overlay")]
        pub fn py_add_overlay(
            &mut self,
            overlay: OverlaySegmentHeater,
        ) -> Result<(), AddOverlayToBuilderError> {
            self.add_overlay(overlay)
        }

        #[pyo3(name = "build")]
        pub fn py_build(
            &self,
            global_config: GlobalConfig,
            user_segment: UserSegmentBuilder,
        ) -> Result<Context, BuildContextError> {
            // Silly clone because we can't move from a Python instance
            self.clone().build(global_config, user_segment)
        }
    }
}
