/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::BTreeMap;

use crate::rom_address::RomAddress;

use super::SegmentMetadata;

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

    pub(crate) fn placeholder_segment(&self) -> &SegmentMetadata {
        &self.placeholder_segment
    }
    pub(crate) fn placeholder_segment_mut(&mut self) -> &mut SegmentMetadata {
        &mut self.placeholder_segment
    }

    pub(crate) fn segments(&self) -> &BTreeMap<RomAddress, SegmentMetadata> {
        &self.segments
    }
    pub(crate) fn segments_mut(&mut self) -> &mut BTreeMap<RomAddress, SegmentMetadata> {
        &mut self.segments
    }
}
