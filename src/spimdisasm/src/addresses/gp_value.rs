/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GpValue {
    inner: u32,
}

impl GpValue {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }

    pub const fn inner(&self) -> u32 {
        self.inner
    }
}

impl fmt::Debug for GpValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GpValue {{ 0x{:08X} }}", self.inner)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl GpValue {
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
