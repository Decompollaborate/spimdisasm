/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use address_space::{Rom, Size, Vram};
use pyo3::prelude::*;

use crate::metadata::{SymbolMetadata, SymbolType};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "spimdisasm", name = "SymInfo", from_py_object)
)]
pub struct PySymInfo {
    vram: Vram,
    rom: Option<Rom>,
    sym_type: Option<SymbolType>,
    size: Option<Size>,
    is_defined: bool,
    reference_counter: usize,
    overlay_category: Option<Arc<str>>,
}

impl PySymInfo {
    pub fn new(metadata: &SymbolMetadata) -> Self {
        Self {
            vram: metadata.vram(),
            rom: metadata.rom(),
            sym_type: metadata.sym_type(),
            size: metadata.size(),
            is_defined: metadata.is_defined(),
            reference_counter: metadata.reference_counter(),
            overlay_category: metadata.parent_metadata().and_then(|x| {
                x.parent_segment_info()
                    .overlay_category_name()
                    .map(|x| x.inner())
            }),
        }
    }
}

#[pymethods]
impl PySymInfo {
    #[pyo3(name = "vram")]
    pub fn py_vram(&self) -> u32 {
        self.vram.inner()
    }
    #[pyo3(name = "rom")]
    pub fn py_rom(&self) -> Option<u32> {
        self.rom.map(|x| x.inner())
    }
    #[pyo3(name = "sym_type")]
    pub fn py_sym_type(&self) -> Option<SymbolType> {
        self.sym_type
    }
    #[pyo3(name = "size")]
    pub fn py_size(&self) -> Option<u32> {
        self.size.map(|x| x.inner())
    }
    #[pyo3(name = "is_defined")]
    pub fn py_is_defined(&self) -> bool {
        self.is_defined
    }
    #[pyo3(name = "reference_counter")]
    pub fn py_reference_counter(&self) -> usize {
        self.reference_counter
    }
    #[pyo3(name = "overlay_category")]
    pub fn py_overlay_category(&self) -> Option<String> {
        self.overlay_category.as_ref().map(|x| x.to_string())
    }
}
