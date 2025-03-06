/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum LabelType {
    Branch,
    Jumptable,
    GccExceptTable,
    AlternativeEntry,
    // DispatchTable,
}

impl LabelType {
    pub const fn does_new_takes_precedence(self, new: Self) -> bool {
        match (new, self) {
            (LabelType::Branch, _) => false,
            (LabelType::Jumptable, LabelType::GccExceptTable | LabelType::AlternativeEntry) => {
                false
            }
            (LabelType::Jumptable, LabelType::Branch | LabelType::Jumptable) => true,
            (LabelType::GccExceptTable, _) => true,
            (LabelType::AlternativeEntry, _) => true,
        }
    }
}
