/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use addended_ordered_map::{AddendedOrderedMap, FindSettings};
use address_space::{Size, Vram};

use super::SymbolMetadata;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct AbsoluteSegmentMetadata {
    symbols: AddendedOrderedMap<Vram, SymbolMetadata, Size>,
}

impl AbsoluteSegmentMetadata {
    pub(crate) fn new(symbols: AddendedOrderedMap<Vram, SymbolMetadata, Size>) -> Self {
        Self { symbols }
    }

    #[must_use]
    pub fn find_symbol(&self, vram: Vram, settings: FindSettings) -> Option<&SymbolMetadata> {
        self.symbols.find_value(&vram, settings)
    }
}
