/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::Rom, context::OwnedSegmentNotFoundError, relocation::RelocationType,
    section_type::SectionType,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct OwnedSymbolNotFoundError {}
impl OwnedSymbolNotFoundError {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
impl fmt::Display for OwnedSymbolNotFoundError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
impl error::Error for OwnedSymbolNotFoundError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UnalignedUserRelocError {
    reloc_rom: Rom,
    reloc_type: RelocationType,
}
impl UnalignedUserRelocError {
    pub(crate) fn new(reloc_rom: Rom, reloc_type: RelocationType) -> Self {
        Self {
            reloc_rom,
            reloc_type,
        }
    }
}
impl fmt::Display for UnalignedUserRelocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to set user reloc `{}`, because the given rom address for the reloc ({:?}) is not word-aligned", self.reloc_type.name(), self.reloc_rom)
    }
}
impl error::Error for UnalignedUserRelocError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct InvalidRelocForSectionError {
    reloc_rom: Rom,
    reloc_type: RelocationType,
    section_type: SectionType,
}
impl InvalidRelocForSectionError {
    pub(crate) fn new(
        reloc_rom: Rom,
        reloc_type: RelocationType,
        section_type: SectionType,
    ) -> Self {
        Self {
            reloc_rom,
            reloc_type,
            section_type,
        }
    }
}
impl fmt::Display for InvalidRelocForSectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to set user reloc `{}` (rom {:?}), because this kind of relocation type is not valid for the section type `{:?}`", self.reloc_type.name(), self.reloc_rom, self.section_type)
    }
}
impl error::Error for InvalidRelocForSectionError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SymbolPostProcessError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    OwnedSymbolNotFound(OwnedSymbolNotFoundError),
    UnalignedUserReloc(UnalignedUserRelocError),
    InvalidRelocForSection(InvalidRelocForSectionError),
}

impl fmt::Display for SymbolPostProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{}", owned_segment_not_found_error)
            }
            SymbolPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found) => {
                write!(f, "{}", owned_symbol_not_found)
            }
            SymbolPostProcessError::UnalignedUserReloc(unaligned_user_reloc_error) => {
                write!(f, "{}", unaligned_user_reloc_error)
            }
            SymbolPostProcessError::InvalidRelocForSection(invalid_reloc_for_section_error) => {
                write!(f, "{}", invalid_reloc_for_section_error)
            }
        }
    }
}

impl error::Error for SymbolPostProcessError {}

impl From<OwnedSegmentNotFoundError> for SymbolPostProcessError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SymbolPostProcessError::OwnedSegmentNotFound(value)
    }
}

impl From<OwnedSymbolNotFoundError> for SymbolPostProcessError {
    fn from(value: OwnedSymbolNotFoundError) -> Self {
        SymbolPostProcessError::OwnedSymbolNotFound(value)
    }
}
