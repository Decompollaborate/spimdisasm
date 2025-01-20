/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use core::{error, fmt};

use ::polonius_the_crab::prelude::*;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, Vram},
    collections::{addended_ordered_map::FindSettings, unordered_map::UnorderedMap},
    config::GlobalConfig,
    metadata::{OverlayCategory, OverlayCategoryName, SegmentMetadata, SymbolMetadata},
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
    pub(crate) fn find_symbol_from_any_segment<F>(
        &self,
        vram: Vram,
        info: &ParentSegmentInfo,
        settings: FindSettings,
        ovl_sym_validation: F,
    ) -> Option<&SymbolMetadata>
    where
        F: Fn(&SymbolMetadata) -> bool,
    {
        if self.global_segment.in_vram_range(vram) {
            return self.global_segment.find_symbol(vram, settings);
        }

        if let Some(overlay_category_name) = info.overlay_category_name() {
            // First check the segment associated to this category that matches the rom address of the parent segment to prioritize it.
            if let Some(segments_per_rom) = self.overlay_segments.get(overlay_category_name) {
                if let Some(owned_segment) = segments_per_rom.segments().get(&info.segment_rom()) {
                    if owned_segment.in_vram_range(vram) {
                        return owned_segment.find_symbol(vram, settings);
                    }

                    // Check for any prioiritised overlay, if any.
                    for prioritised_overlay in owned_segment.prioritised_overlays() {
                        for (_ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
                            for (_segment_rom, segment) in segments_per_rom.segments() {
                                if segment.name() == Some(prioritised_overlay)
                                    && segment.in_vram_range(vram)
                                {
                                    return segment.find_symbol(vram, settings);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If not found, then we should check every category except the one that associated to the parent segment.

        // First we look for segments categories that only contain a single segment, since it is less likely to grab the wrong symbol.
        let overlay_category_name = info.overlay_category_name();
        for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
            if overlay_category_name == Some(ovl_cat) {
                continue;
            }

            let segments = segments_per_rom.segments();
            if segments.len() == 1 {
                let (_, segment) = segments
                    .into_iter()
                    .next()
                    .expect("Should exist since we already checked the length");
                if segment.in_vram_range(vram) {
                    if let Some(sym) = segment.find_symbol(vram, settings) {
                        if ovl_sym_validation(sym) {
                            return Some(sym);
                        }
                    }
                }
            }
        }

        // If we haven't found the symbol yet then we just look everywhere
        for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
            if overlay_category_name == Some(ovl_cat) {
                continue;
            }

            let segments = segments_per_rom.segments();
            if segments.len() != 1 {
                for (_, segment) in segments {
                    if segment.in_vram_range(vram) {
                        if let Some(sym) = segment.find_symbol(vram, settings) {
                            if ovl_sym_validation(sym) {
                                return Some(sym);
                            }
                        }
                    }
                }
            }
        }

        // If we still can't find it, fall back to the placeholders
        // TODO: is this actually fine? or should we do something else?
        for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
            if overlay_category_name == Some(ovl_cat) {
                continue;
            }

            let segment = segments_per_rom.placeholder_segment();
            if segment.in_vram_range(vram) {
                if let Some(sym) = segment.find_symbol(vram, settings) {
                    if ovl_sym_validation(sym) {
                        return Some(sym);
                    }
                }
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
                            for (segment_rom, segment) in segments_per_rom.segments() {
                                if segment.name() == Some(prioritised_overlay)
                                    && segment.in_vram_range(vram)
                                {
                                    // We need to clone here to avoid lifetime issues
                                    prioritised_overlay_info =
                                        Some((ovl_cat.clone(), *segment_rom));
                                }
                            }
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
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionExecutable, SectionCreationError> {
            self.create_section_text(settings, name, &raw_bytes, rom, vram, parent_segment_info)
        }

        #[pyo3(name = "create_section_data")]
        pub fn py_create_section_data(
            &mut self,
            settings: &SectionDataSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionData, SectionCreationError> {
            self.create_section_data(settings, name, &raw_bytes, rom, vram, parent_segment_info)
        }

        #[pyo3(name = "create_section_rodata")]
        pub fn py_create_section_rodata(
            &mut self,
            settings: &SectionDataSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionData, SectionCreationError> {
            self.create_section_rodata(settings, name, &raw_bytes, rom, vram, parent_segment_info)
        }

        #[pyo3(name = "create_section_bss")]
        pub fn py_create_section_bss(
            &mut self,
            settings: &SectionNoloadSettings,
            name: String,
            vram_start: Vram,
            vram_end: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionNoload, SectionCreationError> {
            let vram_ranges = AddressRange::new(vram_start, vram_end);

            self.create_section_bss(settings, name, vram_ranges, parent_segment_info)
        }

        #[pyo3(name = "create_section_gcc_except_table")]
        pub fn py_create_section_gcc_except_table(
            &mut self,
            settings: &SectionDataSettings,
            name: String,
            raw_bytes: Cow<[u8]>,
            rom: Rom,
            vram: Vram,
            parent_segment_info: ParentSegmentInfo,
        ) -> Result<SectionData, SectionCreationError> {
            self.create_section_gcc_except_table(
                settings,
                name,
                &raw_bytes,
                rom,
                vram,
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
