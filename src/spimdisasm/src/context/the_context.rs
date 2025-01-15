/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use core::{error, fmt};

use ::polonius_the_crab::prelude::*;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, Vram},
    collections::unordered_map::UnorderedMap,
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata},
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    sections::{
        SectionCreationError, SectionData, SectionDataSettings, SectionExecutable,
        SectionExecutableSettings, SectionNoload, SectionNoloadSettings,
    },
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct Context {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    // unknown_segment: SegmentMetadata,

    //
    overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
    //
    // totalVramRange: SymbolsRanges

    // Maybe move to SegmentMetadata?
    // # Stuff that looks like pointers, but the disassembler shouldn't count it as a pointer
    // self.bannedSymbols: set[int] = set()
    // self.bannedRangedSymbols: list[AddressRange] = list()

    // self.globalRelocationOverrides: dict[int, RelocationInfo] = dict()
    // "key: vrom address"

    // self.gpAccesses = GpAccessContainer()
}

impl Context {
    pub(crate) fn new(
        global_config: GlobalConfig,
        global_segment: SegmentMetadata,
        overlay_segments: UnorderedMap<OverlayCategoryName, OverlayCategory>,
    ) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments,
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
    pub const fn overlay_segments(&self) -> &UnorderedMap<OverlayCategoryName, OverlayCategory> {
        &self.overlay_segments
    }
}

impl Context {
    pub fn create_section_text(
        &mut self,
        settings: &SectionExecutableSettings,
        name: String,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<SectionExecutable, SectionCreationError> {
        SectionExecutable::new(
            self,
            settings,
            name,
            raw_bytes,
            rom,
            vram,
            parent_segment_info,
        )
    }

    pub fn create_section_data(
        &mut self,
        settings: &SectionDataSettings,
        name: String,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<SectionData, SectionCreationError> {
        SectionData::new(
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

    pub fn create_section_rodata(
        &mut self,
        settings: &SectionDataSettings,
        name: String,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<SectionData, SectionCreationError> {
        SectionData::new(
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

    pub fn create_section_bss(
        &mut self,
        settings: &SectionNoloadSettings,
        name: String,
        vram_range: AddressRange<Vram>,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<SectionNoload, SectionCreationError> {
        SectionNoload::new(self, settings, name, vram_range, parent_segment_info)
    }

    pub fn create_section_gcc_except_table(
        &mut self,
        settings: &SectionDataSettings,
        name: String,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<SectionData, SectionCreationError> {
        SectionData::new(
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
    pub(crate) fn find_referenced_segment(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
    ) -> Option<&SegmentMetadata> {
        // TODO: Maybe remove Option and actually implement the unknown_segment?
        if self.global_segment.in_vram_range(vram) {
            return Some(&self.global_segment);
        }

        if let Some(overlay_category_name) = info.overlay_category_name() {
            // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.
            if let Some(segments_per_rom) = self.overlay_segments.get(overlay_category_name) {
                if let Some(segment) = segments_per_rom.segments().get(&info.segment_rom()) {
                    if segment.in_vram_range(vram) {
                        return Some(segment);
                    }
                }
            }
        }

        let overlay_category_name = info.overlay_category_name();
        // If not found, then we should check every category except the one that associated to the parent segment.
        for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
            if overlay_category_name == Some(ovl_cat) {
                continue;
            }
            let segment = &segments_per_rom.placeholder_segment();
            if segment.in_vram_range(vram) {
                return Some(segment);
            }
        }

        None
    }
}

fn find_referenced_segment_mut_impl<'ctx>(
    mut slf: &'ctx mut Context,
    vram: Vram,
    info: &ParentSegmentInfo,
) -> Option<&'ctx mut SegmentMetadata> {
    if slf.global_segment.in_vram_range(vram) {
        return Some(&mut slf.global_segment);
    }

    if let Some(overlay_category_name) = info.overlay_category_name() {
        // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.

        polonius!(|slf| -> Option<&'polonius mut SegmentMetadata> {
            if let Some(segments_per_rom) = slf.overlay_segments.get_mut(overlay_category_name) {
                if let Some(segment) = segments_per_rom.segments_mut().get_mut(&info.segment_rom())
                {
                    if segment.in_vram_range(vram) {
                        polonius_return!(Some(segment));
                    }
                }
            }
        });
    }

    let overlay_category_name = info.overlay_category_name();
    // If not found, then we should check every category except the one that associated to the parent segment.
    for (ovl_cat, segments_per_rom) in slf.overlay_segments.iter_mut() {
        if overlay_category_name == Some(ovl_cat) {
            continue;
        }
        let segment = segments_per_rom.placeholder_segment_mut();
        if segment.in_vram_range(vram) {
            return Some(segment);
        }
    }

    None
}

impl Context {
    // TODO: remove `allow`
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn find_referenced_segment_mut(
        &mut self,
        vram: Vram,
        info: &ParentSegmentInfo,
    ) -> Option<&mut SegmentMetadata> {
        // TODO: Maybe remove Option and actually implement the unknown_segment?
        find_referenced_segment_mut_impl(self, vram, info)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use std::borrow::Cow;

    use pyo3::{exceptions::PyRuntimeError, prelude::*};

    use super::*;

    #[pymethods]
    impl Context {
        #[pyo3(name = "create_section_text")]
        pub fn py_create_section_text(
            &mut self,
            settings: &SectionExecutableSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionExecutable, SectionCreationError> {
            self.create_section_text(
                settings,
                name,
                &raw_bytes,
                rom,
                Vram::new(vram),
                parent_segment_info,
            )
        }

        #[pyo3(name = "create_section_data")]
        pub fn py_create_section_data(
            &mut self,
            settings: &SectionDataSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionData, SectionCreationError> {
            self.create_section_data(
                settings,
                name,
                &raw_bytes,
                rom,
                Vram::new(vram),
                parent_segment_info,
            )
        }

        #[pyo3(name = "create_section_rodata")]
        pub fn py_create_section_rodata(
            &mut self,
            settings: &SectionDataSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionData, SectionCreationError> {
            self.create_section_rodata(
                settings,
                name,
                &raw_bytes,
                rom,
                Vram::new(vram),
                parent_segment_info,
            )
        }

        #[pyo3(name = "create_section_bss")]
        pub fn py_create_section_bss(
            &mut self,
            settings: &SectionNoloadSettings,
            name: String,
            vram_start: u32, // Vram // TODO
            vram_end: u32,   // Vram // TODO
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionNoload, SectionCreationError> {
            let vram_ranges = AddressRange::new(Vram::new(vram_start), Vram::new(vram_end));

            self.create_section_bss(settings, name, vram_ranges, parent_segment_info)
        }

        #[pyo3(name = "create_section_gcc_except_table")]
        pub fn py_create_section_gcc_except_table(
            &mut self,
            settings: &SectionDataSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: u32, // Vram, // TODO
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionData, SectionCreationError> {
            self.create_section_gcc_except_table(
                settings,
                name,
                &raw_bytes,
                rom,
                Vram::new(vram),
                parent_segment_info,
            )
        }
    }

    impl From<OwnedSegmentNotFoundError> for PyErr {
        fn from(_value: OwnedSegmentNotFoundError) -> Self {
            PyRuntimeError::new_err("OwnedSegmentNotFoundError")
        }
    }
}
