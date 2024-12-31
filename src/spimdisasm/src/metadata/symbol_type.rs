/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

// use alloc::string::String;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rabbitizer::access_type::AccessType;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn from_access_type(access_type: AccessType) -> Option<Self> {
        // TODO: use AccessType.min_size and AccessType.min_alignment

        match access_type {
            AccessType::NONE => None,

            AccessType::BYTE => Some(SymbolType::Byte),
            AccessType::SHORT => Some(SymbolType::Short),
            AccessType::WORD => Some(SymbolType::Word),
            AccessType::DOUBLEWORD => Some(SymbolType::DWord),
            AccessType::QUADWORD => Some(SymbolType::DWord), // ?
            AccessType::FLOAT => Some(SymbolType::Float32),
            AccessType::DOUBLEFLOAT => Some(SymbolType::Float64),

            // Struct copies
            AccessType::WORD_LEFT
            | AccessType::WORD_RIGHT
            | AccessType::DOUBLEWORD_LEFT
            | AccessType::DOUBLEWORD_RIGHT => None,

            _ => todo!(),
        }
    }

    pub(crate) fn may_have_addend(&self) -> bool {
        !matches!(
            self,
            SymbolType::Function
                | SymbolType::BranchLabel
                | SymbolType::JumptableLabel
                | SymbolType::GccExceptTableLabel
        )
    }
}
