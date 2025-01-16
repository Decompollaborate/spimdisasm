/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct MacroLabels {
    func: String,
    alt_func: String,
    func_end: Option<String>,

    jtbl_label: String,
    ehtbl_label: String,

    data: String,
    data_end: Option<String>,
}

impl MacroLabels {
    pub fn new() -> Self {
        Self {
            func: "glabel".into(),
            alt_func: "alabel".into(),
            func_end: None,

            jtbl_label: "jlabel".into(),
            ehtbl_label: "ehlabel".into(),

            data: "dlabel".into(),
            data_end: None,
        }
    }

    pub fn func(&self) -> &str {
        &self.func
    }
    pub fn alt_func(&self) -> &str {
        &self.alt_func
    }
    pub fn func_end(&self) -> Option<&str> {
        self.func_end.as_deref()
    }

    pub fn jtbl_label(&self) -> &str {
        &self.jtbl_label
    }
    pub fn ehtbl_label(&self) -> &str {
        &self.ehtbl_label
    }

    pub fn data(&self) -> &str {
        &self.data
    }
    pub fn data_end(&self) -> Option<&str> {
        self.data_end.as_deref()
    }
}

impl Default for MacroLabels {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl MacroLabels {
        #[new]
        pub fn py_new() -> Self {
            Self::new()
        }

        #[pyo3(name="set_func", signature=(func))]
        pub fn py_set_func(&mut self, func: String) {
            self.func = func;
        }
        #[pyo3(name="set_alt_func", signature=(alt_func))]
        pub fn py_set_alt_func(&mut self, alt_func: String) {
            self.alt_func = alt_func;
        }
        #[pyo3(name="set_func_end", signature=(func_end))]
        pub fn py_set_func_end(&mut self, func_end: Option<String>) {
            self.func_end = func_end;
        }
        #[pyo3(name="set_jtbl_label", signature=(jtbl_label))]
        pub fn py_set_jtbl_label(&mut self, jtbl_label: String) {
            self.jtbl_label = jtbl_label;
        }
        #[pyo3(name="set_ehtbl_label", signature=(ehtbl_label))]
        pub fn py_set_ehtbl_label(&mut self, ehtbl_label: String) {
            self.ehtbl_label = ehtbl_label;
        }
        #[pyo3(name="set_data", signature=(data))]
        pub fn py_set_data(&mut self, data: String) {
            self.data = data;
        }
        #[pyo3(name="set_data_end", signature=(data_end))]
        pub fn py_set_data_end(&mut self, data_end: Option<String>) {
            self.data_end = data_end;
        }
    }
}
