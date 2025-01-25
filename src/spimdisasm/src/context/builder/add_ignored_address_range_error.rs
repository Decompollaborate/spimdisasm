/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use crate::addresses::{Size, Vram};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum AddIgnoredAddressRangeErrorVariant {
    Overlap,
    Duplicated,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AddIgnoredAddressRangeError {
    vram: Vram,
    size: Size,

    other_vram: Vram,
    other_size: Size,

    variant: AddIgnoredAddressRangeErrorVariant,
}
impl AddIgnoredAddressRangeError {
    fn new(
        vram: Vram,
        size: Size,
        other_vram: Vram,
        other_size: Size,
        variant: AddIgnoredAddressRangeErrorVariant,
    ) -> Self {
        Self {
            vram,
            size,
            other_vram,
            other_size,
            variant,
        }
    }

    pub(crate) fn new_overlap(vram: Vram, size: Size, other_vram: Vram, other_size: Size) -> Self {
        Self::new(
            vram,
            size,
            other_vram,
            other_size,
            AddIgnoredAddressRangeErrorVariant::Overlap,
        )
    }

    pub(crate) fn new_duplicated(
        vram: Vram,
        size: Size,
        other_vram: Vram,
        other_size: Size,
    ) -> Self {
        Self::new(
            vram,
            size,
            other_vram,
            other_size,
            AddIgnoredAddressRangeErrorVariant::Duplicated,
        )
    }
}

impl fmt::Display for AddIgnoredAddressRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while trying to add an ignored address \"{:?} {:?}\" to the user segment: ",
            self.vram, self.size
        )?;

        match self.variant {
            AddIgnoredAddressRangeErrorVariant::Overlap => {
                write!(f, "This address overlaps with the previously added address")?
            }
            AddIgnoredAddressRangeErrorVariant::Duplicated => {
                write!(f, "It has the same Vram as the previously added address")?
            }
        }

        write!(f, " \"{:?} {:?}\"", self.other_vram, self.other_size)
    }
}
impl error::Error for AddIgnoredAddressRangeError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddIgnoredAddressRangeError, PyRuntimeError);

    impl std::convert::From<super::AddIgnoredAddressRangeError> for PyErr {
        fn from(err: super::AddIgnoredAddressRangeError) -> PyErr {
            AddIgnoredAddressRangeError::new_err(err.to_string())
        }
    }
}
