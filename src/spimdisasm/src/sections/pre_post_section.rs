/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

use address_space::{AddressRange, Vram};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    context::Context,
    relocation::UserRelocs,
    sections::{
        ProcessableSection, SectionPostProcessError, SectionPreprocessed, SectionProcessed,
    },
    segments::OwnedSegmentNotFoundError,
    symbols::{InvalidRelocForSectionError, OwnedSymbolNotFoundError, UnalignedUserRelocError},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash)]
pub enum PrePostSection<Pre, Processed>
where
    Pre: SectionPreprocessed + ProcessableSection<Processed = Processed>,
    Processed: SectionProcessed,
{
    Invalid,
    Preprocessed(Pre),
    Processed(Processed),
}

impl<Pre, Processed> PrePostSection<Pre, Processed>
where
    Pre: SectionPreprocessed + ProcessableSection<Processed = Processed>,
    Processed: SectionProcessed,
{
    pub fn new(section: Pre) -> Self {
        Self::Preprocessed(section)
    }

    pub fn preprocessed(&self) -> Option<&Pre> {
        match &self {
            Self::Invalid => None,
            Self::Preprocessed(section) => Some(section),
            Self::Processed(..) => None,
        }
    }

    pub fn processed(&self) -> Option<&Processed> {
        match &self {
            Self::Invalid => None,
            Self::Preprocessed(..) => None,
            Self::Processed(section) => Some(section),
        }
    }

    pub fn post_process(
        &mut self,
        context: &mut Context,
        user_relocs: &UserRelocs,
    ) -> Result<(), PrePostSectionPostProcessError> {
        let section = core::mem::replace(self, Self::Invalid);

        let new_value = match section {
            Self::Invalid => return Err(PrePostSectionPostProcessError::InvalidState()),
            Self::Preprocessed(section) => section.post_process(context, user_relocs)?,
            Self::Processed(data_section_processed) => {
                return Err(PrePostSectionPostProcessError::AlreadyPostProcessed(
                    AlreadyPostProcessedError {
                        name: data_section_processed.name(),
                        vram: *data_section_processed.vram_range(),
                    },
                ))
            }
        };

        *self = Self::Processed(new_value);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub enum PrePostSectionPostProcessError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    OwnedSymbolNotFound(OwnedSymbolNotFoundError),
    UnalignedUserReloc(UnalignedUserRelocError),
    InvalidRelocForSection(InvalidRelocForSectionError),
    AlreadyPostProcessed(AlreadyPostProcessedError),
    InvalidState(),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct AlreadyPostProcessedError {
    pub(crate) name: Arc<str>,
    pub(crate) vram: AddressRange<Vram>,
}

impl fmt::Display for PrePostSectionPostProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrePostSectionPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                write!(f, "{owned_segment_not_found_error}")
            }
            PrePostSectionPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found) => {
                write!(f, "{owned_symbol_not_found}")
            }
            PrePostSectionPostProcessError::UnalignedUserReloc(unaligned_user_reloc_error) => {
                write!(f, "{unaligned_user_reloc_error}")
            }
            PrePostSectionPostProcessError::InvalidRelocForSection(
                invalid_reloc_for_section_error,
            ) => {
                write!(f, "{invalid_reloc_for_section_error}")
            }
            PrePostSectionPostProcessError::AlreadyPostProcessed(AlreadyPostProcessedError {
                name,
                vram,
            }) => {
                write!(
                    f,
                    "The section {} ({:?} {:?}) has already been post-processed.",
                    name,
                    vram.start(),
                    vram.end(),
                )
            }
            PrePostSectionPostProcessError::InvalidState() => {
                write!(f, "This section is somehow in an invalid state.")
            }
        }
    }
}

impl error::Error for PrePostSectionPostProcessError {
    // TODO: implement source for every other error type
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                Some(owned_segment_not_found_error)
            }
            Self::OwnedSymbolNotFound(owned_symbol_not_found_error) => {
                Some(owned_symbol_not_found_error)
            }
            Self::UnalignedUserReloc(unaligned_user_reloc_error) => {
                Some(unaligned_user_reloc_error)
            }
            Self::InvalidRelocForSection(invalid_reloc_for_section_error) => {
                Some(invalid_reloc_for_section_error)
            }
            Self::AlreadyPostProcessed(..) => None,
            Self::InvalidState() => None,
        }
    }
}

