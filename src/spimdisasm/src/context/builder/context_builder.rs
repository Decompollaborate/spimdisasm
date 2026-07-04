/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};

use addended_ordered_map::FindSettings;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    analysis::ReferencedAddress,
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    context::Context,
    metadata::SegmentMetadata,
    segments::{
        AddGlobalToBuilderError, AddOverlayToBuilderError, SegmentsCollection,
        SegmentsCollectionBuilder,
    },
};

use super::{AbsoluteSegmentBuilder, BuildContextError, GlobalSegmentHeater, OverlaySegmentHeater};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct ContextBuilder {
    collection_builder: SegmentsCollectionBuilder<GlobalSegmentHeater, OverlaySegmentHeater>,
}

impl ContextBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            collection_builder: SegmentsCollectionBuilder::new(),
        }
    }

    pub fn add_global_segment(
        &mut self,
        global_segment: GlobalSegmentHeater,
    ) -> Result<(), AddGlobalToBuilderError> {
        self.collection_builder.add_global_segment(global_segment)
    }

    pub fn add_overlay(
        &mut self,
        overlay: OverlaySegmentHeater,
    ) -> Result<(), AddOverlayToBuilderError> {
        self.collection_builder.add_overlay(overlay)
    }

    fn handle_overlay_references<ASEG>(
        segments_collections: &mut SegmentsCollection<SegmentMetadata, ASEG>,
    ) {
        let mut new_references: UnorderedMap<Arc<str>, Vec<ReferencedAddress>> =
            UnorderedMap::new();
        for (_, _, overlay) in segments_collections.overlay_segments().iter_segments() {
            let visible_ranges = overlay.visible_overlay_ranges();
            for (vram, reference) in overlay.preheater().references() {
                if visible_ranges.iter().any(|x| x.in_range(*vram)) {
                    let mut found = false;
                    for other_name in overlay.prioritised_overlays() {
                        for (_, _, other_overlay) in
                            segments_collections.overlay_segments().iter_segments()
                        {
                            let ovl_name = other_overlay.name();
                            if ovl_name == *other_name
                                && other_overlay.rom_vram_range().in_vram_range(*vram)
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

        for (_, _, overlay) in segments_collections
            .overlay_segments_mut()
            .iter_segments_mut()
        {
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
    }

    pub fn build(
        self,
        global_config: GlobalConfig,
        absolute_segment: AbsoluteSegmentBuilder,
    ) -> Result<Context, BuildContextError> {
        let Self { collection_builder } = self;

        let global_segments = collection_builder.global_segments();
        let overlays = collection_builder.overlays();

        if global_segments.is_empty() {
            return Err(BuildContextError::new_zero_global_segments());
        }

        let mut preheated_sections = UnorderedMap::new();
        for global_segment in global_segments {
            for (rom, _) in global_segment.preheated_sections_rom() {
                preheated_sections.insert(*rom, false);
            }
        }
        for overlay in overlays {
            for (rom, _) in overlay.preheated_sections_rom() {
                preheated_sections.insert(*rom, false);
            }
        }

        let mut segments_collections = collection_builder.build(
            absolute_segment.build(),
            SegmentMetadata::new_unknown_segment(),
        )?;

        Self::handle_overlay_references(&mut segments_collections);

        Ok(Context::new(
            global_config,
            segments_collections,
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
            absolute_segment: AbsoluteSegmentBuilder,
        ) -> Result<Context, BuildContextError> {
            // Silly clone because we can't move from a Python instance
            self.clone().build(global_config, absolute_segment)
        }
    }
}
