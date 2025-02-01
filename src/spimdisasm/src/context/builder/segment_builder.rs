/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, RomVramRange, Size, Vram},
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
    metadata::{
        GeneratedBy, IgnoredAddressRange, OverlayCategoryName, OwnerSegmentKind, SymbolMetadata,
        SymbolType,
    },
};

use super::{
    AddIgnoredAddressRangeError, AddUserSymbolError, GlobalSegmentHeater, OverlaySegmentHeater,
};

#[derive(Debug, Clone, PartialEq)]
struct SegmentBuilder {
    ranges: RomVramRange,
    name: Option<String>,
    prioritised_overlays: Vec<String>,
    user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
    ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
}

impl SegmentBuilder {
    fn new(ranges: RomVramRange, name: Option<String>) -> Self {
        let mut ignored_addresses = AddendedOrderedMap::new();

        // Hardcode the address 0 to always be ignored.
        ignored_addresses.find_mut_or_insert_with(Vram::new(0), FindSettings::new(false), || {
            (
                Vram::new(0),
                IgnoredAddressRange::new(Vram::new(0), Size::new(1)),
            )
        });

        Self {
            ranges,
            name,
            prioritised_overlays: Vec::new(),
            user_symbols: AddendedOrderedMap::new(),
            ignored_addresses,
        }
    }

    fn add_prioritised_overlay(&mut self, segment_name: String) {
        self.prioritised_overlays.push(segment_name);
    }

    fn add_user_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSymbolError> {
        if let Some(rom) = rom {
            if !self.ranges.in_rom_range(rom) {
                return Err(AddUserSymbolError::new_rom_out_of_range(
                    name,
                    vram,
                    self.name.clone(),
                    rom,
                    *self.ranges.rom(),
                ));
            }
        }

        if !self.ranges.in_vram_range(vram) {
            return Err(AddUserSymbolError::new_vram_out_of_range(
                name,
                vram,
                self.name.clone(),
                *self.ranges.vram(),
            ));
        }

        let check_addend = !sym_type.is_some_and(|x| x.is_label());

        let (sym, newly_created) = self.user_symbols.find_mut_or_insert_with(
            vram,
            FindSettings::new(check_addend),
            || {
                let owner_segment_kind = if let Some(name) = &self.name {
                    OwnerSegmentKind::Overlay(name.clone())
                } else {
                    OwnerSegmentKind::Global
                };
                (
                    vram,
                    SymbolMetadata::new(GeneratedBy::UserDeclared, vram, owner_segment_kind),
                )
            },
        );

        if sym.vram() != vram
            && !(sym.is_trustable_function() && sym_type.is_some_and(|x| x.is_label()))
        {
            Err(AddUserSymbolError::new_overlap(
                name,
                vram,
                self.name.clone(),
                sym.display_name().to_string(),
                sym.vram(),
                sym.size().unwrap(),
            ))
        } else if !newly_created {
            Err(AddUserSymbolError::new_duplicated(
                name,
                vram,
                self.name.clone(),
                sym.display_name().to_string(),
                sym.vram(),
            ))
        } else {
            *sym.user_declared_name_mut() = Some(name);
            *sym.rom_mut() = rom;
            if let Some(sym_type) = sym_type {
                sym.set_type_with_priorities(sym_type, GeneratedBy::UserDeclared);
            }
            Ok(sym)
        }
    }

    fn add_ignored_address_range(
        &mut self,
        vram: Vram,
        size: Size,
    ) -> Result<(), AddIgnoredAddressRangeError> {
        let (ignored_address, newly_created) =
            self.ignored_addresses
                .find_mut_or_insert_with(vram, FindSettings::new(true), || {
                    (vram, IgnoredAddressRange::new(vram, size))
                });

        if ignored_address.vram() != vram {
            return Err(AddIgnoredAddressRangeError::new_overlap(
                vram,
                size,
                ignored_address.vram(),
                ignored_address.size(),
            ));
        }

        if !newly_created {
            return Err(AddIgnoredAddressRangeError::new_duplicated(
                vram,
                size,
                ignored_address.vram(),
                ignored_address.size(),
            ));
        }

        Ok(())
    }

