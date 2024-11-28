/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum SymbolType {
    Function,
    BranchLabel,
    Jumptable,
    JumptableLabel,
    // HardwareReg,
    // Constant,
    GccExceptTable,
    GccExceptTableLabel,

    // TODO: add stuff like string, pascal_string, s32, f32, s16, and so on?
    // Float32
    // Float64
    // CString
    // Byte? UInt8?
    // Short? UInt16?

    //
    UserDeclared(String),
}

impl SymbolType {
    pub fn valid_branch_target(&self) -> bool {
        match self {
            SymbolType::Function => true,
            SymbolType::BranchLabel => true,
            SymbolType::Jumptable => false,
            SymbolType::JumptableLabel => true,
            SymbolType::GccExceptTable => false,
            SymbolType::GccExceptTableLabel => true,
            SymbolType::UserDeclared(_) => false,
        }
    }
}
