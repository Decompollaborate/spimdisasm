/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::BTreeMap;

use rabbitizer::Vram;

use crate::{metadata::SymbolType, size::Size};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferencedAddress {
    vram: Vram,

    sym_type: BTreeMap<SymbolType, u32>,

    sizes: BTreeMap<Option<Size>, u32>,
    alignments: BTreeMap<Option<u8>, u32>,

    reference_count: usize,
}

impl ReferencedAddress {
    pub fn new(vram: Vram) -> Self {
        Self {
            vram,

            sym_type: BTreeMap::new(),

            alignments: BTreeMap::new(),
            sizes: BTreeMap::new(),

            reference_count: 0,
        }
    }

    pub const fn vram(&self) -> Vram {
        self.vram
    }

    pub fn sym_type(&self) -> Option<SymbolType> {
        if self.sym_type.len() == 1 {
            self.sym_type.iter().next().map(|(typ, _count)| *typ)
        } else {
            None
        }
    }

    pub fn size(&self) -> Option<Size> {
        if self.sizes.len() == 1 {
            self.sizes.iter().next().and_then(|(siz, _count)| *siz)
        } else {
            None
        }
    }

    pub fn reference_counter(&self) -> usize {
        self.reference_count
    }

    pub fn set_sym_type(&mut self, sym_type: SymbolType) {
        *self.sym_type.entry(sym_type).or_default() += 1;
    }

    pub fn set_size(&mut self, val: Option<u8>) {
        *self
            .sizes
            .entry(val.map(|x| Size::new(x.into())))
            .or_default() += 1;
    }
    pub fn set_alignment(&mut self, val: Option<u8>) {
        *self.alignments.entry(val).or_default() += 1;
    }

    pub fn increment_references(&mut self) {
        self.reference_count += 1;
    }
}
