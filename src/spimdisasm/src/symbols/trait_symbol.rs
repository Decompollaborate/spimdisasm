/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

use crate::{
    address_range::AddressRange,
    context::Context,
    metadata::{segment_metadata::FindSettings, SymbolMetadata},
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    rom_address::RomAddress,
    rom_vram_range::RomVramRange,
    section_type::SectionType,
    size::Size,
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

    #[must_use]
    fn section_type(&self) -> SectionType;

    #[must_use]
    fn find_own_metadata<'ctx>(&self, context: &'ctx Context) -> &'ctx SymbolMetadata {
        context
            .find_owned_segment(self.parent_segment_info())
            .expect("Finding the owned segment shouldn't fail, otherwise there's a bug on the corresponding `Section*`, not here.")
            .find_symbol(
                self.vram_range().start(),
                FindSettings::default().with_allow_addend(false),
            )
            .expect("Each `Symbol*` should make sure it creates its own metadata, so if this fails it means the corresponding `Symbol*` failed to do so.")
    }
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
