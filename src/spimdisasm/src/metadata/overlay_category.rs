/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{Rom, RomVramRange},
    collections::unordered_map::UnorderedMap,
};

use super::{OverlayCategoryName, SegmentMetadata};

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayCategory {
    name: OverlayCategoryName,
    ranges: RomVramRange,
    segments: UnorderedMap<Rom, SegmentMetadata>,
}

impl OverlayCategory {
    pub(crate) fn new(
        name: OverlayCategoryName,
        ranges: RomVramRange,
        segments: UnorderedMap<Rom, SegmentMetadata>,
    ) -> Self {
        Self {
            name,
            ranges,
            segments,
        }
    }

    // pub(crate) fn name(&self) -> &OverlayCategoryName {
    //     &self.name
    // }
    pub(crate) fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }

    pub(crate) fn segments(&self) -> &UnorderedMap<Rom, SegmentMetadata> {
        &self.segments
    }
    pub(crate) fn segments_mut(&mut self) -> &mut UnorderedMap<Rom, SegmentMetadata> {
        &mut self.segments
    }
}
