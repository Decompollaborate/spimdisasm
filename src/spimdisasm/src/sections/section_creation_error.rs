/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Vram},
    context::OwnedSegmentNotFoundError,
    metadata::segment_metadata::AddSymbolError,
    symbols::SymbolCreationError,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SectionCreationError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    AddSymbol(AddSymbolError),
    EmptySection {
        name: String,
        vram: Vram,
    },
    BadBytesSize {
        name: String,
        size: usize,
        multiple_of: usize,
    },
    UnalignedVram {
        name: String,
        vram: Vram,
        multiple_of: usize,
    },
    UnalignedRom {
        name: String,
        rom: Rom,
        multiple_of: usize,
    },
    RomVramAlignmentMismatch {
        name: String,
        rom: Rom,
        vram: Vram,
        multiple_of: usize,
    },
}

impl fmt::Display for SectionCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionCreationError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{}", owned_segment_not_found_error)
            }
            SectionCreationError::AddSymbol(add_symbol_error) => write!(f, "{}", add_symbol_error),
            SectionCreationError::EmptySection { name, vram } => write!(f, "Can't initialize section '{}' ({:?}) with empty bytes.", name, vram),
            SectionCreationError::BadBytesSize { name, size, multiple_of} => write!(f, "Can't create section {} because the bytes length (0x{:X}) is not a multiple of 0x{:X}.", name, size, multiple_of),
            SectionCreationError::UnalignedVram { name, vram, multiple_of} => write!(f, "Can't create section {} because the vram ({:?}) is not aligned to 0x{:X}.", name, vram, multiple_of),
            SectionCreationError::UnalignedRom { name, rom, multiple_of} => write!(f, "Can't create section {} because the rom (0x{:X}) is not aligned to 0x{:X}.", name, rom.inner(), multiple_of),
            SectionCreationError::RomVramAlignmentMismatch {name, rom, vram, multiple_of} => write!(f, "Can't create section {} because the alignment of its rom ({:?}) and vram ({:?}) mod {} does not match.", name, rom, vram, multiple_of),
        }
    }
}

impl error::Error for SectionCreationError {}

impl From<SymbolCreationError> for SectionCreationError {
    fn from(value: SymbolCreationError) -> Self {
        match value {
            SymbolCreationError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                SectionCreationError::OwnedSegmentNotFound(owned_segment_not_found_error)
            }
            SymbolCreationError::AddSymbol(add_symbol_error) => {
                SectionCreationError::AddSymbol(add_symbol_error)
            }
        }
    }
}

impl From<OwnedSegmentNotFoundError> for SectionCreationError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SectionCreationError::OwnedSegmentNotFound(value)
    }
}

impl From<AddSymbolError> for SectionCreationError {
    fn from(value: AddSymbolError) -> Self {
        SectionCreationError::AddSymbol(value)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, SectionCreationError, PyRuntimeError);

    impl std::convert::From<super::SectionCreationError> for PyErr {
        fn from(err: super::SectionCreationError) -> PyErr {
            SectionCreationError::new_err(err.to_string())
        }
    }
}
