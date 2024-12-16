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

    pub fn can_reference_symbols(&self) -> bool {
        match self {
            SymbolType::Function
            | SymbolType::BranchLabel
            | SymbolType::JumptableLabel
            | SymbolType::GccExceptTableLabel => false,
            SymbolType::Jumptable | SymbolType::GccExceptTable => true,
            SymbolType::Byte | SymbolType::Short => false,
            SymbolType::Word => true,
            SymbolType::DWord => false,
            SymbolType::Float32 | SymbolType::Float64 => false,
            SymbolType::CString => false,
            SymbolType::UserCustom => true,
        }
    }
}
