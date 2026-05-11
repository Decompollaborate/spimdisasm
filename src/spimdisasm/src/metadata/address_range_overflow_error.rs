/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

use address_space::{Rom, Size, Vram};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct AddressRangeOverflowError {
    rom: Rom,
    vram: Vram,
    size: Size,
    alignment: u32,
    name: Option<Arc<str>>,
}

impl AddressRangeOverflowError {
    pub(crate) fn new(
        rom: Rom,
        vram: Vram,
        size: Size,
        alignment: u32,
        name: Option<Arc<str>>,
    ) -> Self {
        Self {
            rom,
            vram,
            size,
            alignment,
            name,
        }
    }
}

impl fmt::Display for AddressRangeOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The size {} overflows the {:?} or {:?}. (alignment value 0x{:X})",
            self.size, self.rom, self.vram, self.alignment
        )?;
        if let Some(name) = &self.name {
            write!(f, " (on section {})", name)?;
        }
        Ok(())
    }
}

impl error::Error for AddressRangeOverflowError {}
