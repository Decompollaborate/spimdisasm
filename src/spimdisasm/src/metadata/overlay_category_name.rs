/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlayCategoryName {
    inner: Arc<str>,
}

impl OverlayCategoryName {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self { inner: name.into() }
    }

    pub fn inner(&self) -> Arc<str> {
        self.inner.clone()
    }
}

impl fmt::Display for OverlayCategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
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
