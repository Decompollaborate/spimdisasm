/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;

use crate::{
    context::Context,
    relocation::python_bindings::py_user_relocs::PyUserRelocs,
    sections::{
        before_proc::DataSection,
        processed::DataSectionProcessed,
        python_bindings::{pre_post_section::PrePostSection, py_sym_info::PySymInfo},
        SectionPostProcessError,
    },
    symbols::display::{SymDataDisplaySettings, SymDisplayError},
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
#[pyclass(module = "spimdisasm", name = "DataSection", from_py_object)]
pub struct PyDataSection {
    inner: PrePostSection<DataSection, DataSectionProcessed>,
}

impl PyDataSection {
    pub fn new(section: DataSection) -> Self {
        Self {
            inner: PrePostSection::new(section),
        }
    }

    pub fn unwrap_processed(&self) -> &DataSectionProcessed {
        self.inner.unwrap_processed()
    }
}

#[pymethods]
impl PyDataSection {
    #[pyo3(name = "post_process")]
    fn py_post_process(
        &mut self,
        context: &mut Context,
        user_relocs: &PyUserRelocs,
    ) -> Result<(), SectionPostProcessError> {
        self.inner.post_process(
            context,
            user_relocs.inner(),
            |data_section, context, user_relocs| data_section.post_process(context, user_relocs),
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
        settings: &SymDataDisplaySettings,
    ) -> Result<Option<String>, SymDisplayError> {
        self.inner.display_sym(
            |section| section.data_symbols().get(index),
            |sym| sym.display(context, settings).map(|x| x.to_string()),
        )
    }

    #[pyo3(name = "label_count_for_sym")]
    pub fn py_label_count_for_sym(&self, _sym_index: usize) -> usize {
        0
    }
}
