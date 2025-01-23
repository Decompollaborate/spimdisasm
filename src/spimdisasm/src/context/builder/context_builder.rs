/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{borrow::ToOwned, string::String, vec::Vec};

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

use super::{GlobalSegmentHeater, OverlaySegmentHeater, UserSegmentBuilder};

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilder {
    global_segment: SegmentMetadata,
    platform_segment: UserSegmentBuilder,
    overlays: Vec<SegmentMetadata>,
}

impl ContextBuilder {
    #[must_use]
    pub fn new(global_segment: GlobalSegmentHeater, platform_segment: UserSegmentBuilder) -> Self {
        Self {
            global_segment: global_segment.finish(),
            platform_segment,
            overlays: Vec::new(),
        }
    }

    pub fn add_overlay(&mut self, overlay: OverlaySegmentHeater) {
        // TODO: add checks like: unique overlay name, overlay's vram does not overlap with the global segment's vram
        self.overlays.push(overlay.finish());
    }

    #[must_use]
    fn get_visible_vram_ranges_for_segment(
        segment: &SegmentMetadata,
        overlays: &[SegmentMetadata],
    ) -> Vec<AddressRange<Vram>> {
        let mut all_ranges = vec![*segment.vram_range()];

        for other_name in segment.prioritised_overlays() {
            for other_overlay in overlays {
                if other_overlay.name() == Some(other_name) {
                    all_ranges.push(*other_overlay.vram_range());
                    break;
                }
            }
        }

        let mut overlapping_ranges = UnorderedSet::new();
        for (i, x) in all_ranges.iter().enumerate() {
            if x.overlaps(segment.vram_range()) {
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

        non_overlapping_ranges
    }

    #[must_use]
    pub fn build(self, global_config: GlobalConfig) -> Context {
        // A lot of the code in this function should probably be moved somewhere else, this is such a mess.

        let mut global_segment = self.global_segment;
        global_segment.set_visible_overlay_ranges(Self::get_visible_vram_ranges_for_segment(
            &global_segment,
            &self.overlays,
        ));

        let mut visible_ranges_for_overlays = Vec::new();
        for overlay in &self.overlays {
            visible_ranges_for_overlays.push(Self::get_visible_vram_ranges_for_segment(
                overlay,
                &self.overlays,
            ));
        }

        let mut overlays: Vec<SegmentMetadata> = self
            .overlays
            .into_iter()
            .zip(visible_ranges_for_overlays)
            .map(|(mut segment, visible_ranges)| {
                segment.set_visible_overlay_ranges(visible_ranges);
                segment
            })
            .collect();

        let mut new_references: UnorderedMap<String, Vec<ReferencedAddress>> = UnorderedMap::new();
        for overlay in &overlays {
            for (vram, reference) in overlay.preheater().references() {
                if overlay.is_vram_in_visible_overlay(*vram) {
                    let mut found = false;
                    for other_name in overlay.prioritised_overlays() {
                        for other_overlay in &overlays {
                            let ovl_name = other_overlay.name();
                            if ovl_name == Some(other_name) && other_overlay.in_vram_range(*vram) {
                                new_references
                                    .entry(ovl_name.expect("msg").to_owned())
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
        for overlay in &mut overlays {
            if let Some(name) = overlay.name() {
                if let Some(references_for_this_overlay) = new_references.remove(name) {
                    let references = overlay.preheater_mut().references_mut();
                    for reference in references_for_this_overlay {
                        let reference_vram = reference.vram();
                        let (new_reference, _) = references.find_mut_or_insert_with(
                            reference_vram,
                            FindSettings::new(true),
                            || {
                                if reference.user_declared() {
                                    (
                                        reference_vram,
                                        ReferencedAddress::new_user_declared(reference_vram),
                                    )
                                } else {
                                    (reference_vram, ReferencedAddress::new(reference_vram))
                                }
                            },
                        );

                        new_reference.set_from_other_reference(reference);
                    }
                }
            }
        }

        let mut grouped_segments: UnorderedMap<OverlayCategoryName, Vec<SegmentMetadata>> =
            UnorderedMap::new();

        for overlay in overlays {
            grouped_segments
                .entry(overlay.category_name().expect("How?").clone())
                .or_default()
                .push(overlay);
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

        Context::new(
            global_config,
            global_segment,
            self.platform_segment.build(),
            overlay_segments,
        )
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new(
            global_segment: GlobalSegmentHeater,
            platform_segment: UserSegmentBuilder,
        ) -> Self {
            Self::new(global_segment, platform_segment)
        }

        #[pyo3(name = "add_overlay")]
        pub fn py_add_overlay(&mut self, overlay: OverlaySegmentHeater) {
            self.add_overlay(overlay);
        }

        #[pyo3(name = "build")]
        pub fn py_build(&self, global_config: GlobalConfig) -> Context {
            // Silly clone because we can't move from a Python instance
            self.clone().build(global_config)
        }
    }
}
