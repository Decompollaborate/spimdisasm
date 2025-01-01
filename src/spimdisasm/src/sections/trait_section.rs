/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::BTreeSet;

use rabbitizer::Vram;

use crate::address_range::AddressRange;
use crate::parent_segment_info::ParentSegmentInfo;
use crate::rom_address::RomAddress;
use crate::rom_vram_range::RomVramRange;
use crate::section_type::SectionType;
use crate::size::Size;
use crate::symbols::Symbol;

pub trait Section {
    #[must_use]
    fn name(&self) -> &str;
    #[must_use]
    fn vram_range(&self) -> &AddressRange<Vram>;

    #[must_use]
    fn size(&self) -> Size {
        self.vram_range().size()
    }

    #[must_use]
    fn parent_segment_info(&self) -> &ParentSegmentInfo;

    #[must_use]
    fn section_type(&self) -> SectionType;

    #[must_use]
    fn symbol_list(&self) -> &[impl Symbol];
    #[must_use]
    fn symbols_vrams(&self) -> &BTreeSet<Vram>;
}

pub trait RomSection {
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
}
