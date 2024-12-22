/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::string::String;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::symbols::display::SymDisplayError;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum PairingError {
    MissingTextSection,
    MissingRodataSection,
    FunctionOutOfBounds {
        index: usize,
        len: usize,
        section_name: String,
    },
    RodataOutOfBounds {
        index: usize,
        len: usize,
        section_name: String,
    },
    SymDisplayFail {
        err: SymDisplayError,
    },
}

impl fmt::Display for PairingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Not able to create a display name for this Function-Rodata pairing: "
        )?;
        match self {
            PairingError::MissingTextSection => write!(
                f,
                "Text section should be the same as the one used to generate the pairing"
            ),
            PairingError::MissingRodataSection => write!(
                f,
                "Rodata section should be the same as the one used to generate the pairing"
            ),
            PairingError::FunctionOutOfBounds {
                index,
                len,
                section_name,
            } => {
                writeln!(f, "Out of bound function access.")?;
                writeln!(f, "    Tried to access function with index '{}' but section '{}' has '{}' elements.", index, section_name, len)?;
                write!(f, "    This may be caused because the passed section is not the same as the one used to generate this pairing.")
            }
            PairingError::RodataOutOfBounds {
                index,
                len,
                section_name,
            } => {
                writeln!(f, "Out of bound function access.")?;
                writeln!(f, "    Tried to access function with index '{}' but section '{}' has '{}' elements.", index, section_name, len)?;
                write!(f, "    This may be caused because the passed section is not the same as the one used to generate this pairing.")
            }
            PairingError::SymDisplayFail { err } => write!(f, "{}", err),
        }
    }
}

impl error::Error for PairingError {}

impl From<SymDisplayError> for PairingError {
    fn from(value: SymDisplayError) -> Self {
        PairingError::SymDisplayFail { err: value }
    }
}
