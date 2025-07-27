/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Vram};

use super::LabelType;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct AddLabelError {
    label_vram: Vram,
    label_type: LabelType,
    segment_name: Option<Arc<str>>,
    segment_ranges: AddressRange<Vram>,
}

impl AddLabelError {
    pub(crate) fn new_vram_out_of_range(
        label_vram: Vram,
        label_type: LabelType,
        segment_name: Option<Arc<str>>,
        segment_ranges: AddressRange<Vram>,
    ) -> Self {
        Self {
            label_vram,
            label_type,
            segment_name,
            segment_ranges,
        }
    }
}

impl fmt::Display for AddLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when trying to add label to ")?;
        if let Some(name) = &self.segment_name {
            write!(f, "overlay segment '{name}'")?;
        } else {
            write!(f, "global segment")?;
        }
        write!(f, ": ")?;
        write!(
            f,
            "Label's vram 0x{} is out of the segment's range `{}`",
            self.label_vram, self.segment_ranges
        )
    }
}

impl error::Error for AddLabelError {}
