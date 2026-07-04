/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

use address_space::{Size, Vram};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum AddAbsoluteSegmentSymbolErrorVariant {
    Overlap,
    Duplicated,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AddAbsoluteSegmentSymbolError {
    vram: Vram,
    name: Option<Arc<str>>,
    size: Size,

    other_sym_vram: Vram,
    other_sym_name: Option<Arc<str>>,
    other_sym_size: Size,

    variant: AddAbsoluteSegmentSymbolErrorVariant,
}
impl AddAbsoluteSegmentSymbolError {
    fn new(
        vram: Vram,
        name: Option<Arc<str>>,
        size: Size,
        other_sym_vram: Vram,
        other_sym_name: Option<Arc<str>>,
        other_sym_size: Size,
        variant: AddAbsoluteSegmentSymbolErrorVariant,
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
            AddAbsoluteSegmentSymbolErrorVariant::Overlap,
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
            AddAbsoluteSegmentSymbolErrorVariant::Duplicated,
        )
    }
}

impl fmt::Display for AddAbsoluteSegmentSymbolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error while trying to add a symbol \"")?;
        if let Some(name) = &self.name {
            write!(f, "'{name}' ")?;
        }
        write!(
            f,
            "{:?} {:?}\" to the absolute segment: ",
            self.vram, self.size,
        )?;

        match self.variant {
            AddAbsoluteSegmentSymbolErrorVariant::Overlap => {
                write!(f, "This symbol overlaps with the previously added symbol")?
            }
            AddAbsoluteSegmentSymbolErrorVariant::Duplicated => {
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
impl error::Error for AddAbsoluteSegmentSymbolError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddAbsoluteSegmentSymbolError, PyRuntimeError);

    impl std::convert::From<super::AddAbsoluteSegmentSymbolError> for PyErr {
        fn from(err: super::AddAbsoluteSegmentSymbolError) -> PyErr {
            AddAbsoluteSegmentSymbolError::new_err(err.to_string())
        }
    }
}
