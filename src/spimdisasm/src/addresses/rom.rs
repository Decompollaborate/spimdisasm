/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, ops};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::Size;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct Rom {
    inner: u32,
}

impl Rom {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }

    pub const fn inner(&self) -> u32 {
        self.inner
    }
}

impl Rom {
    pub const fn add_size(&self, size: &Size) -> Self {
        size.add_rom(self)
    }

    pub const fn sub_rom(&self, rhs: &Rom) -> Size {
        Size::new(self.inner - rhs.inner)
    }
}

impl ops::Sub<Rom> for Rom {
    type Output = Size;

    fn sub(self, rhs: Rom) -> Self::Output {
        self.sub_rom(&rhs)
    }
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rom {{ 0x{:08X} }}", self.inner)
    }
}

impl ops::Index<Rom> for [u8] {
    type Output = u8;

    #[inline]
    fn index(&self, idx: Rom) -> &Self::Output {
        &self[idx.inner as usize]
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl Rom {
        #[new]
        pub fn py_new(value: u32) -> Self {
            Self::new(value)
        }

        #[pyo3(name = "inner")]
        pub fn py_inner(&self) -> u32 {
            self.inner()
        }
    }
}
