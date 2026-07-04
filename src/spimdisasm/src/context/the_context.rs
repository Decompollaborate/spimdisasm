/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};

use addended_ordered_map::FindSettings;
use address_space::{AddressRange, Rom, Vram};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    config::GlobalConfig,
    metadata::{AbsoluteSegmentMetadata, LabelMetadata, SegmentMetadata, SymbolMetadata},
    section_type::SectionType,
    sections::{
        before_proc::{
            DataSection, DataSectionSettings, ExecutableSection, ExecutableSectionSettings,
            NobitsSection, NobitsSectionSettings,
        },
        SectionAlreadyCreatedError, SectionCreationError, SectionNotPreheatedError,
    },
    segments::{OwnedSegmentNotFoundError, ParentSegmentInfo, SegmentsCollection},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct Context {
    global_config: GlobalConfig,

    segments: SegmentsCollection<SegmentMetadata, AbsoluteSegmentMetadata>,

    //
    // totalVramRange: SymbolsRanges

    // self.gpAccesses = GpAccessContainer()

    //
    preheated_sections: UnorderedMap<Rom, bool>,
    created_noload_sections: UnorderedSet<(ParentSegmentInfo, Vram)>,
}

impl Context {
    pub(crate) fn new(
        global_config: GlobalConfig,
        segments: SegmentsCollection<SegmentMetadata, AbsoluteSegmentMetadata>,
        preheated_sections: UnorderedMap<Rom, bool>,
    ) -> Self {
        Self {
            global_config,
            segments,
            preheated_sections,
            created_noload_sections: UnorderedSet::new(),
        }
    }
}

impl Context {
    #[must_use]
    pub const fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }
    #[must_use]
    pub const fn segments(&self) -> &SegmentsCollection<SegmentMetadata, AbsoluteSegmentMetadata> {
        &self.segments
    }
}

