/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AddPlatformSymbolError {}
impl fmt::Display for AddPlatformSymbolError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
impl error::Error for AddPlatformSymbolError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddPlatformSymbolError, PyRuntimeError);

    impl std::convert::From<super::AddPlatformSymbolError> for PyErr {
        fn from(err: super::AddPlatformSymbolError) -> PyErr {
            AddPlatformSymbolError::new_err(err.to_string())
        }
    }
}
