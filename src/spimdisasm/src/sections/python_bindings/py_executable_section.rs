/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;

use crate::{
    context::Context,
    metadata::LabelType,
    relocation::python_bindings::py_user_relocs::PyUserRelocs,
    sections::{
        before_proc::ExecutableSection,
        processed::ExecutableSectionProcessed,
        python_bindings::{pre_post_section::PrePostSection, py_sym_info::PySymInfo},
        Section, SectionPostProcessError,
    },
    symbols::{
        display::{FunctionDisplaySettings, SymDataDisplaySettings, SymDisplayError},
        processed::EitherFuncDataSymProcessed,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "spimdisasm", name = "ExecutableSection", from_py_object)
)]
pub struct PyExecutableSection {
    inner: PrePostSection<ExecutableSection, ExecutableSectionProcessed>,
}

impl PyExecutableSection {
    pub fn new(section: ExecutableSection) -> Self {
        Self {
            inner: PrePostSection::new(section),
        }
    }

    pub fn unwrap_processed(&self) -> &ExecutableSectionProcessed {
        self.inner.unwrap_processed()
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
        self.inner.post_process(
            context,
            user_relocs.inner(),
            |section, context, user_relocs| section.post_process(context, user_relocs),
        )
    }

    #[pyo3(name = "sym_count")]
    pub fn py_sym_count(&self) -> usize {
        self.inner.sym_count()
    }

    #[pyo3(name = "get_sym_info")]
    pub fn py_get_sym_info(&self, context: &Context, index: usize) -> Option<PySymInfo> {
        self.inner.get_sym_info(context, index)
    }

    #[pyo3(name = "set_sym_name")]
    pub fn py_set_sym_name(&mut self, context: &mut Context, index: usize, new_name: String) {
        self.inner.set_sym_name(context, index, new_name)
    }

    #[pyo3(name = "display_sym")]
    pub fn py_display_sym(
        &self,
        context: &Context,
        index: usize,
        settings: &FunctionDisplaySettings,
    ) -> Result<Option<String>, SymDisplayError> {
        let data_settings = SymDataDisplaySettings::new();
        self.inner.display_sym(
            |section| section.symbols().get(index),
            |sym| {
                sym.display(context, settings, &data_settings)
                    .map(|x| x.to_string())
            },
        )
    }

    #[pyo3(name = "label_count_for_sym")]
    pub fn py_label_count_for_sym(&self, sym_index: usize) -> usize {
        let sym = self.unwrap_processed().symbols().get(sym_index);

        if let Some(EitherFuncDataSymProcessed::Func(sym)) = sym {
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
    ) -> Option<(u32, Option<u32>, LabelType, bool, usize)> {
        let (sym, parent_segment_info) = {
            let section = self.unwrap_processed();
            (
                section.symbols().get(sym_index),
                section.parent_segment_info(),
            )
        };

        if let Some(EitherFuncDataSymProcessed::Func(sym)) = sym {
            if let Some(label_vram) = sym.labels().get(label_index) {
                let metadata = context
                    .find_owned_segment(parent_segment_info)
                    .unwrap()
                    .find_label(*label_vram)
                    .unwrap();

                Some((
                    metadata.vram().inner(),
                    metadata.rom().map(|x| x.inner()),
                    metadata.label_type(),
                    metadata.is_defined(),
                    metadata.reference_counter(),
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
        let section = self.inner.unwrap_processed();
        let sym = section.symbols().get(sym_index);
        let parent_segment_info = section.parent_segment_info();

        if let Some(EitherFuncDataSymProcessed::Func(sym)) = sym {
            if let Some(label_vram) = sym.labels().get(label_index) {
                let metadata = context
                    .find_owned_segment_mut(parent_segment_info)
                    .unwrap()
                    .find_label_mut(*label_vram)
                    .unwrap();

                metadata.set_user_declared_name(new_name.into());
            }
        }
    }
}
