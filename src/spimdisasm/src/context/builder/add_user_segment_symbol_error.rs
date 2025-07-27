/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

use crate::addresses::{Size, Vram};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum AddUserSegmentSymbolErrorVariant {
    Overlap,
    Duplicated,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AddUserSegmentSymbolError {
    vram: Vram,
    name: Option<Arc<str>>,
    size: Size,

    other_sym_vram: Vram,
    other_sym_name: Option<Arc<str>>,
    other_sym_size: Size,

    variant: AddUserSegmentSymbolErrorVariant,
}
impl AddUserSegmentSymbolError {
    fn new(
        vram: Vram,
        name: Option<Arc<str>>,
        size: Size,
        other_sym_vram: Vram,
        other_sym_name: Option<Arc<str>>,
        other_sym_size: Size,
        variant: AddUserSegmentSymbolErrorVariant,
    ) -> Self {
        Self {
            vram,
            name,
            size,
            other_sym_vram,
            other_sym_name,
            other_sym_size,
            variant,
        }
    }

    pub(crate) fn new_overlap(
        vram: Vram,
        name: Option<Arc<str>>,
        size: Size,
        other_sym_vram: Vram,
        other_sym_name: Option<Arc<str>>,
        other_sym_size: Size,
    ) -> Self {
        Self::new(
            vram,
            name,
            size,
            other_sym_vram,
            other_sym_name,
            other_sym_size,
            AddUserSegmentSymbolErrorVariant::Overlap,
        )
    }

    pub(crate) fn new_duplicated(
        vram: Vram,
        name: Option<Arc<str>>,
        size: Size,
        other_sym_vram: Vram,
        other_sym_name: Option<Arc<str>>,
        other_sym_size: Size,
    ) -> Self {
        Self::new(
            vram,
            name,
            size,
            other_sym_vram,
            other_sym_name,
            other_sym_size,
            AddUserSegmentSymbolErrorVariant::Duplicated,
        )
    }
}

impl fmt::Display for AddUserSegmentSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while trying to add a symbol \"")?;
        if let Some(name) = &self.name {
            write!(f, "'{name}' ")?;
        }
        write!(f, "{:?} {:?}\" to the user segment: ", self.vram, self.size)?;

        match self.variant {
            AddUserSegmentSymbolErrorVariant::Overlap => {
                write!(f, "This symbol overlaps with the previously added symbol")?
            }
            AddUserSegmentSymbolErrorVariant::Duplicated => {
                write!(f, "It has the same Vram as the previously added symbol")?
            }
        }

        write!(f, " \"")?;
        if let Some(name) = &self.other_sym_name {
            write!(f, "'{name}' ")?;
        }
        write!(f, "{:?} {:?}\"", self.other_sym_vram, self.other_sym_size)
    }
}
impl error::Error for AddUserSegmentSymbolError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddUserSegmentSymbolError, PyRuntimeError);

    impl std::convert::From<super::AddUserSegmentSymbolError> for PyErr {
        fn from(err: super::AddUserSegmentSymbolError) -> PyErr {
            AddUserSegmentSymbolError::new_err(err.to_string())
        }
    }
}
