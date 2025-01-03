/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::hash::Hash;

use alloc::vec::Vec;

use crate::{
    addresses::{Size, SizedAddress, Vram},
    collections::unordered_map::UnorderedMap,
    metadata::SymbolType,
};

#[derive(Debug, Clone, Eq)]
pub struct ReferencedAddress {
    vram: Vram,

    referenced_by: Vec<Vram>,

    user_declared_type: Option<SymbolType>,
    sym_type: UnorderedMap<SymbolType, u32>,

    user_declared_size: Option<Size>,
    sizes: UnorderedMap<Option<Size>, u32>,
    alignments: UnorderedMap<Option<u8>, u32>,
}

impl ReferencedAddress {
    pub(crate) fn new(vram: Vram) -> Self {
        Self {
            vram,

            referenced_by: Vec::new(),

            user_declared_type: None,
            sym_type: UnorderedMap::new(),

            user_declared_size: None,
            alignments: UnorderedMap::new(),
            sizes: UnorderedMap::new(),
        }
    }

    pub const fn vram(&self) -> Vram {
        self.vram
    }
    pub fn referenced_by(&self) -> &[Vram] {
        &self.referenced_by
    }

    pub fn sym_type(&self) -> Option<SymbolType> {
        if let Some(typ) = self.user_declared_type {
            Some(typ)
        } else if self.sym_type.len() == 1 {
            self.sym_type.iter().next().map(|(typ, _count)| *typ)
        } else {
            None
        }
    }

    pub fn size(&self) -> Option<Size> {
        if let Some(size) = self.user_declared_size {
            Some(size)
        } else if self.sizes.len() == 1 {
            self.sizes.iter().next().and_then(|(siz, _count)| *siz)
        } else {
            None
        }
    }

    pub fn alignment(&self) -> Option<u8> {
        if self.alignments.len() == 1 {
            self.alignments.iter().next().and_then(|(x, _count)| *x)
        } else {
            None
        }
    }

    pub fn reference_counter(&self) -> usize {
        self.referenced_by.len()
    }

    pub fn add_referenced_by(&mut self, specific_address: Vram) {
        self.referenced_by.push(specific_address);
    }

    pub fn set_user_declared_type(&mut self, typ: SymbolType) {
        self.user_declared_type = Some(typ);
    }
    pub fn set_sym_type(&mut self, sym_type: SymbolType) {
        *self.sym_type.entry(sym_type).or_default() += 1;
    }

    pub fn set_user_declared_size(&mut self, size: Size) {
        self.user_declared_size = Some(size);
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
}

impl PartialEq for ReferencedAddress {
    fn eq(&self, other: &Self) -> bool {
        self.vram == other.vram
    }
}
impl PartialOrd for ReferencedAddress {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.vram.partial_cmp(&other.vram)
    }
}
impl Hash for ReferencedAddress {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.vram.hash(state);
    }
}

impl SizedAddress for ReferencedAddress {
    fn size(&self) -> Option<Size> {
        self.size()
    }
}
