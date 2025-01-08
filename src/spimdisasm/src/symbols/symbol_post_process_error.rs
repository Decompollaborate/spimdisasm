/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::context::OwnedSegmentNotFoundError;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OwnedSymbolNotFoundError {}

impl OwnedSymbolNotFoundError {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl fmt::Display for OwnedSymbolNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

impl error::Error for OwnedSymbolNotFoundError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SymbolPostProcessError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    OwnedSymbolNotFound(OwnedSymbolNotFoundError),
}

impl fmt::Display for SymbolPostProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{}", owned_segment_not_found_error)
            }
            SymbolPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found) => {
                write!(f, "{}", owned_symbol_not_found)
            }
        }
    }
}

impl error::Error for SymbolPostProcessError {}

impl From<OwnedSegmentNotFoundError> for SymbolPostProcessError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SymbolPostProcessError::OwnedSegmentNotFound(value)
    }
}

impl From<OwnedSymbolNotFoundError> for SymbolPostProcessError {
    fn from(value: OwnedSymbolNotFoundError) -> Self {
        SymbolPostProcessError::OwnedSymbolNotFound(value)
    }
}
