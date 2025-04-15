/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rabbitizer::access_type::AccessType;

use crate::config::Compiler;

use super::LabelType;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum SymbolType {
    Function,
    Jumptable,
    // HardwareReg,
    // Constant,
    GccExceptTable,
    // DispatchTable,
    Byte,
    Short,
    Word,
    DWord,

    Float32,
    Float64,
    CString,

    VirtualTable,

    UserCustom,
}

impl SymbolType {
    pub fn valid_call_target(&self) -> bool {
        matches!(self, SymbolType::Function)
    }

    pub fn is_table(&self) -> bool {
        matches!(self, SymbolType::Jumptable | SymbolType::GccExceptTable)
    }

    pub(crate) fn label_for_table(maybe_table: Option<Self>) -> Option<LabelType> {
        maybe_table.and_then(|x| match x {
            SymbolType::Jumptable => Some(LabelType::Jumptable),
            SymbolType::GccExceptTable => Some(LabelType::GccExceptTable),
            _ => None,
        })
    }

    pub fn can_reference_symbols(&self) -> bool {
        match self {
            SymbolType::Function => false,
            SymbolType::Jumptable | SymbolType::GccExceptTable => true,
            SymbolType::Byte | SymbolType::Short => false,
            SymbolType::Word => true,
            SymbolType::DWord => false,
            SymbolType::Float32 | SymbolType::Float64 => false,
            SymbolType::CString => false,
            SymbolType::VirtualTable => true,
            SymbolType::UserCustom => true,
        }
    }

    pub fn from_access_type(access_type: AccessType) -> Option<Self> {
        // TODO: use AccessType.min_size and AccessType.min_alignment

        match access_type {
            AccessType::BYTE => Some(SymbolType::Byte),
            AccessType::SHORT => Some(SymbolType::Short),
            AccessType::WORD | AccessType::WORD_COP2 | AccessType::LINKED_WORD_WORD => {
                Some(SymbolType::Word)
            }
            AccessType::DOUBLEWORD
            | AccessType::DOUBLEWORD_COP2
            | AccessType::LINKED_WORD_DOUBLEWORD => Some(SymbolType::DWord),
            AccessType::QUADWORD => Some(SymbolType::DWord), // ?
            AccessType::FLOAT => Some(SymbolType::Float32),
            AccessType::DOUBLEFLOAT => Some(SymbolType::Float64),

            // Struct copies
            AccessType::UNALIGNED_WORD_LEFT
            | AccessType::UNALIGNED_WORD_RIGHT
            | AccessType::UNALIGNED_DOUBLEWORD_LEFT
            | AccessType::UNALIGNED_DOUBLEWORD_RIGHT => None,
        }
    }

    pub(crate) fn may_have_addend(&self) -> bool {
        !matches!(
            self,
            SymbolType::Function
                | SymbolType::Jumptable
                | SymbolType::GccExceptTable
                | SymbolType::VirtualTable
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
