/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, Vram},
    metadata::LabelType,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum AddUserLabelErrorVariant {
    Duplicated {
        other_name: Arc<str>,
        other_vram: Vram,
        other_type: LabelType,
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
pub struct AddUserLabelError {
    label_name: Arc<str>,
    label_vram: Vram,
    label_type: LabelType,
    segment_name: Option<Arc<str>>,
    variant: AddUserLabelErrorVariant,
}

impl AddUserLabelError {
    pub(crate) fn new_duplicated(
        label_name: Arc<str>,
        label_vram: Vram,
        label_type: LabelType,
        segment_name: Option<Arc<str>>,
        other_name: Arc<str>,
        other_vram: Vram,
        other_type: LabelType,
    ) -> Self {
        Self {
            label_name,
            label_vram,
            label_type,
            segment_name,
            variant: AddUserLabelErrorVariant::Duplicated {
                other_name,
                other_vram,
                other_type,
            },
        }
    }

    pub(crate) fn new_vram_out_of_range(
        label_name: Arc<str>,
        label_vram: Vram,
        label_type: LabelType,
        segment_name: Option<Arc<str>>,
        segment_ranges: AddressRange<Vram>,
    ) -> Self {
        Self {
            label_name,
            label_vram,
            label_type,
            segment_name,
            variant: AddUserLabelErrorVariant::VramOutOfRnage { segment_ranges },
        }
    }

    pub(crate) fn new_rom_out_of_range(
        label_name: Arc<str>,
        label_vram: Vram,
        label_type: LabelType,
        segment_name: Option<Arc<str>>,
        rom: Rom,
        segment_ranges: AddressRange<Rom>,
    ) -> Self {
        Self {
            label_name,
            label_vram,
            label_type,
            segment_name,
            variant: AddUserLabelErrorVariant::RomOutOfRange {
                rom,
                segment_ranges,
            },
        }
    }
}

impl fmt::Display for AddUserLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error when trying to add user label `{}` ({:?} {:?}) to ",
            self.label_name, self.label_vram, self.label_type
        )?;
        if let Some(name) = &self.segment_name {
            write!(f, "overlay segment `{}`", name)?;
        } else {
            write!(f, "the global segment")?;
        }
        write!(f, ": ")?;

        match &self.variant {
            AddUserLabelErrorVariant::Duplicated {
                other_name,
                other_vram,
                other_type,
            } => {
                write!(
                    f,
                    "It has the same Vram as the symbol `{}` (vram: 0x{}, {:?}).",
                    other_name, other_vram, other_type,
                )
            }
            AddUserLabelErrorVariant::VramOutOfRnage { segment_ranges } => {
                write!(
                    f,
                    "Vram is outside the segment's range `{:?}`",
                    segment_ranges
                )
            }
            AddUserLabelErrorVariant::RomOutOfRange {
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
impl error::Error for AddUserLabelError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddUserLabelError, PyRuntimeError);

    impl std::convert::From<super::AddUserLabelError> for PyErr {
        fn from(err: super::AddUserLabelError) -> PyErr {
            AddUserLabelError::new_err(err.to_string())
        }
    }
}
