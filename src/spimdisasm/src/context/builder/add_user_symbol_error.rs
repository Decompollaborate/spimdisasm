/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, Size, Vram};

use super::SegmentBuilderKind;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum AddUserSymbolErrorVariant {
    Overlap {
        other_name: Arc<str>,
        other_vram: Vram,
        other_size: Size,
    },
    Duplicated {
        other_name: Arc<str>,
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
    sym_name: Arc<str>,
    sym_vram: Vram,
    segment_kind: SegmentBuilderKind,
    variant: AddUserSymbolErrorVariant,
}

impl AddUserSymbolError {
    pub(crate) fn new_overlap(
        sym_name: Arc<str>,
        sym_vram: Vram,
        segment_kind: SegmentBuilderKind,
        other_name: Arc<str>,
        other_vram: Vram,
        other_size: Size,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_kind,
            variant: AddUserSymbolErrorVariant::Overlap {
                other_name,
                other_vram,
                other_size,
            },
        }
    }

    pub(crate) fn new_duplicated(
        sym_name: Arc<str>,
        sym_vram: Vram,
        segment_kind: SegmentBuilderKind,
        other_name: Arc<str>,
        other_vram: Vram,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_kind,
            variant: AddUserSymbolErrorVariant::Duplicated {
                other_name,
                other_vram,
            },
        }
    }

    pub(crate) fn new_vram_out_of_range(
        sym_name: Arc<str>,
        sym_vram: Vram,
        segment_kind: SegmentBuilderKind,
        segment_ranges: AddressRange<Vram>,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_kind,
            variant: AddUserSymbolErrorVariant::VramOutOfRnage { segment_ranges },
        }
    }

    pub(crate) fn new_rom_out_of_range(
        sym_name: Arc<str>,
        sym_vram: Vram,
        segment_kind: SegmentBuilderKind,
        rom: Rom,
        segment_ranges: AddressRange<Rom>,
    ) -> Self {
        Self {
            sym_name,
            sym_vram,
            segment_kind,
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
            self.sym_name, self.sym_vram,
        )?;
        self.segment_kind.write_verbose(f)?;
        write!(f, ": ")?;

        match &self.variant {
            AddUserSymbolErrorVariant::Overlap {
                other_name,
                other_vram,
                other_size,
            } => {
                write!(f,
                    "This symbol overlaps the symbol `{other_name}` (vram: 0x{other_vram}). `{other_name}` has a size of {other_size} bytes",
                )
            }
            AddUserSymbolErrorVariant::Duplicated {
                other_name,
                other_vram,
            } => {
                write!(
                    f,
                    "It has the same Vram as the symbol `{other_name}` (vram: 0x{other_vram}).",
                )
            }
            AddUserSymbolErrorVariant::VramOutOfRnage { segment_ranges } => {
                write!(
                    f,
                    "Vram is outside the segment's range `{segment_ranges:?}`"
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
