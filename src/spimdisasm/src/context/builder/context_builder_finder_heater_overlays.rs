/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    context::Context,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilderFinderHeaterOverlays {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
}

impl ContextBuilderFinderHeaterOverlays {
    pub(crate) fn new(
        global_config: GlobalConfig,

        global_segment: SegmentMetadata,
        overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
    ) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments,
        }
    }

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
    use super::*;

    #[pymethods]
    impl ContextBuilderFinderHeaterOverlays {
        #[pyo3(name = "build")]
        pub fn py_build(&self) -> Context {
            self.clone().build()
        }
    }
}
