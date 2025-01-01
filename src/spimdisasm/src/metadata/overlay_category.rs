/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{collections::unordered_map::UnorderedMap, rom_address::RomAddress};

use super::SegmentMetadata;

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayCategory {
    placeholder_segment: SegmentMetadata,
    segments: UnorderedMap<RomAddress, SegmentMetadata>,
}

impl OverlayCategory {
    pub(crate) fn new(
        placeholder_segment: SegmentMetadata,
        segments: UnorderedMap<RomAddress, SegmentMetadata>,
    ) -> Self {
        Self {
            placeholder_segment,
            segments,
        }
    }

    pub(crate) fn placeholder_segment(&self) -> &SegmentMetadata {
        &self.placeholder_segment
    }
    pub(crate) fn placeholder_segment_mut(&mut self) -> &mut SegmentMetadata {
        &mut self.placeholder_segment
    }

    pub(crate) fn segments(&self) -> &UnorderedMap<RomAddress, SegmentMetadata> {
        &self.segments
    }
    pub(crate) fn segments_mut(&mut self) -> &mut UnorderedMap<RomAddress, SegmentMetadata> {
        &mut self.segments
    }
}
