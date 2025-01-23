/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::Vram,
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
};

use super::SymbolMetadata;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub(crate) struct PlatformSegmentMetadata {
    symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
}

impl PlatformSegmentMetadata {
    pub(crate) fn new(symbols: AddendedOrderedMap<Vram, SymbolMetadata>) -> Self {
        Self { symbols }
    }

    #[must_use]
    pub(crate) fn find_symbol(
        &self,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<&SymbolMetadata> {
        self.symbols.find(&vram, settings)
    }
}
