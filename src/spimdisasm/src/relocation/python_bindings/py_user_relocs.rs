/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use address_space::Rom;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::relocation::{RelocReferencedSym, RelocationType, UserRelocAddError, UserRelocs};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "spimdisasm", name = "UserRelocs", from_py_object)
)]
pub struct PyUserRelocs {
    inner: UserRelocs,
}

impl PyUserRelocs {
    pub fn inner(&self) -> &UserRelocs {
        &self.inner
    }
}

#[pymethods]
impl PyUserRelocs {
    #[new]
    pub fn py_new() -> Self {
        Self {
            inner: UserRelocs::new(),
        }
    }

    pub fn add_reloc(
        &mut self,
        rom: u32,
        reloc_type: RelocationType,
        sym_name: String,
        addend: i64,
    ) -> Result<(), UserRelocAddError> {
        let rom = Rom::new(rom);
        let reloc_info =
            reloc_type.new_reloc_info(RelocReferencedSym::SymName(Arc::from(sym_name), addend));

        self.inner.add_reloc(rom, reloc_info)
    }
}
