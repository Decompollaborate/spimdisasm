/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlayCategoryName {
    inner: String,
}

impl OverlayCategoryName {
    pub const fn new(name: String) -> Self {
        Self { inner: name }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl OverlayCategoryName {
        #[new]
        pub fn py_new(name: String) -> Self {
            Self::new(name)
        }
    }
}
