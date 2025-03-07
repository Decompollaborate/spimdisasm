/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{Rom, Vram},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    parent_segment_info::ParentSegmentInfo,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReferrerInfo {
    Function {
        sym_vram: Vram,
        parent: ParentSegmentInfo,
        specific_rom: Rom,
    },
    Data {
        sym_vram: Vram,
        parent: ParentSegmentInfo,
        specific_rom: Rom,
    },
}

impl ReferrerInfo {
    pub(crate) const fn new_function(
        sym_vram: Vram,
        parent: ParentSegmentInfo,
        specific_rom: Rom,
    ) -> Self {
        ReferrerInfo::Function {
            sym_vram,
            parent,
            specific_rom,
        }
    }
    pub(crate) const fn new_data(
        sym_vram: Vram,
        parent: ParentSegmentInfo,
        specific_rom: Rom,
    ) -> Self {
        ReferrerInfo::Data {
            sym_vram,
            parent,
            specific_rom,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Referrers {
    /// Which functions reference this.
    /// Key is the vram of the function and the segment it is contained on (since vrams can overlap
    /// on different segments).
    /// Value is the rom of the instruction that references this symbol, so we can know how many
    /// times a function references the same address.
    reference_functions: UnorderedMap<(Vram, ParentSegmentInfo), UnorderedSet<Rom>>,
    /// Which symbols reference this.
    /// Key is the vram of the non-function symbol and the segment it is contained on (since vrams
    /// can overlap on different segments).
    /// Value is the rom of the word that references this symbol, so we can know how many
    /// times a symbol references the same address.
    reference_symbols: UnorderedMap<(Vram, ParentSegmentInfo), UnorderedSet<Rom>>,
}

impl Referrers {
    pub(crate) fn new() -> Self {
        Self {
            reference_functions: UnorderedMap::new(),
            reference_symbols: UnorderedMap::new(),
        }
    }

    /// How much this address is referenced by something else.
    pub fn reference_counter(&self) -> usize {
        self.reference_functions.values().count() + self.reference_symbols.values().count()
    }

    pub(crate) fn add(&mut self, referrer: ReferrerInfo) {
        match referrer {
            ReferrerInfo::Function {
                sym_vram,
                parent,
                specific_rom,
            } => {
                self.reference_functions
                    .entry((sym_vram, parent))
                    .or_default()
                    .insert(specific_rom);
            }
            ReferrerInfo::Data {
                sym_vram,
                parent,
                specific_rom,
            } => {
                self.reference_symbols
                    .entry((sym_vram, parent))
                    .or_default()
                    .insert(specific_rom);
            }
        }
    }
}
