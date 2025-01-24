/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::string::String;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, Size, Vram};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum AddUserSymbolErrorVariant {
    Overlap {
        other_name: String,
        other_vram: Vram,
        other_size: Size,
    },
    Duplicated {
        other_name: String,
        other_vram: Vram,
    },
    VramOutOfRnage {
        segment_ranges: AddressRange<Vram>,
    },
    RomOutOfRange {
        rom: Rom,
        segment_ranges: AddressRange<Rom>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct AddUserSymbolError {
    sym_name: String,
    sym_vram: Vram,
    segment_name: Option<String>,
    variant: AddUserSymbolErrorVariant,
}

impl AddUserSymbolError {
    pub(crate) fn new_overlap(
        sym_name: String,
        sym_vram: Vram,
        segment_name: Option<String>,
        other_name: String,
        other_vram: Vram,
        other_size: Size,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_name,
            variant: AddUserSymbolErrorVariant::Overlap {
                other_name,
                other_vram,
                other_size,
            },
        }
    }

    pub(crate) fn new_duplicated(
        sym_name: String,
        sym_vram: Vram,
        segment_name: Option<String>,
        other_name: String,
        other_vram: Vram,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_name,
            variant: AddUserSymbolErrorVariant::Duplicated {
                other_name,
                other_vram,
            },
        }
    }

    pub(crate) fn new_vram_out_of_range(
        sym_name: String,
        sym_vram: Vram,
        segment_name: Option<String>,
        segment_ranges: AddressRange<Vram>,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_name,
            variant: AddUserSymbolErrorVariant::VramOutOfRnage { segment_ranges },
        }
    }

    pub(crate) fn new_rom_out_of_range(
        sym_name: String,
        sym_vram: Vram,
        segment_name: Option<String>,
        rom: Rom,
        segment_ranges: AddressRange<Rom>,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_name,
            variant: AddUserSymbolErrorVariant::RomOutOfRange {
                rom,
                segment_ranges,
            },
        }
    }
}

impl fmt::Display for AddUserSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error when trying to add user symbol `{}` ({:?}) to ",
            self.sym_name, self.sym_vram
        )?;
        if let Some(name) = &self.segment_name {
            write!(f, "overlay segment `{}`", name)?;
        } else {
            write!(f, "the global segment")?;
        }
        write!(f, ": ")?;

        match &self.variant {
            AddUserSymbolErrorVariant::Overlap {
                other_name,
                other_vram,
                other_size,
            } => {
                write!(f,
                    "This symbol overlaps the symbol `{}` (vram: 0x{}). `{}` has a size of {} bytes",
                    other_name,
                    other_vram,
                    other_name,
                    other_size,
                )
            }
            AddUserSymbolErrorVariant::Duplicated {
                other_name,
                other_vram,
            } => {
                write!(
                    f,
                    "It has the same Vram as the symbol `{}` (vram: 0x{}).",
                    other_name, other_vram,
                )
            }
            AddUserSymbolErrorVariant::VramOutOfRnage { segment_ranges } => {
                write!(
                    f,
                    "Vram is outside the segment's range `{:?}`",
                    segment_ranges
                )
            }
            AddUserSymbolErrorVariant::RomOutOfRange {
                rom,
                segment_ranges,
            } => {
                write!(
                    f,
                    "The rom address `0x{:08X}` of the symbol is out of the rom's range `{:?}` of the segment",
                    rom.inner(),
                    segment_ranges
                )
            }
        }
    }
}
impl error::Error for AddUserSymbolError {}

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
