/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{config::GlobalConfig, metadata::SegmentMetadata, rom_vram_range::RomVramRange};

use super::{ContextBuilderOverlay, SegmentModifier};

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilder {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
}

impl ContextBuilder {
    #[must_use]
    pub fn new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
        let global_segment = SegmentMetadata::new(global_ranges, None);

        Self {
            global_config,
            global_segment,
        }
    }

    #[must_use]
    pub fn global_segment(&mut self) -> SegmentModifier {
        SegmentModifier::new(&mut self.global_segment)
    }

    #[must_use]
    pub fn process(self) -> ContextBuilderOverlay {
        ContextBuilderOverlay::new(self.global_config, self.global_segment)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
            Self::new(global_config, global_ranges)
        }

        // TODO: add a way to add symbols
        // #[pyo3(name = "global_segment")]
        // pub fn py_global_segment(&mut self) -> SegmentModifier {
        //     self.global_segment()
        // }

        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderOverlay {
            // Silly clone because we can't move from a Python instance
            self.clone().process()
        }
    }
}
