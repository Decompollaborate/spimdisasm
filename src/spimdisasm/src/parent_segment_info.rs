/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{address_abstraction::Vram, metadata::OverlayCategoryName, rom_address::RomAddress};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ParentSegmentInfo {
    segment_rom: RomAddress,
    segment_vram: Vram,
    overlay_category_name: Option<OverlayCategoryName>,
}
impl ParentSegmentInfo {
    pub const fn new(
        segment_rom: RomAddress,
        segment_vram: Vram,
        overlay_category_name: Option<OverlayCategoryName>,
    ) -> Self {
        Self {
            segment_rom,
            segment_vram,
            overlay_category_name,
        }
    }

    pub const fn segment_rom(&self) -> RomAddress {
        self.segment_rom
    }
    pub const fn segment_vram(&self) -> Vram {
        self.segment_vram
    }
    pub const fn overlay_category_name(&self) -> Option<&OverlayCategoryName> {
        self.overlay_category_name.as_ref()
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl ParentSegmentInfo {
        #[new]
        // https://pyo3.rs/v0.23.2/function/signature.html#trailing-optional-arguments
        #[pyo3(signature = (segment_rom, segment_vram, overlay_category_name))]
        pub fn py_new(
            segment_rom: RomAddress,
            segment_vram: u32, // Vram,
            overlay_category_name: Option<OverlayCategoryName>,
        ) -> Self {
            Self::new(segment_rom, Vram::new(segment_vram), overlay_category_name)
        }
    }
}
