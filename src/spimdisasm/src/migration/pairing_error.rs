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
    MissingTextSection {},
    MissingRodataSection {},
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
            PairingError::MissingTextSection {} => write!(
                f,
                "Text section should be the same as the one used to generate the pairing"
            ),
            PairingError::MissingRodataSection {} => write!(
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

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, PairingError, PyRuntimeError);

    pyo3::create_exception!(spimdisasm, MissingTextSection, PairingError);
    pyo3::create_exception!(spimdisasm, MissingRodataSection, PairingError);
    pyo3::create_exception!(spimdisasm, FunctionOutOfBounds, PairingError);
    pyo3::create_exception!(spimdisasm, RodataOutOfBounds, PairingError);
    pyo3::create_exception!(spimdisasm, SymDisplayFail, PairingError);

    impl std::convert::From<super::PairingError> for PyErr {
        fn from(err: super::PairingError) -> PyErr {
            match err {
                super::PairingError::MissingTextSection {} => {
                    MissingTextSection::new_err(err.to_string())
                }
                super::PairingError::MissingRodataSection {} => {
                    MissingRodataSection::new_err(err.to_string())
                }
                super::PairingError::FunctionOutOfBounds { .. } => {
                    FunctionOutOfBounds::new_err(err.to_string())
                }
                super::PairingError::RodataOutOfBounds { .. } => {
                    RodataOutOfBounds::new_err(err.to_string())
                }
                super::PairingError::SymDisplayFail { .. } => {
                    SymDisplayFail::new_err(err.to_string())
                }
            }
        }
    }
}
