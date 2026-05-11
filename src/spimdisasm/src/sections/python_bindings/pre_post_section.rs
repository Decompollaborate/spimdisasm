/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::BTreeMap;

use address_space::Rom;

use crate::{
    context::Context,
    relocation::RelocationInfo,
    sections::{
        python_bindings::py_sym_info::PySymInfo,
        section_post_process_error::AlreadyPostProcessedError, Section, SectionPostProcessError,
        SectionPreprocessed, SectionProcessed,
    },
    symbols::{
        display::{NotPostProcessedYetError, SymDisplayError},
        Symbol,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub enum PrePostSection<Pre, Processed>
where
    Pre: SectionPreprocessed,
    Processed: SectionProcessed,
{
    Invalid,
    Preprocessed(Pre),
    Processed(Processed),
}

impl<Pre, Processed> PrePostSection<Pre, Processed>
where
    Pre: SectionPreprocessed,
    Processed: SectionProcessed,
{
    pub fn new(section: Pre) -> Self {
        Self::Preprocessed(section)
    }

    pub fn unwrap_processed(&self) -> &Processed {
        match &self {
            Self::Invalid => panic!(),
            Self::Preprocessed(..) => panic!(),
            Self::Processed(section) => section,
        }
    }

    pub fn post_process<F>(
        &mut self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
        post_process: F,
    ) -> Result<(), SectionPostProcessError>
    where
        F: FnOnce(
            Pre,
            &mut Context,
            &BTreeMap<Rom, RelocationInfo>,
        ) -> Result<Processed, SectionPostProcessError>,
    {
        let section = core::mem::replace(self, Self::Invalid);

        let new_value = match section {
            Self::Invalid => return Err(SectionPostProcessError::InvalidState()),
            Self::Preprocessed(section) => post_process(section, context, user_relocs)?,
            Self::Processed(data_section_processed) => {
                return Err(SectionPostProcessError::AlreadyPostProcessed(
                    AlreadyPostProcessedError {
                        name: data_section_processed.name().to_string(),
                        vram_start: data_section_processed.vram_range().start(),
                        vram_end: data_section_processed.vram_range().end(),
                    },
                ))
            }
        };

        *self = Self::Processed(new_value);
        Ok(())
    }

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
