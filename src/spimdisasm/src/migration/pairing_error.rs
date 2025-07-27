/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::symbols::display::SymDisplayError;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum PairingError {
    MissingTextSection(MissingTextSectionError),
    MissingRodataSection(MissingRodataSectionError),
    FunctionOutOfBounds(FunctionOutOfBoundsError),
    RodataOutOfBounds(RodataOutOfBoundsError),
    SymDisplayFail(SymDisplayError),
}
impl fmt::Display for PairingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Not able to create a display name for this Function-Rodata pairing: "
        )?;
        match self {
            PairingError::MissingTextSection(x) => write!(f, "{x}"),
            PairingError::MissingRodataSection(x) => write!(f, "{x}"),
            PairingError::FunctionOutOfBounds(x) => write!(f, "{x}"),
            PairingError::RodataOutOfBounds(x) => write!(f, "{x}"),
            PairingError::SymDisplayFail(x) => write!(f, "{x}"),
        }
    }
}
impl error::Error for PairingError {}

impl From<MissingTextSectionError> for PairingError {
    fn from(value: MissingTextSectionError) -> Self {
        PairingError::MissingTextSection(value)
    }
}
impl From<MissingRodataSectionError> for PairingError {
    fn from(value: MissingRodataSectionError) -> Self {
        PairingError::MissingRodataSection(value)
    }
}
impl From<FunctionOutOfBoundsError> for PairingError {
    fn from(value: FunctionOutOfBoundsError) -> Self {
        PairingError::FunctionOutOfBounds(value)
    }
}
impl From<RodataOutOfBoundsError> for PairingError {
    fn from(value: RodataOutOfBoundsError) -> Self {
        PairingError::RodataOutOfBounds(value)
    }
}
impl From<SymDisplayError> for PairingError {
    fn from(value: SymDisplayError) -> Self {
        PairingError::SymDisplayFail(value)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct MissingTextSectionError {}
impl MissingTextSectionError {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
impl fmt::Display for MissingTextSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Text section should be the same as the one used to generate the pairing. Somehow it is missing."
        )
    }
}
impl error::Error for MissingTextSectionError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct MissingRodataSectionError {}
impl MissingRodataSectionError {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
impl fmt::Display for MissingRodataSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rodata section should be the same as the one used to generate the pairing. Somehow it is missing."
        )
    }
}
impl error::Error for MissingRodataSectionError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct FunctionOutOfBoundsError {
    index: usize,
    len: usize,
    section_name: Arc<str>,
}
impl FunctionOutOfBoundsError {
    pub(crate) fn new(index: usize, len: usize, section_name: Arc<str>) -> Self {
        Self {
            index,
            len,
            section_name,
        }
    }
}
impl fmt::Display for FunctionOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Out of bound function access.")?;
        writeln!(
            f,
            "    Tried to access function with index '{}' but section '{}' has '{}' elements.",
            self.index, self.section_name, self.len
        )?;
        write!(f, "    This may be caused because the passed section is not the same as the one used to generate this pairing.")
    }
}
impl error::Error for FunctionOutOfBoundsError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct RodataOutOfBoundsError {
    index: usize,
    len: usize,
    section_name: Arc<str>,
}
impl RodataOutOfBoundsError {
    pub(crate) fn new(index: usize, len: usize, section_name: Arc<str>) -> Self {
        Self {
            index,
            len,
            section_name,
        }
    }
}
impl fmt::Display for RodataOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Out of bound function access.")?;
        writeln!(
            f,
            "    Tried to access function with index '{}' but section '{}' has '{}' elements.",
            self.index, self.section_name, self.len
        )?;
        write!(f, "    This may be caused because the passed section is not the same as the one used to generate this pairing.")
    }
}
impl error::Error for RodataOutOfBoundsError {}

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
                super::PairingError::MissingTextSection(..) => {
                    MissingTextSection::new_err(err.to_string())
                }
                super::PairingError::MissingRodataSection(..) => {
                    MissingRodataSection::new_err(err.to_string())
                }
                super::PairingError::FunctionOutOfBounds(..) => {
                    FunctionOutOfBounds::new_err(err.to_string())
                }
                super::PairingError::RodataOutOfBounds(..) => {
                    RodataOutOfBounds::new_err(err.to_string())
                }
                super::PairingError::SymDisplayFail(..) => SymDisplayFail::new_err(err.to_string()),
            }
        }
    }
}
