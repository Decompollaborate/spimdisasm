/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::collections::btree_map::BTreeMap;
use rabbitizer::Vram;

use crate::{address_range::AddressRange, metadata::{OverlayCategoryName, SegmentMetadata, SymbolMetadata}, parent_segment_info::ParentSegmentInfo, rom_address::RomAddress};

use super::GlobalConfig;

pub struct Context {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
    // unknown_segment: SegmentMetadata,

    overlay_segments: BTreeMap<OverlayCategoryName, BTreeMap<RomAddress, SegmentMetadata>>,

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
    pub fn new(global_config: GlobalConfig, global_rom_range: AddressRange<RomAddress>, global_vram_range: AddressRange<Vram>) -> Self {
        let global_segment = SegmentMetadata::new(global_rom_range, global_vram_range, None);

        Self {
            global_config,
            global_segment,
            overlay_segments: BTreeMap::new(),
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

impl Context {
    pub/*(crate)*/ fn find_owned_segment(&self, info: &ParentSegmentInfo) -> Result<&SegmentMetadata, OwnedSegmentNotFoundError> {
        if let Some(overlay_name) = info.overlay_name() {
            if let Some(segments_per_rom) = self.overlay_segments.get(overlay_name) {
                if let Some(segment) = segments_per_rom.get(&info.segment_rom()) {
                    debug_assert!(segment.category_name() == Some(overlay_name));
                    debug_assert!(segment.rom_range().start() == info.segment_rom());
                    return Ok(segment)
                }
            }
        } else if self.global_segment.in_rom_range(info.segment_rom()) {
            // Global segment may contain more than one actual segment, so checking for ranges is okay.
            return Ok(&self.global_segment);
        }
        Err(OwnedSegmentNotFoundError {})
    }
    pub(crate) fn find_owned_segment_mut(&mut self, info: &ParentSegmentInfo) -> Result<&mut SegmentMetadata, OwnedSegmentNotFoundError> {
        if let Some(overlay_name) = info.overlay_name() {
            if let Some(segments_per_rom) = self.overlay_segments.get_mut(overlay_name) {
                if let Some(segment) = segments_per_rom.get_mut(&info.segment_rom()) {
                    debug_assert!(segment.category_name() == Some(overlay_name));
                    debug_assert!(segment.rom_range().start() == info.segment_rom());
                    return Ok(segment)
                }
            }
        } else if self.global_segment.in_rom_range(info.segment_rom()) {
            // Global segment may contain more than one actual segment, so checking for ranges is okay.
            return Ok(&mut self.global_segment);
        }
        Err(OwnedSegmentNotFoundError {})
    }
}

impl Context {
    pub fn find_symbol(&self) -> Option<&SymbolMetadata> {
        // TODO
        None
    }

    //pub fn find_symbol_mut(&self) -> Option<&mut SymbolMetadata> {
    //    // TODO
    //    None
    //}
}
