/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};
use core::{error, fmt};

use ::polonius_the_crab::prelude::*;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, Vram},
    collections::{
        addended_ordered_map::FindSettings, unordered_map::UnorderedMap,
        unordered_set::UnorderedSet,
    },
    config::GlobalConfig,
    metadata::{
        LabelMetadata, OverlayCategory, OverlayCategoryName, SegmentMetadata, SymbolMetadata,
        UserSegmentMetadata,
    },
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    sections::{
        before_proc::{
            DataSection, DataSectionSettings, ExecutableSection, ExecutableSectionSettings,
            NobitsSection, NobitsSectionSettings,
        },
        SectionAlreadyCreatedError, SectionCreationError, SectionNotPreheatedError,
    },
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct Context {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    user_segment: UserSegmentMetadata,

    //
    overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,

    unknown_segment: SegmentMetadata,
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
        global_segment: SegmentMetadata,
        user_segment: UserSegmentMetadata,
        overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
        preheated_sections: UnorderedMap<Rom, bool>,
    ) -> Self {
        Self {
            global_config,
            global_segment,
            user_segment,
            overlay_segments,
            unknown_segment: SegmentMetadata::new_unknown_segment(),
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
    pub const fn global_segment(&self) -> &SegmentMetadata {
        &self.global_segment
    }
    #[must_use]
    pub const fn user_segment(&self) -> &UserSegmentMetadata {
        &self.user_segment
    }
    #[must_use]
    pub const fn overlay_segments(&self) -> &UnorderedMap<OverlayCategoryName, OverlayCategory> {
        &self.overlay_segments
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OwnedSegmentNotFoundError {
    info: ParentSegmentInfo,
}
impl fmt::Display for OwnedSegmentNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: more info
        write!(f, "Can't find owned segment for ")?;
        if let Some(overlay_name) = self.info.overlay_category_name() {
            write!(f, "overlay '{}'", overlay_name)?;
        } else {
            write!(f, "global segment")?;
        }
        write!(
            f,
            ". Addresses of the expected parent segment: Rom: 0x{:08X}, Vram: 0x{:08X}",
            self.info.segment_rom().inner(),
            self.info.segment_vram().inner()
        )?;

        Ok(())
    }
}
impl error::Error for OwnedSegmentNotFoundError {}

impl Context {
    pub(crate) fn find_owned_segment(
        &self,
        info: &ParentSegmentInfo,
    ) -> Result<&SegmentMetadata, OwnedSegmentNotFoundError> {
        if let Some(overlay_name) = info.overlay_category_name() {
            if let Some(segments_per_rom) = self.overlay_segments.get(overlay_name) {
                if let Some(segment) = segments_per_rom.segments().get(&info.segment_rom()) {
                    debug_assert!(segment.category_name() == Some(overlay_name));
                    debug_assert!(segment.rom_range().start() == info.segment_rom());
                    return Ok(segment);
                }
            }
        } else if self.global_segment.in_rom_range(info.segment_rom()) {
            // Global segment may contain more than one actual segment, so checking for ranges is okay.
            return Ok(&self.global_segment);
        } else if self.global_segment.in_vram_range(info.segment_vram()) {
            // Global segment doesn't have overlapping issues, so it should be fine to check for vram address.
            // This can be required by segments that only have bss sections.
            return Ok(&self.global_segment);
        }
        Err(OwnedSegmentNotFoundError { info: info.clone() })
    }
    pub(crate) fn find_owned_segment_mut(
        &mut self,
        info: &ParentSegmentInfo,
    ) -> Result<&mut SegmentMetadata, OwnedSegmentNotFoundError> {
        if let Some(overlay_category_name) = info.overlay_category_name() {
            if let Some(segments_per_rom) = self.overlay_segments.get_mut(overlay_category_name) {
                if let Some(segment) = segments_per_rom.segments_mut().get_mut(&info.segment_rom())
                {
                    debug_assert!(segment.category_name() == Some(overlay_category_name));
                    debug_assert!(segment.rom_range().start() == info.segment_rom());
                    return Ok(segment);
                }
            }
        } else if self.global_segment.in_rom_range(info.segment_rom()) {
            // Global segment may contain more than one actual segment, so checking for ranges is okay.
            return Ok(&mut self.global_segment);
        } else if self.global_segment.in_vram_range(info.segment_vram()) {
            // Global segment doesn't have overlapping issues, so it should be fine to check for vram address.
            // This can be required by segments that only have bss sections.
            return Ok(&mut self.global_segment);
        }
        Err(OwnedSegmentNotFoundError { info: info.clone() })
    }

    #[must_use]
    fn find_from_any_segment<'a, T: 'a, FS, FU, FV>(
        &'a self,
        vram: Vram,
        info: &ParentSegmentInfo,
        find_within_segment: FS,
        find_within_user_segment: FU,
        validate: FV,
    ) -> Option<T>
    where
        FS: Fn(&'a SegmentMetadata) -> Option<T>,
        FU: Fn(&'a UserSegmentMetadata) -> Option<T>,
        FV: Fn(&T) -> bool,
    {
        if let Some(t) = find_within_user_segment(&self.user_segment) {
            return Some(t);
        }

        if self.global_segment.in_vram_range(vram) {
            return find_within_segment(&self.global_segment).filter(|t| validate(t));
        }

        if !self.overlay_segments.is_empty() {
            if let Some(t) =
                self.find_from_overlay_segments(vram, info, &find_within_segment, &validate)
            {
                return Some(t);
            }
        }

        // If we still can't find it, fall back to the unknown segment
        find_within_segment(&self.unknown_segment).filter(|t| validate(t))
    }

    #[must_use]
    fn find_from_overlay_segments<'a, T: 'a, FS, FV>(
        &'a self,
        vram: Vram,
        info: &ParentSegmentInfo,
        find_within_segment: FS,
        validate: FV,
    ) -> Option<T>
    where
        FS: Fn(&'a SegmentMetadata) -> Option<T>,
        FV: Fn(&T) -> bool,
    {
        let overlay_category_name = info.overlay_category_name();

        if let Some(overlay_category_name) = overlay_category_name {
            // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.
            if let Some(segments_per_rom) = self.overlay_segments.get(overlay_category_name) {
                if let Some(owned_segment) = segments_per_rom.segments().get(&info.segment_rom()) {
                    if owned_segment.in_vram_range(vram) {
                        return find_within_segment(owned_segment);
                    }

                    // Check for any prioiritised overlay, if any.
                    for prioritised_overlay in owned_segment.prioritised_overlays() {
                        for (_ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
                            if !segments_per_rom.ranges().in_vram_range(vram) {
                                continue;
                            }
                            for (_segment_rom, segment) in segments_per_rom.segments() {
                                if segment.name().as_ref() == Some(prioritised_overlay)
                                    && segment.in_vram_range(vram)
                                {
                                    if let Some(t) =
                                        find_within_segment(segment).filter(|t| validate(t))
                                    {
                                        return Some(t);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // If not found, then we should check every category except the one that associated to the parent segment.

        // First we look for segments categories that only contain a single segment, since it is less likely to grab the wrong symbol.
        for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
            if overlay_category_name == Some(ovl_cat)
                || !segments_per_rom.ranges().in_vram_range(vram)
            {
                continue;
            }

            let segments = segments_per_rom.segments();
            if segments.len() == 1 {
                let (_, segment) = segments
                    .into_iter()
                    .next()
                    .expect("Should exist since we already checked the length");
                if segment.in_vram_range(vram) {
                    if let Some(t) = find_within_segment(segment).filter(|t| validate(t)) {
                        return Some(t);
                    }
                }
            }
        }

        // If we haven't found the symbol yet then we just look everywhere
        for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
            if overlay_category_name == Some(ovl_cat)
                || !segments_per_rom.ranges().in_vram_range(vram)
            {
                continue;
            }

            let segments = segments_per_rom.segments();
            if segments.len() != 1 {
                for (_, segment) in segments {
                    if segment.in_vram_range(vram) {
                        if let Some(t) = find_within_segment(segment).filter(|t| validate(t)) {
                            return Some(t);
                        }
                    }
                }
            }
        }

        None
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
        self.find_from_any_segment(
            vram,
            info,
            |segment| segment.find_symbol(vram, settings),
            |user_segment| user_segment.find_symbol(vram, settings),
            sym_validation,
        )
    }

    #[must_use]
    pub(crate) fn find_label_from_any_segment<V>(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
        label_validation: V,
    ) -> Option<&LabelMetadata>
    where
        V: Fn(&&LabelMetadata) -> bool,
    {
        self.find_from_any_segment(
            vram,
            info,
            |segment| segment.find_label(vram),
            |_| None,
            label_validation,
        )
    }
}

fn find_referenced_segment_mut_impl<'ctx>(
    mut slf: &'ctx mut Context,
    vram: Vram,
    info: &ParentSegmentInfo,
) -> &'ctx mut SegmentMetadata {
    if slf.global_segment.in_vram_range(vram) {
        return &mut slf.global_segment;
    }

    if !slf.overlay_segments.is_empty() {
        polonius!(|slf| -> &'polonius mut SegmentMetadata {
            if let Some(owned_segment) = find_referenced_overlay_segment_mut(slf, vram, info) {
                polonius_return!(owned_segment);
            }
        });
    }

    // Fallback to the unknown segment
    &mut slf.unknown_segment
}

fn find_referenced_overlay_segment_mut<'ctx>(
    mut slf: &'ctx mut Context,
    vram: Vram,
    info: &ParentSegmentInfo,
) -> Option<&'ctx mut SegmentMetadata> {
    if let Some(overlay_category_name) = info.overlay_category_name() {
        // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.

        let mut has_prioritised_overlays = false;

        polonius!(|slf| -> Option<&'polonius mut SegmentMetadata> {
            if let Some(segments_per_rom) = slf.overlay_segments.get_mut(overlay_category_name) {
                if let Some(owned_segment) =
                    segments_per_rom.segments_mut().get_mut(&info.segment_rom())
                {
                    if owned_segment.in_vram_range(vram) {
                        polonius_return!(Some(owned_segment));
                    }

                    has_prioritised_overlays = !owned_segment.prioritised_overlays().is_empty();
                }
            }
        });

        if has_prioritised_overlays {
            let mut prioritised_overlay_info = None;

            if let Some(segments_per_rom) = slf.overlay_segments.get(overlay_category_name) {
                if let Some(owned_segment) = segments_per_rom.segments().get(&info.segment_rom()) {
                    // Check for any prioiritised overlay, if any.
                    for prioritised_overlay in owned_segment.prioritised_overlays() {
                        for (ovl_cat, segments_per_rom) in slf.overlay_segments.iter() {
                            if !segments_per_rom.ranges().in_vram_range(vram) {
                                continue;
                            }
                            for (segment_rom, segment) in segments_per_rom.segments() {
                                if segment.name().as_ref() == Some(prioritised_overlay)
                                    && segment.in_vram_range(vram)
                                {
                                    // We need to clone here to avoid lifetime issues
                                    prioritised_overlay_info =
                                        Some((ovl_cat.clone(), *segment_rom));
                                    break;
                                }
                            }
                            if prioritised_overlay_info.is_some() {
                                break;
                            }
                        }
                        if prioritised_overlay_info.is_some() {
                            break;
                        }
                    }
                }
            }

            if let Some((ovl_cat, segment_rom)) = prioritised_overlay_info {
                polonius!(|slf| -> Option<&'polonius mut SegmentMetadata> {
                    if let Some(segment) = slf
                        .overlay_segments
                        .get_mut(&ovl_cat)
                        .and_then(|x| x.segments_mut().get_mut(&segment_rom))
                    {
                        polonius_return!(Some(segment));
                    }
                });
            }
        }
    }

    None
}

impl Context {
    #[must_use]
    pub(crate) fn find_referenced_segment_mut(
        &mut self,
        vram: Vram,
        info: &ParentSegmentInfo,
    ) -> &mut SegmentMetadata {
        find_referenced_segment_mut_impl(self, vram, info)
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
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyExecutableSection, SectionCreationError> {
            Ok(PyExecutableSection::new(self.create_section_text(
                settings,
                name,
                raw_bytes,
                rom,
                vram,
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_data")]
        pub fn py_create_section_data(
            &mut self,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: Vec<u8>,
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyDataSection, SectionCreationError> {
            Ok(PyDataSection::new(self.create_section_data(
                settings,
                name,
                raw_bytes,
                rom,
                vram,
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_rodata")]
        pub fn py_create_section_rodata(
            &mut self,
            settings: &DataSectionSettings,
            name: String,
            raw_bytes: Vec<u8>,
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyDataSection, SectionCreationError> {
            Ok(PyDataSection::new(self.create_section_rodata(
                settings,
                name,
                raw_bytes,
                rom,
                vram,
                parent_segment_info,
            )?))
        }

        #[pyo3(name = "create_section_bss")]
        pub fn py_create_section_bss(
            &mut self,
            settings: &NobitsSectionSettings,
            name: String,
            vram_start: Vram,
            vram_end: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyNobitsSection, SectionCreationError> {
            let vram_ranges = AddressRange::new(vram_start, vram_end);

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
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<PyDataSection, SectionCreationError> {
            Ok(PyDataSection::new(self.create_section_gcc_except_table(
                settings,
                name,
                raw_bytes,
                rom,
                vram,
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
