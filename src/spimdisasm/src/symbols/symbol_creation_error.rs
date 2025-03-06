/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    context::OwnedSegmentNotFoundError,
    metadata::{segment_metadata::AddSymbolError, AddLabelError},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SymbolCreationError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    AddSymbol(AddSymbolError),
    AddLabel(AddLabelError),
}

impl fmt::Display for SymbolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolCreationError::OwnedSegmentNotFound(x) => {
                write!(f, "{}", x)
            }
            SymbolCreationError::AddSymbol(x) => write!(f, "{}", x),
            SymbolCreationError::AddLabel(x) => write!(f, "{}", x),
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
impl From<AddLabelError> for SymbolCreationError {
    fn from(value: AddLabelError) -> Self {
        SymbolCreationError::AddLabel(value)
    }
}