impl Context {
    pub fn create_section_text<T>(
        &mut self,
        settings: &ExecutableSectionSettings,
        name: T,
        raw_bytes: Vec<u8>,
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<ExecutableSection, SectionCreationError>
    where
        T: Into<Arc<str>>,
    {
        let name = name.into();

        if let Some(was_created) = self.preheated_sections.get_mut(&rom) {
            if *was_created {
                return Err(SectionAlreadyCreatedError::new(name, Some(rom), vram).into());
            } else {
                *was_created = true;
            }
        } else {
            return Err(SectionNotPreheatedError::new(name, rom, vram).into());
        }

        ExecutableSection::new(
            self,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            parent_segment_info,
        )
    }

    pub fn create_section_data<T>(
        &mut self,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: Vec<u8>,
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<DataSection, SectionCreationError>
    where
        T: Into<Arc<str>>,
    {
        let name = name.into();

        if let Some(was_created) = self.preheated_sections.get_mut(&rom) {
            if *was_created {
                return Err(SectionAlreadyCreatedError::new(name, Some(rom), vram).into());
            } else {
                *was_created = true;
            }
        } else {
            return Err(SectionNotPreheatedError::new(name, rom, vram).into());
        }

        DataSection::new(
            self,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            parent_segment_info,
            SectionType::Data,
        )
    }

    pub fn create_section_rodata<T>(
        &mut self,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: Vec<u8>,
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<DataSection, SectionCreationError>
    where
        T: Into<Arc<str>>,
    {
        let name = name.into();

        if let Some(was_created) = self.preheated_sections.get_mut(&rom) {
            if *was_created {
                return Err(SectionAlreadyCreatedError::new(name, Some(rom), vram).into());
            } else {
                *was_created = true;
            }
        } else {
            return Err(SectionNotPreheatedError::new(name, rom, vram).into());
        }

        DataSection::new(
            self,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            parent_segment_info,
            SectionType::Rodata,
        )
    }

    pub fn create_section_gcc_except_table<T>(
        &mut self,
        settings: &DataSectionSettings,
        name: T,
        raw_bytes: Vec<u8>,
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<DataSection, SectionCreationError>
    where
        T: Into<Arc<str>>,
    {
        let name = name.into();

        if let Some(was_created) = self.preheated_sections.get_mut(&rom) {
            if *was_created {
                return Err(SectionAlreadyCreatedError::new(name, Some(rom), vram).into());
            } else {
                *was_created = true;
            }
        } else {
            return Err(SectionNotPreheatedError::new(name, rom, vram).into());
        }

        DataSection::new(
            self,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            parent_segment_info,
            SectionType::GccExceptTable,
        )
    }

    pub fn create_section_bss<T>(
        &mut self,
        settings: &NobitsSectionSettings,
        name: T,
        vram_range: AddressRange<Vram>,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<NobitsSection, SectionCreationError>
    where
        T: Into<Arc<str>>,
    {
        let name = name.into();

        if !self
            .created_noload_sections
            .insert((parent_segment_info.clone(), vram_range.start()))
        {
            return Err(SectionAlreadyCreatedError::new(name, None, vram_range.start()).into());
        }

        NobitsSection::new(self, settings, name, vram_range, parent_segment_info)
    }
}

impl Context {
    pub(crate) fn find_owned_segment(
        &self,
        info: &ParentSegmentInfo,
    ) -> Result<&SegmentMetadata, OwnedSegmentNotFoundError> {
        self.segments.find_owned_segment(info)
    }
    pub(crate) fn find_owned_segment_mut(
        &mut self,
        info: &ParentSegmentInfo,
    ) -> Result<&mut SegmentMetadata, OwnedSegmentNotFoundError> {
        self.segments.find_owned_segment_mut(info)
    }

    #[must_use]
    pub(crate) fn find_symbol_from_any_segment<V>(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
        settings: FindSettings,
        sym_validation: V,
    ) -> Option<&SymbolMetadata>
    where
        V: Fn(&&SymbolMetadata) -> bool,
    {
        self.segments
            .find_symbol_from_any_segment(vram, info, &settings, sym_validation)
    }

    #[must_use]
    pub(crate) fn find_label_from_any_segment<V>(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
        settings: FindSettings,
        label_validation: V,
    ) -> Option<&LabelMetadata>
    where
        V: Fn(&&LabelMetadata) -> bool,
    {
        self.segments
            .find_label_from_any_segment(vram, info, &settings, label_validation)
    }
}

impl Context {
    #[must_use]
    pub(crate) fn find_referenced_segment_mut(
        &mut self,
        vram: Vram,
        info: &ParentSegmentInfo,
    ) -> &mut SegmentMetadata {
        self.segments.find_referenced_segment_mut(vram, info)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::{exceptions::PyRuntimeError, prelude::*};

    use crate::sections::python_bindings::{
        py_data_section::PyDataSection, py_executable_section::PyExecutableSection,
        py_nobits_section::PyNobitsSection,
    };

    use super::*;

    #[pymethods]
    impl Context {
        #[pyo3(name = "create_section_text")]
        pub fn py_create_section_text(
            &mut self,
            settings: &ExecutableSectionSettings,
            name: String,
            raw_bytes: Vec<u8>,
            rom: u32,
            vram: u32,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyExecutableSection, SectionCreationError> {
            Ok(PyExecutableSection::new(self.create_section_text(
                settings,
                name,
                raw_bytes,
                Rom::new(rom),
                Vram::new(vram),
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_data")]
        pub fn py_create_section_data(
            &mut self,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: Vec<u8>,
            rom: u32,
            vram: u32,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyDataSection, SectionCreationError> {
            Ok(PyDataSection::new(self.create_section_data(
                settings,
                name,
                raw_bytes,
                Rom::new(rom),
                Vram::new(vram),
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_rodata")]
        pub fn py_create_section_rodata(
            &mut self,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: Vec<u8>,
            rom: u32,
            vram: u32,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyDataSection, SectionCreationError> {
            Ok(PyDataSection::new(self.create_section_rodata(
                settings,
                name,
                raw_bytes,
                Rom::new(rom),
                Vram::new(vram),
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_bss")]
        pub fn py_create_section_bss(
            &mut self,
            settings: &NobitsSectionSettings,
            name: String,
            vram_start: u32,
            vram_end: u32,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyNobitsSection, SectionCreationError> {
            let vram_ranges =
                AddressRange::new(Vram::new(vram_start), Vram::new(vram_end)).unwrap();

            Ok(PyNobitsSection::new(self.create_section_bss(
                settings,
                name,
                vram_ranges,
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_gcc_except_table")]
        pub fn py_create_section_gcc_except_table(
            &mut self,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: Vec<u8>,
            rom: u32,
            vram: u32,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyDataSection, SectionCreationError> {
            Ok(PyDataSection::new(self.create_section_gcc_except_table(
                settings,
                name,
                raw_bytes,
                Rom::new(rom),
                Vram::new(vram),
                parent_segment_info,
            )?))
        }
    }

    impl From<OwnedSegmentNotFoundError> for PyErr {
        fn from(_value: OwnedSegmentNotFoundError) -> Self {
            PyRuntimeError::new_err("OwnedSegmentNotFoundError")
        }
    }
}
