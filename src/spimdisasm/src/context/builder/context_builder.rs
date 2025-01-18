/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    context::Context,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
};

use super::{GlobalSegmentHeater, OverlaySegmentHeater};

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilder {
    global_segment: SegmentMetadata,
    overlays: Vec<SegmentMetadata>,
}

impl ContextBuilder {
    #[must_use]
    pub fn new(global_segment: GlobalSegmentHeater) -> Self {
        Self {
            global_segment: global_segment.finish(),
            overlays: Vec::new(),
        }
    }

    pub fn add_overlay(&mut self, overlay: OverlaySegmentHeater) {
        self.overlays.push(overlay.finish());
    }

    #[must_use]
    pub fn build(self, global_config: GlobalConfig) -> Context {
        let mut grouped_segments: UnorderedMap<OverlayCategoryName, Vec<SegmentMetadata>> =
            UnorderedMap::new();

        for overlay in self.overlays {
            grouped_segments
                .entry(overlay.category_name().expect("How?").clone())
                .or_default()
                .push(overlay);
        }

        let mut overlay_segments = UnorderedMap::new();
        for (name, overlays) in grouped_segments {
            let mut segments = UnorderedMap::new();
            let mut ranges = *overlays[0].rom_vram_range();

            for seg in overlays {
                ranges.expand_ranges(seg.rom_vram_range());
                segments.insert(seg.rom_range().start(), seg);
            }

            let placeholder_segment =
                SegmentMetadata::new_overlay(ranges, name.clone(), format!("{}_placeholder", name));
            overlay_segments.insert(name, OverlayCategory::new(placeholder_segment, segments));
        }

        Context::new(global_config, self.global_segment, overlay_segments)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new(global_segment: GlobalSegmentHeater) -> Self {
            Self::new(global_segment)
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