    fn n64_default_banned_addresses(&mut self) -> Result<(), AddIgnoredAddressRangeError> {
        const ADDRESSES: [Vram; 5] = [
            Vram::new(0x7FFFFFE0), // osInvalICache
            Vram::new(0x7FFFFFF0), // osInvalDCache, osWritebackDCache, osWritebackDCacheAll
            Vram::new(0x7FFFFFFF),
            Vram::new(0x80000010),
            Vram::new(0x80000020),
        ];

        for addr in ADDRESSES {
            self.add_ignored_address_range(addr, Size::new(1))?;
        }

        Ok(())
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
            inner: SegmentBuilder::new(ranges, None),
        }
    }

    pub fn add_prioritised_overlay(&mut self, segment_name: String) {
        self.inner.add_prioritised_overlay(segment_name);
    }

    pub fn add_user_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSymbolError> {
        self.inner.add_user_symbol(name, vram, rom, sym_type)
    }

    pub fn add_ignored_address_range(
        &mut self,
        vram: Vram,
        size: Size,
    ) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.add_ignored_address_range(vram, size)
    }

    pub fn n64_default_banned_addresses(&mut self) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.n64_default_banned_addresses()
    }

    pub fn finish_symbols(self) -> GlobalSegmentHeater {
        GlobalSegmentHeater::new(
            self.inner.ranges,
            self.inner.prioritised_overlays,
            self.inner.user_symbols,
            self.inner.ignored_addresses,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OverlaySegmentBuilder {
    inner: SegmentBuilder,
    category_name: OverlayCategoryName,
}

impl OverlaySegmentBuilder {
    pub fn new(
        ranges: RomVramRange,
        category_name: OverlayCategoryName,
        segment_name: String,
    ) -> Self {
        Self {
            inner: SegmentBuilder::new(ranges, Some(segment_name)),
            category_name,
        }
    }

    pub fn add_prioritised_overlay(&mut self, segment_name: String) {
        self.inner.add_prioritised_overlay(segment_name);
    }

    pub fn add_user_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<Rom>,
        sym_type: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSymbolError> {
        self.inner.add_user_symbol(name, vram, rom, sym_type)
    }

    pub fn add_ignored_address_range(
        &mut self,
        vram: Vram,
        size: Size,
    ) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.add_ignored_address_range(vram, size)
    }

    pub fn n64_default_banned_addresses(&mut self) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.n64_default_banned_addresses()
    }

    pub fn finish_symbols(self) -> OverlaySegmentHeater {
        OverlaySegmentHeater::new(
            self.inner.ranges,
            self.inner.name.expect(
                "Should not be None since that's the only way to create an object of this struct",
            ),
            self.inner.prioritised_overlays,
            self.inner.user_symbols,
            self.inner.ignored_addresses,
            self.category_name,
        )
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

        #[pyo3(name = "add_user_symbol", signature = (name, vram, rom, attributes))]
        pub fn py_add_symbol(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            attributes: &SymAttributes,
        ) -> Result<(), AddUserSymbolError> {
            let sym = self.inner.add_user_symbol(name, vram, rom, None)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "add_ignored_address_range")]
        pub fn py_add_ignored_address_range(
            &mut self,
            vram: Vram,
            size: Size,
        ) -> Result<(), AddIgnoredAddressRangeError> {
            self.add_ignored_address_range(vram, size)
        }

        #[pyo3(name = "n64_default_banned_addresses")]
        pub fn py_n64_default_banned_addresses(
            &mut self,
        ) -> Result<(), AddIgnoredAddressRangeError> {
            self.n64_default_banned_addresses()
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

        #[pyo3(name = "add_user_symbol", signature = (name, vram, rom, attributes))]
        pub fn py_add_symbol(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            attributes: &SymAttributes,
        ) -> Result<(), AddUserSymbolError> {
            let sym = self
                .inner
                .add_user_symbol(name, vram, rom, attributes.typ)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "add_ignored_address_range")]
        pub fn py_add_ignored_address_range(
            &mut self,
            vram: Vram,
            size: Size,
        ) -> Result<(), AddIgnoredAddressRangeError> {
            self.add_ignored_address_range(vram, size)
        }

        #[pyo3(name = "n64_default_banned_addresses")]
        pub fn py_n64_default_banned_addresses(
            &mut self,
        ) -> Result<(), AddIgnoredAddressRangeError> {
            self.n64_default_banned_addresses()
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
