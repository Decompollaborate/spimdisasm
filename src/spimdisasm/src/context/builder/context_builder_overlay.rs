/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::BTreeMap;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
};

use super::{ContextBuilderFinderHeater, OverlaysBuilder};

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderOverlay {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderOverlay {
    #[must_use]
    pub(crate) fn new(global_config: GlobalConfig, global_segment: SegmentMetadata) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn add_overlay_category(&mut self, category: OverlayCategoryName) -> OverlaysBuilder {
        OverlaysBuilder::new(category, &mut self.overlay_segments)
    }

    #[must_use]
    pub fn process(self) -> ContextBuilderFinderHeater {
        ContextBuilderFinderHeater::new(
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
        pub fn py_process(&self) -> ContextBuilderFinderHeater {
            self.clone().process()
        }
    }
}
