/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::string::String;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, Size, Vram},
    context::OwnedSegmentNotFoundError,
    metadata::segment_metadata::AddSymbolError,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UserSymbolOverlapError {
    sym_name: String,
    sym_vram: Vram,

    other_name: String,
    other_vram: Vram,
    other_size: Size,
}

impl UserSymbolOverlapError {
    pub(crate) fn new(
        sym_name: String,
        sym_vram: Vram,
        other_name: String,
        other_vram: Vram,
        other_size: Size,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            other_name,
            other_vram,
            other_size,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct RomOutOfRangeError {
    rom: Rom,
    segment_ranges: AddressRange<Rom>,
}

impl RomOutOfRangeError {
    pub(crate) fn new(rom: Rom, segment_ranges: AddressRange<Rom>) -> Self {
        Self {
            rom,
            segment_ranges,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum AddUserSymbolError {
    Overlap(UserSymbolOverlapError),
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    AddSymbol(AddSymbolError),
    RomOutOfRange(RomOutOfRangeError),
}

impl fmt::Display for AddUserSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddUserSymbolError::Overlap(overlap) => {
                write!(f,
                    "The symbol `{}` (vram: 0x{}) overlaps the symbol `{}` (vram: 0x{}). `{}` has a size of {} bytes",
                    overlap.sym_name,
                    overlap.sym_vram,
                    overlap.other_name,
                    overlap.other_vram,
                    overlap.other_name,
                    overlap.other_size,
                )
            }
            AddUserSymbolError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{}", owned_segment_not_found_error)
            }
            AddUserSymbolError::AddSymbol(add_symbol_error) => {
                write!(f, "{}", add_symbol_error)
            }
            AddUserSymbolError::RomOutOfRange(rom_out_of_range) => {
                write!(
                    f,
                    "Rom 0x{} is out of range {:?}",
                    rom_out_of_range.rom.inner(),
                    rom_out_of_range.segment_ranges
                )
            }
        }
    }
}
impl error::Error for AddUserSymbolError {}

impl From<AddSymbolError> for AddUserSymbolError {
    fn from(value: AddSymbolError) -> Self {
        AddUserSymbolError::AddSymbol(value)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddUserSymbolError, PyRuntimeError);

    impl std::convert::From<super::AddUserSymbolError> for PyErr {
        fn from(err: super::AddUserSymbolError) -> PyErr {
            AddUserSymbolError::new_err(err.to_string())
        }
    }
}