impl From<SectionPostProcessError> for PrePostSectionPostProcessError {
    fn from(value: SectionPostProcessError) -> Self {
        match value {
            SectionPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error) => {
                PrePostSectionPostProcessError::OwnedSegmentNotFound(owned_segment_not_found_error)
            }
            SectionPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found) => {
                PrePostSectionPostProcessError::OwnedSymbolNotFound(owned_symbol_not_found)
            }
            SectionPostProcessError::UnalignedUserReloc(unaligned_user_reloc_error) => {
                PrePostSectionPostProcessError::UnalignedUserReloc(unaligned_user_reloc_error)
            }
            SectionPostProcessError::InvalidRelocForSection(invalid_reloc_for_section_error) => {
                PrePostSectionPostProcessError::InvalidRelocForSection(
                    invalid_reloc_for_section_error,
                )
            }
        }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    use crate::{
        context::Context,
        sections::{
            python_bindings::py_sym_info::PySymInfo, ProcessableSection, Section,
            SectionPreprocessed, SectionProcessed,
        },
        symbols::{
            display::{NotPostProcessedYetError, SymDisplayError},
            Symbol,
        },
    };

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, PrePostSectionPostProcessError, PyRuntimeError);

    impl std::convert::From<super::PrePostSectionPostProcessError> for PyErr {
        fn from(err: super::PrePostSectionPostProcessError) -> PyErr {
            PrePostSectionPostProcessError::new_err(err.to_string())
        }
    }

    impl<Pre, Processed> super::PrePostSection<Pre, Processed>
    where
        Pre: SectionPreprocessed + ProcessableSection<Processed = Processed>,
        Processed: SectionProcessed,
    {
        pub fn sym_count(&self) -> usize {
            match &self {
                Self::Invalid => panic!(),
                Self::Preprocessed(section) => Section::symbol_list(section).len(),
                Self::Processed(section) => Section::symbol_list(section).len(),
            }
        }

        pub fn get_sym_info(&self, context: &Context, index: usize) -> Option<PySymInfo> {
            let metadata = match &self {
                Self::Invalid => panic!(),
                Self::Preprocessed(section) => Section::symbol_list(section)
                    .get(index)
                    .map(|x| x.find_own_metadata(context)),
                Self::Processed(section) => Section::symbol_list(section)
                    .get(index)
                    .map(|x| x.find_own_metadata(context)),
            };

            metadata.map(PySymInfo::new)
        }

        pub fn set_sym_name(&mut self, context: &mut Context, index: usize, new_name: String) {
            let metadata = match &self {
                Self::Invalid => panic!(),
                Self::Preprocessed(section) => Section::symbol_list(section)
                    .get(index)
                    .map(|x| x.find_own_metadata_mut(context)),
                Self::Processed(section) => Section::symbol_list(section)
                    .get(index)
                    .map(|x| x.find_own_metadata_mut(context)),
            };

            if let Some(metadata) = metadata {
                metadata.set_user_declared_name(new_name.into());
            }
        }

        pub fn display_sym<Sym, FS, FD>(
            &self,
            get_sym: FS,
            display_sym: FD,
        ) -> Result<Option<String>, SymDisplayError>
        where
            FS: FnOnce(&Processed) -> Option<&Sym>,
            FD: FnOnce(&Sym) -> Result<String, SymDisplayError>,
        {
            let section = match &self {
                Self::Invalid => panic!(),
                Self::Preprocessed(section) => {
                    return Err(SymDisplayError::NotPostProcessedYet(
                        NotPostProcessedYetError {
                            name: section.name().to_string(),
                            vram_start: section.vram_range().start(),
                            vram_end: section.vram_range().end(),
                        },
                    ))
                }
                Self::Processed(section) => section,
            };

            let sym = get_sym(section);

            Ok(if let Some(sym) = sym {
                Some(display_sym(sym)?.to_string())
            } else {
                None
            })
        }
    }
}
