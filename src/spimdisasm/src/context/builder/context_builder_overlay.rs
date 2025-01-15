/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
};

use super::{ContextBuilderFinderHeaterOverlays, OverlaysBuilder};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderOverlay {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderOverlay {
    #[must_use]
    pub(crate) fn new(global_config: GlobalConfig, global_segment: SegmentMetadata) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments: UnorderedMap::new(),
        }
    }

    #[must_use]
    pub fn add_overlay_category(&mut self, category: OverlayCategoryName) -> OverlaysBuilder {
        OverlaysBuilder::new(category, &mut self.overlay_segments)
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
    impl ContextBuilderOverlay {
        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderFinderHeaterOverlays {
            self.clone().process()
        }
    }
}
