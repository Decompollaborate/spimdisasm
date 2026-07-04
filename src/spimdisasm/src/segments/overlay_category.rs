/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use address_space::{Rom, RomVramRange};

use super::OverlayCategoryName;
use crate::collections::unordered_map::UnorderedMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayCategory<SEG> {
    name: OverlayCategoryName,
    ranges: RomVramRange,
    segments: UnorderedMap<Rom, SEG>,
}

impl<SEG> OverlayCategory<SEG> {
    pub(crate) fn new(
        name: OverlayCategoryName,
        ranges: RomVramRange,
        segments: UnorderedMap<Rom, SEG>,
    ) -> Self {
        Self {
            name,
            ranges,
            segments,
        }
    }

    pub fn name(&self) -> &OverlayCategoryName {
        &self.name
    }
    pub fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }
    pub fn get(&self, segment_rom: Rom) -> Option<&SEG> {
        self.segments.get(&segment_rom)
    }
    pub fn get_mut(&mut self, segment_rom: Rom) -> Option<&mut SEG> {
        self.segments.get_mut(&segment_rom)
    }

    pub(crate) fn segments(&self) -> &UnorderedMap<Rom, SEG> {
        &self.segments
    }
    pub(crate) fn segments_mut(&mut self) -> &mut UnorderedMap<Rom, SEG> {
        &mut self.segments
    }
}
