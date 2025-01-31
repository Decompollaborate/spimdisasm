/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rabbitizer::access_type::AccessType;

use crate::config::Compiler;

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

    pub fn is_label(&self) -> bool {
        matches!(
            self,
            SymbolType::BranchLabel | SymbolType::JumptableLabel | SymbolType::GccExceptTableLabel
        )
    }

    pub fn is_table(&self) -> bool {
        matches!(self, SymbolType::Jumptable | SymbolType::GccExceptTable)
    }

    pub(crate) fn label_for_table(maybe_table: Option<Self>) -> Option<Self> {
        maybe_table.and_then(|x| match x {
            SymbolType::Jumptable => Some(SymbolType::JumptableLabel),
            SymbolType::GccExceptTable => Some(SymbolType::GccExceptTableLabel),
            _ => None,
        })
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

            _ => unimplemented!("A new AccessType was added to rabbitizer?"),
        }
    }

    pub(crate) fn may_have_addend(&self) -> bool {
        !matches!(
            self,
            SymbolType::Function
                | SymbolType::BranchLabel
                | SymbolType::Jumptable
                | SymbolType::JumptableLabel
                | SymbolType::GccExceptTable
                | SymbolType::GccExceptTableLabel
        )
    }

    pub(crate) fn is_late_rodata(&self, compiler: Option<Compiler>) -> bool {
        if compiler.is_some_and(|x| x.has_late_rodata()) {
            // late rodata only exists in IDO's world

            matches!(
                self,
                SymbolType::Jumptable | SymbolType::Float32 | SymbolType::Float64
            )
        } else {
            false
        }
    }
}
