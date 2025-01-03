/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{Size, Vram},
    collections::addended_ordered_map::FindSettings,
    metadata::{SegmentMetadata, SymbolMetadata, SymbolType},
};

use super::{Preheater, ReferencedAddress};

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd)]
pub enum ReferenceWrapper<'seg, 'addr> {
    Metadata(&'seg SymbolMetadata),
    Address(&'addr ReferencedAddress),
}

impl<'seg, 'addr> ReferenceWrapper<'seg, 'addr> {
    pub fn find(
        owned_segment: &'seg SegmentMetadata,
        preheater: &'addr Preheater,
        vram: Vram,
    ) -> Option<Self> {
        if let Some(owned) =
            owned_segment.find_symbol(vram, FindSettings::default().with_allow_addend(false))
        {
            Some(ReferenceWrapper::Metadata(owned))
        } else {
            preheater
                .references()
                .get(&vram)
                .map(ReferenceWrapper::Address)
        }
    }

    pub fn find_with_addend(
        owned_segment: &'seg SegmentMetadata,
        preheater: &'addr Preheater,
        vram: Vram,
    ) -> Option<Self> {
        if let Some(owned) =
            owned_segment.find_symbol(vram, FindSettings::default().with_allow_addend(true))
        {
            Some(ReferenceWrapper::Metadata(owned))
        } else {
            let mut range = preheater.references().range(..=vram);

            if let Some((sym_vram, sym)) = range.next_back() {
                if *sym_vram == vram {
                    Some(ReferenceWrapper::Address(sym))
                } else {
                    sym.size().and_then(|siz| {
                        if vram < *sym_vram + siz {
                            Some(ReferenceWrapper::Address(sym))
                        } else {
                            None
                        }
                    })
                }
            } else {
                None
            }
        }
    }
}

impl ReferenceWrapper<'_, '_> {
    pub fn vram(&self) -> Vram {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.vram(),
            ReferenceWrapper::Address(address) => address.vram(),
        }
    }

    pub fn user_declared_size(&self) -> Option<Size> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.user_declared_size(),
            ReferenceWrapper::Address(_address) => None,
        }
    }

    pub fn sym_type(&self) -> Option<SymbolType> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.sym_type(),
            ReferenceWrapper::Address(address) => address.sym_type(),
        }
    }
    pub fn user_declared_type(&self) -> Option<SymbolType> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.user_declared_type(),
            ReferenceWrapper::Address(_address) => None,
        }
    }
    pub fn autodetected_type(&self) -> Option<SymbolType> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.autodetected_type(),
            ReferenceWrapper::Address(address) => address.sym_type(),
        }
    }

    pub fn reference_counter(&self) -> usize {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.reference_counter(),
            ReferenceWrapper::Address(address) => address.reference_counter(),
        }
    }
}

impl<'seg> From<&'seg SymbolMetadata> for ReferenceWrapper<'seg, '_> {
    fn from(value: &'seg SymbolMetadata) -> Self {
        ReferenceWrapper::Metadata(value)
    }
}
