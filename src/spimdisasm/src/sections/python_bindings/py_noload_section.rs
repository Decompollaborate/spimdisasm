/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Size},
    context::Context,
    metadata::SymbolType,
    sections::{
        preprocessed::NoloadSection, processed::NoloadSectionProcessed, Section,
        SectionPostProcessError,
    },
    symbols::{
        display::{SymDisplayError, SymNoloadDisplaySettings},
        Symbol,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
enum PyNoloadSectionInner {
    Invalid,
    Preprocessed(NoloadSection),
    Processed(NoloadSectionProcessed),
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", name = "DataSection"))]
pub struct PyNoloadSection {
    inner: PyNoloadSectionInner,
}

impl PyNoloadSection {
    pub fn new(section: NoloadSection) -> Self {
        Self {
            inner: PyNoloadSectionInner::Preprocessed(section),
        }
    }
}

#[pymethods]
impl PyNoloadSection {
    #[pyo3(name = "post_process")]
    fn py_post_process(&mut self, context: &mut Context) -> Result<(), SectionPostProcessError> {
        let section = core::mem::replace(&mut self.inner, PyNoloadSectionInner::Invalid);

        let new_value = match section {
            PyNoloadSectionInner::Invalid => return Err(SectionPostProcessError::InvalidState()),
            PyNoloadSectionInner::Preprocessed(data_section) => {
                data_section.post_process(context)?
            }
            PyNoloadSectionInner::Processed(data_section_processed) => {
                return Err(SectionPostProcessError::AlreadyPostProcessed {
                    name: data_section_processed.name().to_string(),
                    vram_start: data_section_processed.vram_range().start(),
                    vram_end: data_section_processed.vram_range().end(),
                })
            }
        };

        self.inner = PyNoloadSectionInner::Processed(new_value);
        Ok(())
    }

    #[pyo3(name = "sym_count")]
    pub fn py_sym_count(&self) -> usize {
        match &self.inner {
            PyNoloadSectionInner::Invalid => panic!(),
            PyNoloadSectionInner::Preprocessed(section) => section.symbol_list().len(),
            PyNoloadSectionInner::Processed(section) => section.symbol_list().len(),
        }
    }

    #[pyo3(name = "get_sym_info")]
    pub fn py_get_sym_info(
        &self,
        context: &Context,
        index: usize,
    ) -> Option<(
        u32,
        Option<Rom>,
        Option<SymbolType>,
        Option<Size>,
        bool,
        usize,
        Option<String>,
    )> {
        let metadata = match &self.inner {
            PyNoloadSectionInner::Invalid => panic!(),
            PyNoloadSectionInner::Preprocessed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata(context)),
            PyNoloadSectionInner::Processed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata(context)),
        };

        metadata.map(|x| {
            (
                x.vram().inner(),
                x.rom(),
                x.sym_type(),
                x.size(),
                x.is_defined(),
                x.reference_counter(),
                x.parent_metadata().and_then(|x| {
                    x.parent_segment_info()
                        .overlay_category_name()
                        .map(|x| x.inner().to_owned())
                }),
            )
        })
    }

    #[pyo3(name = "set_sym_name")]
    pub fn py_set_sym_name(&mut self, context: &mut Context, index: usize, new_name: String) {
        let metadata = match &self.inner {
            PyNoloadSectionInner::Invalid => panic!(),
            PyNoloadSectionInner::Preprocessed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata_mut(context)),
            PyNoloadSectionInner::Processed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata_mut(context)),
        };

        if let Some(metadata) = metadata {
            *metadata.user_declared_name_mut() = Some(new_name);
        }
    }

    #[pyo3(name = "display_sym")]
    pub fn py_display_sym(
        &self,
        context: &Context,
        index: usize,
        settings: &SymNoloadDisplaySettings,
    ) -> Result<Option<String>, SymDisplayError> {
        let sym = match &self.inner {
            PyNoloadSectionInner::Invalid => panic!(),
            PyNoloadSectionInner::Preprocessed(section) => {
                return Err(SymDisplayError::NotPostProcessedYet {
                    name: section.name().to_string(),
                    vram_start: section.vram_range().start(),
                    vram_end: section.vram_range().end(),
                })
            }
            PyNoloadSectionInner::Processed(section) => section.noload_symbols().get(index),
        };

        Ok(if let Some(sym) = sym {
            Some(sym.display(context, settings)?.to_string())
        } else {
            None
        })
    }

    #[pyo3(name = "label_count_for_sym")]
    pub fn py_label_count_for_sym(&self, _sym_index: usize) -> usize {
        0
    }
}
