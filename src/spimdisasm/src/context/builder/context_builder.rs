/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{config::GlobalConfig, metadata::SegmentMetadata, rom_vram_range::RomVramRange};

use super::{ContextBuilderOverlay, SegmentModifier};

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilder {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
}

impl ContextBuilder {
    #[must_use]
    pub fn new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
        let global_segment = SegmentMetadata::new(global_ranges, None);

        Self {
            global_config,
            global_segment,
        }
    }

    #[must_use]
    pub fn global_segment(&mut self) -> SegmentModifier {
        SegmentModifier::new(&mut self.global_segment)
    }

    #[must_use]
    pub fn process(self) -> ContextBuilderOverlay {
        ContextBuilderOverlay::new(self.global_config, self.global_segment)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use crate::{
        address_abstraction::Vram,
        context::builder::UserSymbolOverlapError,
        metadata::{RodataMigrationBehavior, SymbolMetadata, SymbolType},
        rom_address::RomAddress,
        size::Size,
    };

    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
            Self::new(global_config, global_ranges)
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_symbol(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_symbol(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_function(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_function(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_branch_label(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_branch_label(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_jumptable(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_jumptable(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_jumptable_label(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_jumptable_label(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_gcc_except_table(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_gcc_except_table(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(signature = (name, vram, rom, attributes))]
        pub fn add_gcc_except_table_label(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
            attributes: &SymAttributes,
        ) -> Result<(), UserSymbolOverlapError> {
            let mut segment = self.global_segment();
            let sym = segment.add_gcc_except_table_label(name, Vram::new(vram), rom)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderOverlay {
            // Silly clone because we can't move from a Python instance
            self.clone().process()
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
