/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};
use std::collections::BTreeMap;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::Rom,
    relocation::{RelocReferencedSym, RelocationInfo, RelocationType},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", name = "UserRelocs"))]
pub struct PyUserRelocs {
    inner: BTreeMap<Rom, RelocationInfo>,
}

impl PyUserRelocs {
    pub fn inner(&self) -> &BTreeMap<Rom, RelocationInfo> {
        &self.inner
    }
}

#[pymethods]
impl PyUserRelocs {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn add_reloc(
        &mut self,
        rom: Rom,
        reloc_type: RelocationType,
        sym_name: String,
        addend: i32,
    ) -> Result<(), UserRelocAddError> {
        let reloc = reloc_type.new_reloc_info(RelocReferencedSym::SymName(sym_name, addend));

        if self.inner.insert(rom, reloc).is_some() {
            // err
            Err(UserRelocAddError { rom })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UserRelocAddError {
    rom: Rom,
}
impl fmt::Display for UserRelocAddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Duplicated rom address {:?} while adding relocs",
            self.rom
        )
    }
}
impl error::Error for UserRelocAddError {}

use pyo3::exceptions::PyRuntimeError;
pyo3::create_exception!(spimdisasm, PyUserRelocAddError, PyRuntimeError);

impl std::convert::From<UserRelocAddError> for PyErr {
    fn from(err: UserRelocAddError) -> PyErr {
        PyUserRelocAddError::new_err(err.to_string())
    }
}
