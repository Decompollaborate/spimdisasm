/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    collections::btree_map::{self, BTreeMap},
    string::ToString,
    sync::Arc,
    vec::Vec,
};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{GlobalOffsetTable, Rom, RomVramRange, Size, UserSize, Vram},
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
    config::GlobalConfig,
    metadata::{
        GeneratedBy, IgnoredAddressRange, LabelMetadata, LabelType, OverlayCategoryName,
        OwnerSegmentKind, SymbolMetadata, SymbolNameGenerationSettings, SymbolType,
        UserLabelMetadata, UserSymMetadata,
    },
};

use super::{
    segment_builder_error::AddPrioritisedOverlayError, AddGlobalOffsetTableError,
    AddIgnoredAddressRangeError, AddUserLabelError, AddUserSymbolError, GlobalSegmentHeater,
    OverlaySegmentHeater,
};

#[derive(Debug, Clone, PartialEq)]
struct SegmentBuilder {
    ranges: RomVramRange,
    name: Option<Arc<str>>,
    prioritised_overlays: Vec<Arc<str>>,
    user_symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
    user_labels: BTreeMap<Vram, LabelMetadata>,
    ignored_addresses: AddendedOrderedMap<Vram, IgnoredAddressRange>,
    global_offset_table: Option<GlobalOffsetTable>,
}

impl SegmentBuilder {
    fn new(ranges: RomVramRange, name: Option<Arc<str>>) -> Self {
        let mut ignored_addresses = AddendedOrderedMap::new();

        // Hardcode the address 0 to always be ignored.
        ignored_addresses.find_mut_or_insert_with(Vram::new(0), FindSettings::new(false), || {
            IgnoredAddressRange::new(Vram::new(0), Size::new(1))
        });

        Self {
            ranges,
            name,
            prioritised_overlays: Vec::new(),
            user_symbols: AddendedOrderedMap::new(),
            user_labels: BTreeMap::new(),
            ignored_addresses,
            global_offset_table: None,
        }
    }

    fn add_prioritised_overlay(
        &mut self,
        segment_name: Arc<str>,
    ) -> Result<(), AddPrioritisedOverlayError> {
        if self.name.as_ref() == Some(&segment_name) {
            Err(AddPrioritisedOverlayError::new_self_name(
                self.name.clone(),
                segment_name,
            ))
        } else if self.prioritised_overlays.contains(&segment_name) {
            Err(AddPrioritisedOverlayError::new_duplicated(
                self.name.clone(),
                segment_name,
            ))
        } else {
            self.prioritised_overlays.push(segment_name);
            Ok(())
        }
    }

    fn add_user_symbol(
        &mut self,
        name: Arc<str>,
        vram: Vram,
        rom: Option<Rom>,
        size: Option<Size>,
        sym_type: Option<SymbolType>,
    ) -> Result<UserSymMetadata<'_>, AddUserSymbolError> {
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

        // Check overlaps
        if let Some(size) = size {
            if let Some(other) = self.user_symbols.find(
                &(vram + Size::new(size.inner() - 1)),
                FindSettings::new(true),
            ) {
                return Err(AddUserSymbolError::new_overlap(
                    name,
                    vram,
                    self.name.clone(),
                    Arc::from(other.display_name().to_string()),
                    other.vram(),
                    other.size().unwrap(),
                ));
            }
        }

        let check_addend = sym_type.is_none_or(|x| x.may_have_addend());

        let (sym, newly_created) = self.user_symbols.find_mut_or_insert_with(
            vram,
            FindSettings::new(check_addend),
            || {
                let owner_segment_kind = if let Some(name) = self.name.clone() {
                    OwnerSegmentKind::Overlay(name)
                } else {
                    OwnerSegmentKind::Global
                };
                SymbolMetadata::new(
                    GeneratedBy::UserDeclared,
                    vram,
                    owner_segment_kind,
                    SymbolNameGenerationSettings::new(),
                )
            },
        );

