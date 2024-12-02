/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, ops};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::size::Size;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct RomAddress {
    inner: u32,
}

impl RomAddress {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }

    pub const fn inner(&self) -> u32 {
        self.inner
    }
}

impl RomAddress {
    pub const fn add_size(&self, size: &Size) -> Self {
        size.add_rom(self)
    }

    pub const fn sub_rom(&self, rhs: &RomAddress) -> Size {
        Size::new(self.inner - rhs.inner)
    }
}

impl ops::Sub<RomAddress> for RomAddress {
    type Output = Size;

    fn sub(self, rhs: RomAddress) -> Self::Output {
        self.sub_rom(&rhs)
    }
}

impl fmt::Debug for RomAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RomAddress {{ 0x{:08X} }}", self.inner)
    }
}

impl ops::Index<RomAddress> for [u8] {
    type Output = u8;

    #[inline]
    fn index(&self, idx: RomAddress) -> &Self::Output {
        &self[idx.inner as usize]
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::prelude::*;

    use super::*;

    #[pymethods]
    impl RomAddress {
        #[new]
        pub fn py_new(value: u32) -> Self {
            Self::new(value)
        }
    }
}
