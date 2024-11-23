/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{metadata::OverlayCategoryName, rom_address::RomAddress};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParentSegmentInfo {
    segment_rom: RomAddress,
    overlay_name: Option<OverlayCategoryName>,
}
impl ParentSegmentInfo {
    pub const fn new(segment_rom: RomAddress, overlay_name: Option<OverlayCategoryName>) -> Self {
        Self {
            segment_rom,
            overlay_name,
        }
    }

    pub const fn segment_rom(&self) -> RomAddress {
        self.segment_rom
    }
    pub const fn overlay_name(&self) -> Option<&OverlayCategoryName> {
        self.overlay_name.as_ref()
    }
}
