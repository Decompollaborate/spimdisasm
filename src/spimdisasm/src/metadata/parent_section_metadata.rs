/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use crate::{addresses::Vram, parent_segment_info::ParentSegmentInfo};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ParentSectionMetadata {
    name: Arc<str>,
    vram: Vram,
    parent_segment_info: ParentSegmentInfo,
}

impl ParentSectionMetadata {
    pub(crate) fn new(name: Arc<str>, vram: Vram, parent_segment_info: ParentSegmentInfo) -> Self {
        Self {
            name,
            vram,
            parent_segment_info,
        }
    }

    // TODO:
    #[allow(dead_code)]
    pub fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    pub fn vram(&self) -> Vram {
        self.vram
    }
    // TODO:
    #[allow(dead_code)]
    pub fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }
}
