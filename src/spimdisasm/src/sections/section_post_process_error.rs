/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::Vram,
    context::OwnedSegmentNotFoundError,
    symbols::{OwnedSymbolNotFoundError, SymbolPostProcessError},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SectionPostProcessError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    OwnedSymbolNotFound(OwnedSymbolNotFoundError),

    #[cfg(feature = "pyo3")]
    AlreadyPostProcessed {
        name: String,
        vram_start: Vram,
        vram_end: Vram,
    },
    #[cfg(feature = "pyo3")]
    InvalidState(),
}

impl fmt::Display for SectionPostProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{}", owned_segment_not_found_error)
            }
            SectionPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found) => {
                write!(f, "{}", owned_symbol_not_found)
            }
            #[cfg(feature = "pyo3")]
            SectionPostProcessError::AlreadyPostProcessed {
                name,
                vram_start,
                vram_end,
            } => {
                write!(
                    f,
                    "The section {} ({:?} {:?}) has already been post-processed.",
                    name, vram_start, vram_end
                )
            }
            #[cfg(feature = "pyo3")]
            SectionPostProcessError::InvalidState() => {
                write!(f, "This section is somehow in an invalid state.")
            }
        }
    }
}

impl error::Error for SectionPostProcessError {}

impl From<SymbolPostProcessError> for SectionPostProcessError {
    fn from(value: SymbolPostProcessError) -> Self {
        match value {
            SymbolPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                SectionPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error)
            }
            SymbolPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found) => {
                SectionPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found)
            }
        }
    }
}

impl From<OwnedSegmentNotFoundError> for SectionPostProcessError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SectionPostProcessError::OwnedSegmentNotFound(value)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, SectionPostProcessError, PyRuntimeError);

    impl std::convert::From<super::SectionPostProcessError> for PyErr {
        fn from(err: super::SectionPostProcessError) -> PyErr {
            SectionPostProcessError::new_err(err.to_string())
        }
    }
}
