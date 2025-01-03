/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::string::{String, ToString};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::Vram,
    addresses::{Rom, Size},
    metadata::{segment_metadata::FindSettings, GeneratedBy, SegmentMetadata, SymbolMetadata},
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
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        let sym = self
            .segment
            .add_symbol(vram, rom, GeneratedBy::UserDeclared, None, true);
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
            Ok(sym)
        }
    }

    pub fn add_function(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        // TODO: avoid the double symbol fetching due to find_symbol and add_function

        if let Some(sym) = self
            .segment
            .find_symbol(vram, FindSettings::default().with_allow_addend(true))
        {
            if sym.vram() != vram {
                return Err(UserSymbolOverlapError {
                    sym_name: name,
                    sym_vram: vram,

                    other_name: sym.display_name().to_string(),
                    other_vram: sym.vram(),
                    other_size: sym.size().unwrap(),
                });
            }
        }

        let sym = self
            .segment
            .add_function(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        Ok(sym)
    }

    pub fn add_branch_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        let sym = self
            .segment
            .add_branch_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        Ok(sym)
    }

    pub fn add_jumptable(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        if let Some(sym) = self
            .segment
            .find_symbol(vram, FindSettings::default().with_allow_addend(true))
        {
            if sym.vram() != vram {
                return Err(UserSymbolOverlapError {
                    sym_name: name,
                    sym_vram: vram,

                    other_name: sym.display_name().to_string(),
                    other_vram: sym.vram(),
                    other_size: sym.size().unwrap(),
                });
            }
        }

        let sym = self
            .segment
            .add_jumptable(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        Ok(sym)
    }

    pub fn add_jumptable_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        let sym = self
            .segment
            .add_jumptable_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        Ok(sym)
    }

    pub fn add_gcc_except_table(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        if let Some(sym) = self
            .segment
            .find_symbol(vram, FindSettings::default().with_allow_addend(true))
        {
            if sym.vram() != vram {
                return Err(UserSymbolOverlapError {
                    sym_name: name,
                    sym_vram: vram,

                    other_name: sym.display_name().to_string(),
                    other_vram: sym.vram(),
                    other_size: sym.size().unwrap(),
                });
            }
        }

        let sym = self
            .segment
            .add_gcc_except_table(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        Ok(sym)
    }

    pub fn add_gcc_except_table_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
    ) -> Result<&mut SymbolMetadata, UserSymbolOverlapError> {
        let sym = self
            .segment
            .add_gcc_except_table_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        Ok(sym)
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
