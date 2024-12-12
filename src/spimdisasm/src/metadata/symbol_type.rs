/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

// use alloc::string::String;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum SymbolType {
    Function,
    BranchLabel,
    Jumptable,
    JumptableLabel,
    // HardwareReg,
    // Constant,
    GccExceptTable,
    GccExceptTableLabel,

    Byte,
    Short,
    Word,
    DWord,

    Float32,
    Float64,
    CString,

    UserCustom,
}

impl SymbolType {
    pub fn valid_branch_target(&self) -> bool {
        matches!(
            self,
            SymbolType::Function
                | SymbolType::BranchLabel
                | SymbolType::JumptableLabel
                | SymbolType::GccExceptTableLabel
        )
    }
}
