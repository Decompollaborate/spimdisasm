/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{metadata::OverlayCategoryName, rom_address::RomAddress};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParentSegmentInfo {
    segment_rom: RomAddress,
    overlay_category_name: Option<OverlayCategoryName>,
}
impl ParentSegmentInfo {
    pub const fn new(
        segment_rom: RomAddress,
        overlay_category_name: Option<OverlayCategoryName>,
    ) -> Self {
        Self {
            segment_rom,
            overlay_category_name,
        }
    }

    pub const fn segment_rom(&self) -> RomAddress {
        self.segment_rom
    }
    pub const fn overlay_category_name(&self) -> Option<&OverlayCategoryName> {
        self.overlay_category_name.as_ref()
    }
}