        if sym.vram() != vram {
            Err(AddUserSymbolError::new_overlap(
                name,
                vram,
                self.name.clone(),
                Arc::from(sym.display_name().to_string()),
                sym.vram(),
                sym.size().unwrap(),
            ))
        } else if !newly_created {
            Err(AddUserSymbolError::new_duplicated(
                name,
                vram,
                self.name.clone(),
                Arc::from(sym.display_name().to_string()),
                sym.vram(),
            ))
        } else {
            sym.set_user_declared_name(name);
            *sym.rom_mut() = rom;
            *sym.user_declared_size_mut() = size;
            if let Some(sym_type) = sym_type {
                sym.set_type(sym_type, GeneratedBy::UserDeclared);
            }
            Ok(UserSymMetadata::new(sym))
        }
    }

    fn add_user_label(
        &mut self,
        name: Arc<str>,
        vram: Vram,
        rom: Option<Rom>,
        label_type: LabelType,
    ) -> Result<UserLabelMetadata<'_>, AddUserLabelError> {
        if let Some(rom) = rom {
            if !self.ranges.in_rom_range(rom) {
                return Err(AddUserLabelError::new_rom_out_of_range(
                    name,
                    vram,
                    label_type,
                    self.name.clone(),
                    rom,
                    *self.ranges.rom(),
                ));
            }
        }

        if !self.ranges.in_vram_range(vram) {
            return Err(AddUserLabelError::new_vram_out_of_range(
                name,
                vram,
                label_type,
                self.name.clone(),
                *self.ranges.vram(),
            ));
        }

        let entry = self.user_labels.entry(vram);
        match entry {
            btree_map::Entry::Occupied(occupied_entry) => {
                let label = occupied_entry.get();

                Err(AddUserLabelError::new_duplicated(
                    name,
                    vram,
                    label_type,
                    self.name.clone(),
                    Arc::from(label.display_name().to_string()),
                    label.vram(),
                    label.label_type(),
                ))
            }
            btree_map::Entry::Vacant(vacant_entry) => {
                let owner_segment_kind = if let Some(name) = self.name.clone() {
                    OwnerSegmentKind::Overlay(name)
                } else {
                    OwnerSegmentKind::Global
                };

                let label = vacant_entry.insert(LabelMetadata::new_user(
                    vram,
                    owner_segment_kind,
                    label_type,
                    name,
                    rom,
                ));
                Ok(UserLabelMetadata::new(label))
            }
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
                    IgnoredAddressRange::new(vram, size)
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

    // TODO: Document adding symbols outside of the program should be added to the UserSegment instead of here
    fn add_global_offset_table(
        &mut self,
        global_config: &GlobalConfig,
        global_offset_table: GlobalOffsetTable,
    ) -> Result<(), AddGlobalOffsetTableError> {
        if self.global_offset_table.is_some() {
            Err(AddGlobalOffsetTableError::new_already_added())
        } else if global_config.gp_config().is_none_or(|x| !x.pic()) {
            Err(AddGlobalOffsetTableError::new_not_pic())
        } else {
            self.global_offset_table = Some(global_offset_table);
            Ok(())
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
            inner: SegmentBuilder::new(ranges, None),
        }
    }

    pub fn add_prioritised_overlay<T>(
        &mut self,
        segment_name: T,
    ) -> Result<(), AddPrioritisedOverlayError>
    where
        T: Into<Arc<str>>,
    {
        self.inner.add_prioritised_overlay(segment_name.into())
    }

    pub fn add_user_symbol<T>(
        &mut self,
        name: T,
        vram: Vram,
        rom: Option<Rom>,
        size: Option<UserSize>,
        sym_type: Option<SymbolType>,
    ) -> Result<UserSymMetadata<'_>, AddUserSymbolError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .add_user_symbol(name.into(), vram, rom, size.map(|x| x.into()), sym_type)
    }

    pub fn add_user_label<T>(
        &mut self,
        name: T,
        vram: Vram,
        rom: Option<Rom>,
        label_type: LabelType,
    ) -> Result<UserLabelMetadata<'_>, AddUserLabelError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .add_user_label(name.into(), vram, rom, label_type)
    }

    pub fn add_ignored_address_range(
        &mut self,
        vram: Vram,
        size: UserSize,
    ) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.add_ignored_address_range(vram, size.into())
    }

    pub fn n64_default_banned_addresses(&mut self) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.n64_default_banned_addresses()
    }

    // TODO: Document adding symbols outside of the program should be added to the UserSegment instead of here
    pub fn add_global_offset_table(
        &mut self,
        global_config: &GlobalConfig,
        global_offset_table: GlobalOffsetTable,
    ) -> Result<(), AddGlobalOffsetTableError> {
        self.inner
            .add_global_offset_table(global_config, global_offset_table)
    }

    pub fn finish_symbols(self) -> GlobalSegmentHeater {
        GlobalSegmentHeater::new(
            self.inner.ranges,
            self.inner.prioritised_overlays.into(),
            self.inner.user_symbols,
            self.inner.user_labels,
            self.inner.ignored_addresses,
            self.inner.global_offset_table,
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
    pub fn new<T>(ranges: RomVramRange, category_name: OverlayCategoryName, segment_name: T) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self {
            inner: SegmentBuilder::new(ranges, Some(segment_name.into())),
            category_name,
        }
    }

    pub fn add_prioritised_overlay<T>(
        &mut self,
        segment_name: T,
    ) -> Result<(), AddPrioritisedOverlayError>
    where
        T: Into<Arc<str>>,
    {
        self.inner.add_prioritised_overlay(segment_name.into())
    }

    pub fn add_user_symbol<T>(
        &mut self,
        name: T,
        vram: Vram,
        rom: Option<Rom>,
        size: Option<UserSize>,
        sym_type: Option<SymbolType>,
    ) -> Result<UserSymMetadata<'_>, AddUserSymbolError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .add_user_symbol(name.into(), vram, rom, size.map(|x| x.into()), sym_type)
    }

    pub fn add_user_label<T>(
        &mut self,
        name: T,
        vram: Vram,
        rom: Option<Rom>,
        label_type: LabelType,
    ) -> Result<UserLabelMetadata<'_>, AddUserLabelError>
    where
        T: Into<Arc<str>>,
    {
        self.inner
            .add_user_label(name.into(), vram, rom, label_type)
    }

    pub fn add_ignored_address_range(
        &mut self,
        vram: Vram,
        size: UserSize,
    ) -> Result<(), AddIgnoredAddressRangeError> {
        self.inner.add_ignored_address_range(vram, size.into())
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
            self.inner.prioritised_overlays.into(),
            self.inner.user_symbols,
            self.inner.user_labels,
            self.inner.ignored_addresses,
            self.inner.global_offset_table,
            self.category_name,
        )
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use crate::metadata::RodataMigrationBehavior;

    use super::*;

    #[pymethods]
    impl GlobalSegmentBuilder {
        #[new]
        pub fn py_new(ranges: RomVramRange) -> Self {
            Self::new(ranges)
        }

        #[pyo3(name = "add_prioritised_overlay")]
        pub fn py_add_prioritised_overlay(
            &mut self,
            segment_name: String,
        ) -> Result<(), AddPrioritisedOverlayError> {
            self.add_prioritised_overlay(segment_name)
        }

        #[pyo3(name = "add_user_symbol", signature = (name, vram, rom, attributes))]
        pub fn py_add_symbol(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            attributes: &SymAttributes,
        ) -> Result<(), AddUserSymbolError> {
            let sym = self.add_user_symbol(name, vram, rom, attributes.size, attributes.typ)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "add_user_label", signature = (name, vram, rom, label_type))]
        pub fn py_add_user_label(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            label_type: LabelType,
        ) -> Result<(), AddUserLabelError> {
            self.add_user_label(name, vram, rom, label_type)?;
            Ok(())
        }

        #[pyo3(name = "add_ignored_address_range")]
        pub fn py_add_ignored_address_range(
            &mut self,
            vram: Vram,
            size: UserSize,
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
        pub fn py_add_prioritised_overlay(
            &mut self,
            segment_name: String,
        ) -> Result<(), AddPrioritisedOverlayError> {
            self.add_prioritised_overlay(segment_name)
        }

        #[pyo3(name = "add_user_symbol", signature = (name, vram, rom, attributes))]
        pub fn py_add_symbol(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            attributes: &SymAttributes,
        ) -> Result<(), AddUserSymbolError> {
            let sym = self.add_user_symbol(name, vram, rom, attributes.size, attributes.typ)?;
            attributes.apply_to_sym(sym);
            Ok(())
        }

        #[pyo3(name = "add_user_label", signature = (name, vram, rom, label_type))]
        pub fn py_add_user_label(
            &mut self,
            name: String,
            vram: Vram,
            rom: Option<Rom>,
            label_type: LabelType,
        ) -> Result<(), AddUserLabelError> {
            self.add_user_label(name, vram, rom, label_type)?;
            Ok(())
        }

        #[pyo3(name = "add_ignored_address_range")]
        pub fn py_add_ignored_address_range(
            &mut self,
            vram: Vram,
            size: UserSize,
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

    #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    #[cfg_attr(
        feature = "pyo3",
        pyclass(module = "spimdisasm", eq, name = "RodataMigrationBehavior")
    )]
    pub enum PyRodataMigrationBehavior {
        Default(),
        ForceMigrate(),
        ForceNotMigrate(),
        MigrateToSpecificFunction(String),
    }

    impl From<PyRodataMigrationBehavior> for RodataMigrationBehavior {
        fn from(value: PyRodataMigrationBehavior) -> Self {
            match value {
                PyRodataMigrationBehavior::Default() => RodataMigrationBehavior::Default,
                PyRodataMigrationBehavior::ForceMigrate() => RodataMigrationBehavior::ForceMigrate,
                PyRodataMigrationBehavior::ForceNotMigrate() => {
                    RodataMigrationBehavior::ForceNotMigrate
                }
                PyRodataMigrationBehavior::MigrateToSpecificFunction(x) => {
                    RodataMigrationBehavior::MigrateToSpecificFunction(x.into())
                }
            }
        }
    }

    #[pyclass(module = "spimdisasm")]
    pub struct SymAttributes {
        typ: Option<SymbolType>,
        defined: bool,
        size: Option<UserSize>,
        migration_behavior: PyRodataMigrationBehavior,
        allow_ref_with_addend: Option<bool>,
        can_reference: bool,
        can_be_referenced: bool,
        name_end: Option<String>,
        visibility: Option<String>,
    }

    #[pymethods]
    impl SymAttributes {
        #[new]
        pub fn py_new() -> Self {
            Self {
                typ: None,
                defined: false,
                size: None,
                migration_behavior: PyRodataMigrationBehavior::Default(),
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
        pub fn set_size(&mut self, val: &UserSize) {
            self.size = Some(*val);
        }
        pub fn set_migration_behavior(&mut self, val: &PyRodataMigrationBehavior) {
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
        pub fn apply_to_sym(&self, mut sym: UserSymMetadata) {
            //if self.defined {
            //    sym.set_defined();
            //}
            *sym.rodata_migration_behavior_mut() = self.migration_behavior.clone().into();
            if let Some(allow_ref_with_addend) = self.allow_ref_with_addend {
                sym.set_allow_ref_with_addend(allow_ref_with_addend);
            }
            /*
            sym.can_reference = self.can_reference;
            sym.can_be_referenced = self.can_be_referenced;
            */
            if let Some(name_end) = &self.name_end {
                sym.set_user_declared_name_end(Arc::from(name_end.clone()));
            }
            if let Some(visibility) = &self.visibility {
                sym.set_visibility(Arc::from(visibility.clone()));
            }
        }
    }
}
