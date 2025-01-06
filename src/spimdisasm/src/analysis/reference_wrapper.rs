/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::cmp::Ordering;

use crate::{
    addresses::{AddressRange, Rom, Size, Vram},
    collections::addended_ordered_map::{self, FindSettings},
    metadata::{SegmentMetadata, SymbolMetadata, SymbolType},
};

use super::{Preheater, ReferencedAddress};

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd)]
pub enum ReferenceWrapper<'seg, 'addr> {
    Metadata(&'seg SymbolMetadata),
    Address(&'addr ReferencedAddress),
    Both(&'seg SymbolMetadata, &'addr ReferencedAddress),
}

impl<'seg, 'addr> ReferenceWrapper<'seg, 'addr> {
    pub(crate) fn find(
        owned_segment: &'seg SegmentMetadata,
        preheater: &'addr Preheater,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<Self> {
        let metadata = owned_segment.find_symbol(vram, settings);
        let reference = preheater.references().find(&vram, settings);

        match (metadata, reference) {
            (None, None) => None,
            (None, Some(reference)) => Some(ReferenceWrapper::Address(reference)),
            (Some(metadata), None) => Some(ReferenceWrapper::Metadata(metadata)),
            (Some(metadata), Some(reference)) => Some(ReferenceWrapper::Both(metadata, reference)),
        }
    }

    pub(crate) fn range(
        owned_segment: &'seg SegmentMetadata,
        preheater: &'addr Preheater,
        vram_range: AddressRange<Vram>,
    ) -> Range<'seg, 'addr> {
        Range::new(
            owned_segment.symbols().range(vram_range),
            preheater.references().range(vram_range),
        )
    }
}

impl ReferenceWrapper<'_, '_> {
    pub fn vram(&self) -> Vram {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.vram(),
            ReferenceWrapper::Address(address) => address.vram(),
            ReferenceWrapper::Both(metadata, _address) => metadata.vram(),
        }
    }

    pub fn rom(&self) -> Option<Rom> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.rom(),
            ReferenceWrapper::Address(_address) => None,
            ReferenceWrapper::Both(metadata, _address) => metadata.rom(),
        }
    }

    pub fn user_declared_size(&self) -> Option<Size> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.user_declared_size(),
            ReferenceWrapper::Address(_address) => None,
            ReferenceWrapper::Both(metadata, _address) => metadata.user_declared_size(),
        }
    }

    pub fn sym_type(&self) -> Option<SymbolType> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.sym_type(),
            ReferenceWrapper::Address(address) => address.sym_type(),
            ReferenceWrapper::Both(metadata, address) => {
                metadata.sym_type().or_else(|| address.sym_type())
            }
        }
    }
    pub fn user_declared_type(&self) -> Option<SymbolType> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.user_declared_type(),
            ReferenceWrapper::Address(_address) => None,
            ReferenceWrapper::Both(metadata, _address) => metadata.user_declared_type(),
        }
    }
    pub fn autodetected_type(&self) -> Option<SymbolType> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.autodetected_type(),
            ReferenceWrapper::Address(address) => address.sym_type(),
            ReferenceWrapper::Both(metadata, address) => {
                metadata.autodetected_type().or_else(|| address.sym_type())
            }
        }
    }

    pub fn reference_counter(&self) -> usize {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.reference_counter(),
            ReferenceWrapper::Address(address) => address.reference_counter(),
            ReferenceWrapper::Both(metadata, address) => {
                let counter = metadata.reference_counter();
                if counter != 0 {
                    counter
                } else {
                    address.reference_counter()
                }
            }
        }
    }

    pub fn is_trustable_function(&self) -> bool {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.is_trustable_function(),
            ReferenceWrapper::Address(address) => address.is_trustable_function(),
            ReferenceWrapper::Both(metadata, _address) => metadata.is_trustable_function(),
        }
    }
}

impl<'seg> From<&'seg SymbolMetadata> for ReferenceWrapper<'seg, '_> {
    fn from(value: &'seg SymbolMetadata) -> Self {
        ReferenceWrapper::Metadata(value)
    }
}

impl<'addr> From<&'addr ReferencedAddress> for ReferenceWrapper<'_, 'addr> {
    fn from(value: &'addr ReferencedAddress) -> Self {
        ReferenceWrapper::Address(value)
    }
}

impl<'seg, 'addr> From<(&'seg SymbolMetadata, &'addr ReferencedAddress)>
    for ReferenceWrapper<'seg, 'addr>
{
    fn from(value: (&'seg SymbolMetadata, &'addr ReferencedAddress)) -> Self {
        ReferenceWrapper::Both(value.0, value.1)
    }
}

#[must_use]
pub(crate) struct Range<'seg, 'addr> {
    metadata_range: addended_ordered_map::Range<'seg, Vram, SymbolMetadata>,
    references_range: addended_ordered_map::Range<'addr, Vram, ReferencedAddress>,

    buffer_metadata: Option<(&'seg Vram, &'seg SymbolMetadata)>,
    buffer_reference: Option<(&'addr Vram, &'addr ReferencedAddress)>,
}

impl<'seg, 'addr> Range<'seg, 'addr> {
    fn new(
        metadata_range: addended_ordered_map::Range<'seg, Vram, SymbolMetadata>,
        references_range: addended_ordered_map::Range<'addr, Vram, ReferencedAddress>,
    ) -> Self {
        Self {
            metadata_range,
            references_range,
            buffer_metadata: None,
            buffer_reference: None,
        }
    }

    fn pick_one_and_store_the_other(
        &mut self,
        metadata_temp: Option<(&'seg Vram, &'seg SymbolMetadata)>,
        reference_temp: Option<(&'addr Vram, &'addr ReferencedAddress)>,
    ) -> Option<ReferenceWrapper<'seg, 'addr>> {
        match (metadata_temp, reference_temp) {
            (None, None) => None,
            (None, Some(x)) => Some(x.1.into()),
            (Some(x), None) => Some(x.1.into()),
            (Some(metadata), Some(reference)) => match metadata.0.cmp(reference.0) {
                Ordering::Less => {
                    self.buffer_reference = Some(reference);
                    Some(metadata.1.into())
                }
                Ordering::Equal => Some((metadata.1, reference.1).into()),
                Ordering::Greater => {
                    self.buffer_metadata = Some(metadata);
                    Some(reference.1.into())
                }
            },
        }
    }
}

impl<'seg, 'addr> Iterator for Range<'seg, 'addr> {
    type Item = ReferenceWrapper<'seg, 'addr>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match (self.buffer_metadata.take(), self.buffer_reference.take()) {
            (None, None) => {
                let metadata_temp = self.metadata_range.next();
                let reference_temp = self.references_range.next();

                self.pick_one_and_store_the_other(metadata_temp, reference_temp)
            }
            (None, Some(reference_temp)) => {
                let metadata_temp = self.metadata_range.next();

                self.pick_one_and_store_the_other(metadata_temp, Some(reference_temp))
            }
            (Some(metadata_temp), None) => {
                let reference_temp = self.references_range.next();

                self.pick_one_and_store_the_other(Some(metadata_temp), reference_temp)
            }
            (Some(metadata_temp), Some(reference_temp)) => {
                // I don't think this is possible
                self.pick_one_and_store_the_other(Some(metadata_temp), Some(reference_temp))
            }
        };

        result
    }
}
