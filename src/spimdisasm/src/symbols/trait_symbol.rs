/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::BTreeMap;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    collections::addended_ordered_map::FindSettings,
    context::Context,
    metadata::SymbolMetadata,
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
};

use super::SymbolPostProcessError;

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
                FindSettings::new(false),
            )
            .expect("Each `Symbol*` should make sure it creates its own metadata, so if this fails it means the corresponding `Symbol*` failed to do so.")
    }

    #[must_use]
    fn find_own_metadata_mut<'ctx>(&self, context: &'ctx mut Context) -> &'ctx mut SymbolMetadata {
        context
            .find_owned_segment_mut(self.parent_segment_info())
            .expect("Finding the owned segment shouldn't fail, otherwise there's a bug on the corresponding `Section*`, not here.")
            .find_symbol_mut(
                self.vram_range().start(),
                FindSettings::new(false),
            )
            .expect("Each `Symbol*` should make sure it creates its own metadata, so if this fails it means the corresponding `Symbol*` failed to do so.")
    }
}

pub trait RomSymbol: Symbol {
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

pub trait SymbolPreprocessed: Symbol {
    type Output: SymbolProcessed;

    #[doc(hidden)]
    fn post_process(
        self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<Self::Output, SymbolPostProcessError>;
}

pub trait RomSymbolPreprocessed: RomSymbol + SymbolPreprocessed {}

pub trait SymbolProcessed: Symbol {}

pub trait RomSymbolProcessed: RomSymbol + SymbolProcessed {
    #[must_use]
    fn relocs(&self) -> &[Option<RelocationInfo>];
}
