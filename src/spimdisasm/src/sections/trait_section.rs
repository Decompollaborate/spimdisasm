/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::addresses::{AddressRange, Rom, RomVramRange, Size, Vram};
use crate::collections::unordered_set::UnorderedSet;
use crate::context::Context;
use crate::parent_segment_info::ParentSegmentInfo;
use crate::section_type::SectionType;
use crate::symbols::Symbol;

use super::SectionPostProcessError;

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
    fn symbols_vrams(&self) -> &UnorderedSet<Vram>;

    fn post_process(&mut self, context: &Context) -> Result<(), SectionPostProcessError>;
}

pub trait RomSection {
    #[must_use]
    fn rom_vram_range(&self) -> &RomVramRange;

    #[must_use]
    fn rom_range(&self) -> &AddressRange<Rom> {
        self.rom_vram_range().rom()
    }

    #[must_use]
    fn rom_size(&self) -> Size {
        self.rom_range().size()
    }
}
