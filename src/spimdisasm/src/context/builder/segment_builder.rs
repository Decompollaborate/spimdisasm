/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::{String, ToString};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, RomVramRange, Vram},
    metadata::{GeneratedBy, OverlayCategoryName, SegmentMetadata, SymbolMetadata, SymbolType},
};

use super::{
    add_user_symbol_error::{RomOutOfRangeError, UserSymbolOverlapError},
    AddUserSymbolError, GlobalSegmentHeater, OverlaySegmentHeater,
};

#[derive(Debug, Clone, PartialEq)]
struct SegmentBuilder {
    segment: SegmentMetadata,
}

impl SegmentBuilder {
    fn new(segment: SegmentMetadata) -> Self {
        Self { segment }
    }

    fn add_prioritised_overlay(&mut self, segment_name: String) {
        self.segment.add_prioritised_overlay(segment_name);
    }

    fn add_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSymbolError> {
        if let Some(rom) = rom {
            if !self.segment.in_rom_range(rom) {
                return Err(AddUserSymbolError::RomOutOfRange(RomOutOfRangeError::new(
                    rom,
                    *self.segment.rom_range(),
                )));
            }
        }

        let check_addend = !sym_type.is_some_and(|x| x.is_label());

        let sym = self
            .segment
            .add_symbol(vram, GeneratedBy::UserDeclared, check_addend)?;
        if sym.vram() != vram {
            Err(AddUserSymbolError::Overlap(UserSymbolOverlapError::new(
                name,
                vram,
                sym.display_name().to_string(),
                sym.vram(),
                sym.size().unwrap(),
            )))
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GlobalSegmentBuilder {
    inner: SegmentBuilder,
}

impl GlobalSegmentBuilder {
    pub fn new(ranges: RomVramRange) -> Self {
        Self {
            inner: SegmentBuilder::new(SegmentMetadata::new_global(ranges)),
        }
    }

    pub fn add_prioritised_overlay(&mut self, segment_name: String) {
        self.inner.add_prioritised_overlay(segment_name);
    }

    pub fn add_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSymbolError> {
        self.inner.add_symbol(name, vram, rom, sym_type)
    }

    pub fn finish_symbols(self) -> GlobalSegmentHeater {
        GlobalSegmentHeater::new(self.inner.segment)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlaySegmentBuilder {
    inner: SegmentBuilder,
}

impl OverlaySegmentBuilder {
    pub fn new(
        ranges: RomVramRange,
        category_name: OverlayCategoryName,
        segment_name: String,
    ) -> Self {
        Self {
            inner: SegmentBuilder::new(SegmentMetadata::new_overlay(
                ranges,
                category_name,
                segment_name,
            )),
        }
    }

    pub fn add_prioritised_overlay(&mut self, segment_name: String) {
        self.inner.add_prioritised_overlay(segment_name);
    }

    pub fn add_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSymbolError> {
        self.inner.add_symbol(name, vram, rom, sym_type)
    }

    pub fn finish_symbols(self) -> OverlaySegmentHeater {
        OverlaySegmentHeater::new(self.inner.segment)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use crate::{addresses::Size, metadata::RodataMigrationBehavior};

    use super::*;

    #[pymethods]
    impl GlobalSegmentBuilder {
        #[new]
        pub fn py_new(ranges: RomVramRange) -> Self {
            Self::new(ranges)
        }

        #[pyo3(name = "add_prioritised_overlay")]
        pub fn py_add_prioritised_overlay(&mut self, segment_name: String) {
            self.add_prioritised_overlay(segment_name);
        }

        #[pyo3(name = "add_symbol", signature = (name, vram, rom, attributes))]
        pub fn py_add_symbol(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            attributes: &SymAttributes,
        ) -> Result<(), AddUserSymbolError> {
            let sym = self.inner.add_symbol(name, vram, rom, None)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "finish_symbols")]
        pub fn py_finish_symbols(&self) -> GlobalSegmentHeater {
            self.clone().finish_symbols()
        }
    }

    #[pymethods]
    impl OverlaySegmentBuilder {
        #[new]
        pub fn py_new(
            ranges: RomVramRange,
            category_name: OverlayCategoryName,
            segment_name: String,
        ) -> Self {
            Self::new(ranges, category_name, segment_name)
        }

        #[pyo3(name = "add_prioritised_overlay")]
        pub fn py_add_prioritised_overlay(&mut self, segment_name: String) {
            self.add_prioritised_overlay(segment_name);
        }

        #[pyo3(name = "add_symbol", signature = (name, vram, rom, attributes))]
        pub fn py_add_symbol(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            attributes: &SymAttributes,
        ) -> Result<(), AddUserSymbolError> {
            let sym = self.inner.add_symbol(name, vram, rom, None)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "finish_symbols")]
        pub fn py_finish_symbols(&self) -> OverlaySegmentHeater {
            self.clone().finish_symbols()
        }
    }

    #[pyclass(module = "spimdisasm")]
    pub struct SymAttributes {
        typ: Option<SymbolType>,
        defined: bool,
        size: Option<Size>,
        migration_behavior: RodataMigrationBehavior,
        allow_ref_with_addend: Option<bool>,
        can_reference: bool,
        can_be_referenced: bool,
        name_end: Option<String>,
        visibility: Option<String>,
    }

    #[pymethods]
    impl SymAttributes {
        #[new]
        pub fn new() -> Self {
            Self {
                typ: None,
                defined: false,
                size: None,
                migration_behavior: RodataMigrationBehavior::Default(),
                allow_ref_with_addend: None,
                can_reference: false,
                can_be_referenced: false,
                name_end: None,
                visibility: None,
            }
        }

        pub fn set_typ(&mut self, val: &SymbolType) {
            self.typ = Some(*val);
        }
        pub fn set_defined(&mut self, val: bool) {
            self.defined = val;
        }
        pub fn set_size(&mut self, val: &Size) {
            self.size = Some(*val);
        }
        pub fn set_migration_behavior(&mut self, val: &RodataMigrationBehavior) {
            self.migration_behavior = val.clone();
        }
        pub fn set_allow_ref_with_addend(&mut self, val: bool) {
            self.allow_ref_with_addend = Some(val);
        }
        pub fn set_can_reference(&mut self, val: bool) {
            self.can_reference = val;
        }
        pub fn set_can_be_referenced(&mut self, val: bool) {
            self.can_be_referenced = val;
        }
        pub fn set_name_end(&mut self, val: String) {
            self.name_end = Some(val);
        }
        pub fn set_visibility(&mut self, val: String) {
            self.visibility = Some(val);
        }
    }

    impl SymAttributes {
        pub fn apply_to_sym(&self, sym: &mut SymbolMetadata) {
            if let Some(typ) = self.typ {
                *sym.user_declared_type_mut() = Some(typ);
            }
            //if self.defined {
            //    sym.set_defined();
            //}
            if let Some(size) = self.size {
                *sym.user_declared_size_mut() = Some(size);
            }
            *sym.rodata_migration_behavior_mut() = self.migration_behavior.clone();
            if let Some(allow_ref_with_addend) = self.allow_ref_with_addend {
                sym.set_allow_ref_with_addend(allow_ref_with_addend);
            }
            /*
            sym.can_reference = self.can_reference;
            sym.can_be_referenced = self.can_be_referenced;
            */
            *sym.user_declared_name_end_mut() = self.name_end.clone();
            *sym.visibility_mut() = self.visibility.clone();
        }
    }
}
