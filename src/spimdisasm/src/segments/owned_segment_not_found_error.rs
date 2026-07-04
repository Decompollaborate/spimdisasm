/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::ParentSegmentInfo;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct OwnedSegmentNotFoundError {
    info: ParentSegmentInfo,
}

impl OwnedSegmentNotFoundError {
    pub(crate) fn new(info: ParentSegmentInfo) -> Self {
        Self { info }
    }
}

impl fmt::Display for OwnedSegmentNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: more info
        write!(f, "Can't find owned segment for ")?;
        if let Some(overlay_name) = self.info.overlay_category_name() {
            write!(f, "overlay '{overlay_name}'")?;
        } else {
            write!(f, "global segment")?;
        }
        write!(
            f,
            ". Addresses of the expected parent segment: Rom: 0x{:08X}, Vram: 0x{:08X}",
            self.info.segment_rom().inner(),
            self.info.segment_vram().inner()
        )?;

        Ok(())
    }
}

impl error::Error for OwnedSegmentNotFoundError {}
