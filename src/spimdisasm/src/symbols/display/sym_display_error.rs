/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::context::OwnedSegmentNotFoundError;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SymDisplayError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    SelfSymNotFound(),
}

impl fmt::Display for SymDisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymDisplayError::OwnedSegmentNotFound(x) => write!(f, "{}", x),
            SymDisplayError::SelfSymNotFound() => {
                // TODO: more info
                write!(f, "Can't find symbol")
            }
        }
    }
}
impl error::Error for SymDisplayError {}

impl From<OwnedSegmentNotFoundError> for SymDisplayError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SymDisplayError::OwnedSegmentNotFound(value)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, SymDisplayError, PyRuntimeError);

    pyo3::create_exception!(spimdisasm, OwnedSegmentNotFound, SymDisplayError);
    pyo3::create_exception!(spimdisasm, SelfSymNotFound, SymDisplayError);

    impl std::convert::From<super::SymDisplayError> for PyErr {
        fn from(err: super::SymDisplayError) -> PyErr {
            match err {
                super::SymDisplayError::OwnedSegmentNotFound(..) => {
                    OwnedSegmentNotFound::new_err(err.to_string())
                }
                super::SymDisplayError::SelfSymNotFound(..) => {
                    SelfSymNotFound::new_err(err.to_string())
                }
            }
        }
    }
}
