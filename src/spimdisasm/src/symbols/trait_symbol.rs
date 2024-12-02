/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

use crate::{
    address_range::AddressRange, parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo, rom_address::RomAddress, rom_vram_range::RomVramRange, size::Size,
};

pub trait Symbol {
    #[must_use]
    fn vram_range(&self) -> &AddressRange<Vram>;

    #[must_use]
    fn size(&self) -> Size {
        self.vram_range().size()
    }

    #[must_use]
    fn parent_segment_info(&self) -> &ParentSegmentInfo;
}

pub trait RomSymbol: Symbol {
    #[must_use]
    fn rom_vram_range(&self) -> &RomVramRange;

    #[must_use]
    fn rom_range(&self) -> &AddressRange<RomAddress> {
        self.rom_vram_range().rom()
    }

    #[must_use]
    fn rom_size(&self) -> Size {
        self.rom_range().size()
    }

    #[must_use]
    fn relocs(&self) -> &[Option<RelocationInfo>];
}
