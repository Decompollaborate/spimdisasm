/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::GpValue;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GpConfig {
    gp_value: GpValue,
    pic: bool,
}

impl GpConfig {
    #[must_use]
    pub const fn new_sdata(gp_value: GpValue) -> Self {
        Self {
            gp_value,
            pic: false,
        }
    }
    #[must_use]
    pub fn new_pic(gp_value: GpValue) -> Self {
        Self {
            gp_value,
            pic: true,
        }
    }

    #[must_use]
    pub(crate) const fn gp_value(&self) -> GpValue {
        self.gp_value
    }

    #[must_use]
    pub(crate) const fn pic(&self) -> bool {
        self.pic
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl GpConfig {
        #[pyo3(name = "new_pic")]
        #[staticmethod]
        pub fn py_new_pic(gp_value: GpValue) -> Self {
            Self::new_pic(gp_value)
        }
        #[pyo3(name = "new_sdata")]
        #[staticmethod]
        pub fn py_new_sdata(gp_value: GpValue) -> Self {
            Self::new_sdata(gp_value)
        }
    }
}
