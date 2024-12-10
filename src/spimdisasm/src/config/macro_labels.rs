/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
        self.func_end.as_ref().map(|x| x.as_str())
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
        self.data_end.as_ref().map(|x| x.as_str())
    }
}

impl Default for MacroLabels {
    fn default() -> Self {
        Self::new()
    }
}
