/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::string::{String, ToString};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Size, Vram},
    metadata::{GeneratedBy, SegmentMetadata, SymbolMetadata, SymbolType},
};

pub struct SegmentModifier<'seg> {
    segment: &'seg mut SegmentMetadata,
}

impl<'seg> SegmentModifier<'seg> {
    pub(crate) const fn new(segment: &'seg mut SegmentMetadata) -> Self {
        Self { segment }
    }
}

impl SegmentModifier<'_> {
    pub fn add_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        let check_addend = !sym_type.is_some_and(|x| x.is_label());

        let sym = self
            .segment
            .add_symbol(vram, GeneratedBy::UserDeclared, check_addend);
        if sym.vram() != vram {
            Err(UserSymbolOverlapError {
                sym_name: name,
                sym_vram: vram,

                other_name: sym.display_name().to_string(),
                other_vram: sym.vram(),
                other_size: sym.size().unwrap(),
            })
        } else {
            *sym.user_declared_name_mut() = Some(name);
            *sym.rom_mut() = rom;
            if let Some(sym_type) = sym_type {
                sym.set_type_with_priorities(sym_type, GeneratedBy::UserDeclared);
            }
            Ok(sym)
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UserSymbolOverlapError {
    sym_name: String,
    sym_vram: Vram,

    other_name: String,
    other_vram: Vram,
    other_size: Size,
}

impl fmt::Display for UserSymbolOverlapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "The symbol `{}` (vram: 0x{}) overlaps the symbol `{}` (vram: 0x{}). `{}` has a size of {} bytes",
            self.sym_name,
            self.sym_vram,
            self.other_name,
            self.other_vram,
            self.other_name,
            self.other_size,
        )
    }
}
impl error::Error for UserSymbolOverlapError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, UserSymbolOverlapError, PyRuntimeError);

    impl std::convert::From<super::UserSymbolOverlapError> for PyErr {
        fn from(err: super::UserSymbolOverlapError) -> PyErr {
            UserSymbolOverlapError::new_err(err.to_string())
        }
    }
}
