/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use address_space::Vram;

use crate::collections::addended_ordered_map::{AddendedOrderedMap, FindSettings};

use super::SymbolMetadata;

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub struct UserSegmentMetadata {
    symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
}

impl UserSegmentMetadata {
    pub(crate) fn new(symbols: AddendedOrderedMap<Vram, SymbolMetadata>) -> Self {
        Self { symbols }
    }

    #[must_use]
    pub fn find_symbol(&self, vram: Vram, settings: FindSettings) -> Option<&SymbolMetadata> {
        self.symbols.find(&vram, settings)
    }
}
