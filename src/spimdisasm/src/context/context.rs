/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::collections::btree_map::BTreeMap;

use ::polonius_the_crab::prelude::*;

use rabbitizer::Vram;

use crate::{
    config::GlobalConfig,
    metadata::{OverlayCategoryName, SegmentMetadata},
    parent_segment_info::ParentSegmentInfo,
    rom_address::RomAddress,
};

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct OverlayCategory {
    placeholder_segment: SegmentMetadata,
    segments: BTreeMap<RomAddress, SegmentMetadata>,
}

impl OverlayCategory {
    pub(crate) fn new(
        placeholder_segment: SegmentMetadata,
        segments: BTreeMap<RomAddress, SegmentMetadata>,
    ) -> Self {
        Self {
            placeholder_segment,
            segments,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Context {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    // unknown_segment: SegmentMetadata,

    //
    overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
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
        overlay_segments: BTreeMap<OverlayCategoryName, OverlayCategory>,
    ) -> Self {
        Self {
            global_config,
            global_segment,
            overlay_segments,
        }
    }
}

impl Context {
    pub const fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }
    pub const fn global_segment(&self) -> &SegmentMetadata {
        &self.global_segment
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct OwnedSegmentNotFoundError {}
impl fmt::Display for OwnedSegmentNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: more info
        write!(f, "Can't find segment")
    }
}
impl error::Error for OwnedSegmentNotFoundError {}
impl From<OwnedSegmentNotFoundError> for fmt::Error {
    fn from(_value: OwnedSegmentNotFoundError) -> Self {
        Self {}
    }
}

impl Context {
    pub(crate) fn find_owned_segment(
        &self,
        info: &ParentSegmentInfo,
    ) -> Result<&SegmentMetadata, OwnedSegmentNotFoundError> {
        if let Some(overlay_name) = info.overlay_category_name() {
            if let Some(segments_per_rom) = self.overlay_segments.get(overlay_name) {
                if let Some(segment) = segments_per_rom.segments.get(&info.segment_rom()) {
                    debug_assert!(segment.category_name() == Some(overlay_name));
                    debug_assert!(segment.rom_range().start() == info.segment_rom());
                    return Ok(segment);
                }
            }
        } else if self.global_segment.in_rom_range(info.segment_rom()) {
            // Global segment may contain more than one actual segment, so checking for ranges is okay.
            return Ok(&self.global_segment);
        }
        Err(OwnedSegmentNotFoundError {})
    }
    #[must_use]
    pub(crate) fn find_owned_segment_mut(
        &mut self,
        info: &ParentSegmentInfo,
    ) -> Result<&mut SegmentMetadata, OwnedSegmentNotFoundError> {
        if let Some(overlay_category_name) = info.overlay_category_name() {
            if let Some(segments_per_rom) = self.overlay_segments.get_mut(overlay_category_name) {
                if let Some(segment) = segments_per_rom.segments.get_mut(&info.segment_rom()) {
                    debug_assert!(segment.category_name() == Some(overlay_category_name));
                    debug_assert!(segment.rom_range().start() == info.segment_rom());
                    return Ok(segment);
                }
            }
        } else if self.global_segment.in_rom_range(info.segment_rom()) {
            // Global segment may contain more than one actual segment, so checking for ranges is okay.
            return Ok(&mut self.global_segment);
        }
        Err(OwnedSegmentNotFoundError {})
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
                if let Some(segment) = segments_per_rom.segments.get(&info.segment_rom()) {
                    if segment.in_vram_range(vram) {
                        return Some(segment);
                    }
                }
            }

            // If not found, then we should check every category except the one that associated to the parent segment.
            for (ovl_cat, segments_per_rom) in self.overlay_segments.iter() {
                if overlay_category_name == ovl_cat {
                    continue;
                }
                let segment = &segments_per_rom.placeholder_segment;
                if segment.in_vram_range(vram) {
                    return Some(segment);
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

        polonius!(|slf| -> Option<&'polonius mut SegmentMetadata> {
            if let Some(segments_per_rom) = slf.overlay_segments.get_mut(overlay_category_name) {
                if let Some(segment) = segments_per_rom.segments.get_mut(&info.segment_rom()) {
                    if segment.in_vram_range(vram) {
                        polonius_return!(Some(segment));
                    }
                }
            }
        });

        // If not found, then we should check every category except the one that associated to the parent segment.
        for (ovl_cat, segments_per_rom) in slf.overlay_segments.iter_mut() {
            if overlay_category_name == ovl_cat {
                continue;
            }
            let segment = &mut segments_per_rom.placeholder_segment;
            if segment.in_vram_range(vram) {
                return Some(segment);
            }
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
