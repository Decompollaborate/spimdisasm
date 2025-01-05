/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::hash::Hash;
use alloc::vec::Vec;
use rabbitizer::access_type::AccessType;

use crate::{
    addresses::{Size, SizedAddress, Vram},
    collections::unordered_map::UnorderedMap,
    metadata::SymbolType,
};

#[derive(Debug, Clone, Eq)]
pub struct ReferencedAddress {
    vram: Vram,

    referenced_by: Vec<Vram>,

    access_types: UnorderedMap<AccessType, u32>,

    user_declared_type: Option<SymbolType>,
    autodetected_types: UnorderedMap<SymbolType, u32>,

    user_declared_size: Option<Size>,
}

impl ReferencedAddress {
    pub(crate) fn new(vram: Vram) -> Self {
        Self {
            vram,

            referenced_by: Vec::new(),

            access_types: UnorderedMap::new(),

            user_declared_type: None,
            autodetected_types: UnorderedMap::new(),

            user_declared_size: None,
        }
    }

    pub const fn vram(&self) -> Vram {
        self.vram
    }
    pub fn referenced_by(&self) -> &[Vram] {
        &self.referenced_by
    }

    pub fn access_type(&self) -> Option<AccessType> {
        if self.access_types.len() == 1 {
            self.access_types.iter().next().map(|(access, _count)| *access)
        } else {
            None
        }
    }

    pub fn sym_type(&self) -> Option<SymbolType> {
        if let Some(typ) = self.user_declared_type {
            Some(typ)
        } else if self.autodetected_types.len() == 1 {
            self.autodetected_types.iter().next().map(|(typ, _count)| *typ)
        } else {
            self.access_type().and_then(|x| SymbolType::from_access_type(x))
        }
    }

    pub fn size(&self) -> Option<Size> {
        if let Some(size) = self.user_declared_size {
            Some(size)
        } else if self.user_declared_type.is_some() || !self.autodetected_types.is_empty() {
            None
        } else {
            self.access_type().and_then(|x| x.min_size().map(|x| Size::new(x.into())))
        }
    }

    pub fn alignment(&self) -> Option<u8> {
        if self.user_declared_type.is_some() || !self.autodetected_types.is_empty() {
            None
        } else {
            self.access_type().and_then(|x| x.min_alignment())
        }
    }

    pub fn reference_counter(&self) -> usize {
        self.referenced_by.len()
    }

    pub fn add_referenced_by(&mut self, specific_address: Vram) {
        self.referenced_by.push(specific_address);
    }

    pub fn set_access_type(&mut self, access_type: AccessType) {
        if access_type != AccessType::NONE {
            *self.access_types.entry(access_type).or_default() += 1;
        }
    }

    pub fn set_user_declared_type(&mut self, typ: SymbolType) {
        self.user_declared_type = Some(typ);
    }
    pub fn set_sym_type(&mut self, sym_type: SymbolType) {
        *self.autodetected_types.entry(sym_type).or_default() += 1;
    }

    pub fn set_user_declared_size(&mut self, size: Size) {
        self.user_declared_size = Some(size);
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
