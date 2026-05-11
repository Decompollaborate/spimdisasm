/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use pyo3::prelude::*;

use crate::{
    context::Context,
    relocation::python_bindings::py_user_relocs::PyUserRelocs,
    sections::{
        before_proc::NobitsSection,
        processed::NobitsSectionProcessed,
        python_bindings::{pre_post_section::PrePostSection, py_sym_info::PySymInfo},
        SectionPostProcessError,
    },
    symbols::display::{SymDisplayError, SymNobitsDisplaySettings},
};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "spimdisasm", name = "NobitsSection", from_py_object)
)]
pub struct PyNobitsSection {
    inner: PrePostSection<NobitsSection, NobitsSectionProcessed>,
}

impl PyNobitsSection {
    pub fn new(section: NobitsSection) -> Self {
        Self {
            inner: PrePostSection::new(section),
        }
    }
}

#[pymethods]
impl PyNobitsSection {
    #[pyo3(name = "post_process")]
    fn py_post_process(
        &mut self,
        context: &mut Context,
        user_relocs: &PyUserRelocs,
    ) -> Result<(), SectionPostProcessError> {
        self.inner.post_process(
            context,
            user_relocs.inner(),
            |section, context, _user_relocs| section.post_process(context),
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
        settings: &SymNobitsDisplaySettings,
    ) -> Result<Option<String>, SymDisplayError> {
        self.inner.display_sym(
            |section| section.nobits_symbols().get(index),
            |sym| sym.display(context, settings).map(|x| x.to_string()),
        )
    }

    #[pyo3(name = "label_count_for_sym")]
    pub fn py_label_count_for_sym(&self, _sym_index: usize) -> usize {
        0
    }
}
