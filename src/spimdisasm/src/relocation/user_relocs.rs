/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::{btree_map, BTreeMap};
use core::{error, fmt};

use address_space::{AddressRange, Rom};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::relocation::RelocationInfo;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserRelocs {
    inner: BTreeMap<Rom, RelocationInfo>,
}

impl UserRelocs {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn add_reloc(
        &mut self,
        rom: Rom,
        reloc_info: RelocationInfo,
    ) -> Result<(), UserRelocAddError> {
        let entry = self.inner.entry(rom);
        match entry {
            btree_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(reloc_info);
                Ok(())
            }
            btree_map::Entry::Occupied(_occupied_entry) => {
                Err(UserRelocAddError { rom, reloc_info })
            }
        }
    }

    pub(crate) fn range(
        &self,
        rom_range: AddressRange<Rom>,
    ) -> btree_map::Range<'_, Rom, RelocationInfo> {
        self.inner.range(rom_range)
    }
}

impl Default for UserRelocs {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct UserRelocAddError {
    rom: Rom,
    reloc_info: RelocationInfo,
}
impl fmt::Display for UserRelocAddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Duplicated rom address {:?} while adding reloc '{:?}'",
            self.rom, self.reloc_info,
        )
    }
}
impl error::Error for UserRelocAddError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    use pyo3::exceptions::PyRuntimeError;
    pyo3::create_exception!(spimdisasm, PyUserRelocAddError, PyRuntimeError);

    impl std::convert::From<UserRelocAddError> for PyErr {
        fn from(err: UserRelocAddError) -> PyErr {
            PyUserRelocAddError::new_err(err.to_string())
        }
    }
}
