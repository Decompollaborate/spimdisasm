/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::cmp::Ordering;

use rabbitizer::access_type::AccessType;

use crate::{
    addresses::{AddressRange, Rom, Size, Vram},
    collections::{
        addended_ordered_map::{self, AddendedOrderedMap, FindSettings},
        unordered_map::UnorderedMap,
    },
    metadata::{GeneratedBy, SymbolMetadata, SymbolType},
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
        symbols: &'seg AddendedOrderedMap<Vram, SymbolMetadata>,
        preheater: &'addr Preheater,
        vram: Vram,
        settings: FindSettings,
    ) -> Option<Self> {
        let metadata = symbols.find(&vram, settings);
        let reference = preheater.references().find(&vram, settings);

        match (metadata, reference) {
            (None, None) => None,
            (None, Some(reference)) => Some(ReferenceWrapper::Address(reference)),
            (Some(metadata), None) => Some(ReferenceWrapper::Metadata(metadata)),
            (Some(metadata), Some(reference)) => {
                // Return the symbol that is nearest to the actual address

                let metadata_vram = metadata.vram();
                let reference_vram = reference.vram();

                if metadata_vram == reference_vram {
                    Some(ReferenceWrapper::Both(metadata, reference))
                } else if metadata_vram == vram {
                    Some(ReferenceWrapper::Metadata(metadata))
                } else if reference_vram == vram {
                    Some(ReferenceWrapper::Address(reference))
                } else if metadata_vram > vram {
                    if reference_vram < vram || metadata_vram < reference_vram {
                        Some(ReferenceWrapper::Metadata(metadata))
                    } else {
                        Some(ReferenceWrapper::Address(reference))
                    }
                } else if reference_vram > vram {
                    Some(ReferenceWrapper::Address(reference))
                } else if metadata_vram > reference_vram {
                    Some(ReferenceWrapper::Metadata(metadata))
                } else {
                    Some(ReferenceWrapper::Address(reference))
                }
            }
        }
    }

    pub(crate) fn range(
        symbols: &'seg AddendedOrderedMap<Vram, SymbolMetadata>,
        preheater: &'addr Preheater,
        vram_range: AddressRange<Vram>,
    ) -> Range<'seg, 'addr> {
        Range::new(
            symbols.range(vram_range),
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

    pub(crate) fn is_user_declared(&self) -> bool {
        match self {
            ReferenceWrapper::Metadata(metadata) => {
                metadata.generated_by() == GeneratedBy::UserDeclared
            }
            ReferenceWrapper::Address(address) => address.user_declared(),
            ReferenceWrapper::Both(metadata, _address) => {
                metadata.generated_by() == GeneratedBy::UserDeclared
            }
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

    pub fn size(&self) -> Option<Size> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.size(),
            ReferenceWrapper::Address(address) => address.size(),
            ReferenceWrapper::Both(metadata, _address) => metadata.size(),
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

    pub(crate) fn all_access_types(&self) -> &UnorderedMap<AccessType, u32> {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.all_access_types(),
            ReferenceWrapper::Address(address) => address.all_access_types(),
            ReferenceWrapper::Both(metadata, address) => {
                let accesses = metadata.all_access_types();
                if !accesses.is_empty() {
                    accesses
                } else {
                    address.all_access_types()
                }
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

    pub(crate) fn is_trustable_function(&self) -> bool {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.is_trustable_function(),
            ReferenceWrapper::Address(address) => address.is_trustable_function(),
            ReferenceWrapper::Both(metadata, _address) => metadata.is_trustable_function(),
        }
    }

    pub fn table_labels(&self) -> &[Vram] {
        match self {
            ReferenceWrapper::Metadata(_metadata) => &[],
            ReferenceWrapper::Address(address) => address.table_labels(),
            ReferenceWrapper::Both(_metadata, address) => address.table_labels(),
        }
    }

    #[must_use]
    pub(crate) fn add_gp_to_pointed_data(&self) -> bool {
        match self {
            ReferenceWrapper::Metadata(metadata) => metadata.add_gp_to_pointed_data(),
            ReferenceWrapper::Address(_address) => false, // TODO
            ReferenceWrapper::Both(metadata, _address) => metadata.add_gp_to_pointed_data(),
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
