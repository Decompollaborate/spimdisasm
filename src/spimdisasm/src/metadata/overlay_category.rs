/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{addresses::Rom, collections::unordered_map::UnorderedMap};

use super::SegmentMetadata;

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayCategory {
    placeholder_segment: SegmentMetadata,
    segments: UnorderedMap<Rom, SegmentMetadata>,
}

impl OverlayCategory {
    pub(crate) fn new(
        placeholder_segment: SegmentMetadata,
        segments: UnorderedMap<Rom, SegmentMetadata>,
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

    pub(crate) fn segments(&self) -> &UnorderedMap<Rom, SegmentMetadata> {
        &self.segments
    }
    pub(crate) fn segments_mut(&mut self) -> &mut UnorderedMap<Rom, SegmentMetadata> {
        &mut self.segments
    }
}
