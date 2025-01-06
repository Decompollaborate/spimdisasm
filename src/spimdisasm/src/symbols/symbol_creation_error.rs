/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{context::OwnedSegmentNotFoundError, metadata::segment_metadata::AddSymbolError};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SymbolCreationError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    AddSymbol(AddSymbolError),
}

impl fmt::Display for SymbolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolCreationError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{}", owned_segment_not_found_error)
            }
            SymbolCreationError::AddSymbol(add_symbol_error) => write!(f, "{}", add_symbol_error),
        }
    }
}

impl error::Error for SymbolCreationError {}

impl From<OwnedSegmentNotFoundError> for SymbolCreationError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SymbolCreationError::OwnedSegmentNotFound(value)
    }
}

impl From<AddSymbolError> for SymbolCreationError {
    fn from(value: AddSymbolError) -> Self {
        SymbolCreationError::AddSymbol(value)
    }
}
