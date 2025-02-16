/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Size},
    collections::addended_ordered_map::FindSettings,
    context::Context,
    metadata::SymbolType,
    relocation::python_bindings::py_user_relocs::PyUserRelocs,
    sections::{
        before_proc::ExecutableSection, processed::ExecutableSectionProcessed, Section,
        SectionPostProcessError,
    },
    symbols::{
        display::{FunctionDisplaySettings, SymDisplayError},
        Symbol,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
enum PyExecutableSectionInner {
    Invalid,
    Preprocessed(ExecutableSection),
    Processed(ExecutableSectionProcessed),
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "spimdisasm", name = "ExecutableSection")
)]
pub struct PyExecutableSection {
    inner: PyExecutableSectionInner,
}

impl PyExecutableSection {
    pub fn new(section: ExecutableSection) -> Self {
        Self {
            inner: PyExecutableSectionInner::Preprocessed(section),
        }
    }

    pub fn unwrap_processed(&self) -> &ExecutableSectionProcessed {
        match &self.inner {
            PyExecutableSectionInner::Invalid => panic!(),
            PyExecutableSectionInner::Preprocessed(..) => panic!(),
            PyExecutableSectionInner::Processed(section) => section,
        }
    }
}

#[pymethods]
impl PyExecutableSection {
    #[pyo3(name = "post_process")]
    fn py_post_process(
        &mut self,
        context: &mut Context,
        user_relocs: &PyUserRelocs,
    ) -> Result<(), SectionPostProcessError> {
        let section = core::mem::replace(&mut self.inner, PyExecutableSectionInner::Invalid);

        let new_value = match section {
            PyExecutableSectionInner::Invalid => {
                return Err(SectionPostProcessError::InvalidState())
            }
            PyExecutableSectionInner::Preprocessed(section) => {
                section.post_process(context, user_relocs.inner())?
            }
            PyExecutableSectionInner::Processed(section) => {
                return Err(SectionPostProcessError::AlreadyPostProcessed {
                    name: section.name().to_string(),
                    vram_start: section.vram_range().start(),
                    vram_end: section.vram_range().end(),
                })
            }
        };

        self.inner = PyExecutableSectionInner::Processed(new_value);
        Ok(())
    }

    #[pyo3(name = "sym_count")]
    pub fn py_sym_count(&self) -> usize {
        match &self.inner {
            PyExecutableSectionInner::Invalid => panic!(),
            PyExecutableSectionInner::Preprocessed(section) => section.symbol_list().len(),
            PyExecutableSectionInner::Processed(section) => section.symbol_list().len(),
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
            PyExecutableSectionInner::Invalid => panic!(),
            PyExecutableSectionInner::Preprocessed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata(context)),
            PyExecutableSectionInner::Processed(section) => section
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
                        .map(|x| x.inner().to_string())
                }),
            )
        })
    }

    #[pyo3(name = "set_sym_name")]
    pub fn py_set_sym_name(&mut self, context: &mut Context, index: usize, new_name: String) {
        let metadata = match &self.inner {
            PyExecutableSectionInner::Invalid => panic!(),
            PyExecutableSectionInner::Preprocessed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata_mut(context)),
            PyExecutableSectionInner::Processed(section) => section
                .symbol_list()
                .get(index)
                .map(|x| x.find_own_metadata_mut(context)),
        };

        if let Some(metadata) = metadata {
            metadata.set_user_declared_name(new_name.into());
        }
    }

    #[pyo3(name = "display_sym")]
    pub fn py_display_sym(
        &self,
        context: &Context,
        index: usize,
        settings: &FunctionDisplaySettings,
    ) -> Result<Option<String>, SymDisplayError> {
        let sym = match &self.inner {
            PyExecutableSectionInner::Invalid => panic!(),
            PyExecutableSectionInner::Preprocessed(section) => {
                return Err(SymDisplayError::NotPostProcessedYet {
                    name: section.name().to_string(),
                    vram_start: section.vram_range().start(),
                    vram_end: section.vram_range().end(),
                })
            }
            PyExecutableSectionInner::Processed(section) => section.functions().get(index),
        };

        Ok(if let Some(sym) = sym {
            Some(sym.display(context, settings)?.to_string())
        } else {
            None
        })
    }

    #[pyo3(name = "label_count_for_sym")]
    pub fn py_label_count_for_sym(&self, sym_index: usize) -> usize {
        let sym = self.unwrap_processed().functions().get(sym_index);

        if let Some(sym) = sym {
            sym.labels().len()
        } else {
            0
        }
    }

    #[pyo3(name = "get_label_info")]
    pub fn py_get_label_info(
        &self,
        context: &Context,
        sym_index: usize,
        label_index: usize,
    ) -> Option<(
        u32,
        Option<Rom>,
        Option<SymbolType>,
        Option<Size>,
        bool,
        usize,
        Option<String>,
    )> {
        let (sym, parent_segment_info) = {
            let section = self.unwrap_processed();
            (
                section.functions().get(sym_index),
                section.parent_segment_info(),
            )
        };

        if let Some(sym) = sym {
            if let Some(label_vram) = sym.labels().get(label_index) {
                let metadata = context
                    .find_owned_segment(parent_segment_info)
                    .unwrap()
                    .find_symbol(*label_vram, FindSettings::new(false))
                    .unwrap();

                Some((
                    metadata.vram().inner(),
                    metadata.rom(),
                    metadata.sym_type(),
                    metadata.size(),
                    metadata.is_defined(),
                    metadata.reference_counter(),
                    metadata.parent_metadata().and_then(|x| {
                        x.parent_segment_info()
                            .overlay_category_name()
                            .map(|x| x.inner().to_string())
                    }),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[pyo3(name = "set_label_name")]
    pub fn py_set_label_name(
        &mut self,
        context: &mut Context,
        sym_index: usize,
        label_index: usize,
        new_name: String,
    ) {
        let (sym, parent_segment_info) = match &self.inner {
            PyExecutableSectionInner::Invalid => panic!(),
            PyExecutableSectionInner::Preprocessed(..) => panic!(),
            PyExecutableSectionInner::Processed(section) => (
                section.functions().get(sym_index),
                section.parent_segment_info(),
            ),
        };

        if let Some(sym) = sym {
            if let Some(label_vram) = sym.labels().get(label_index) {
                let metadata = context
                    .find_owned_segment_mut(parent_segment_info)
                    .unwrap()
                    .find_symbol_mut(*label_vram, FindSettings::new(false))
                    .unwrap();

                metadata.set_user_declared_name(new_name.into());
            }
        }
    }
}
