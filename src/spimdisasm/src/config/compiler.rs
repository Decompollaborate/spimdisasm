/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::metadata::SymbolType;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum Compiler {
    // N64
    IDO,
    KMC,
    SN64,

    // iQue
    EGCS,

    // PS1
    PSYQ,

    // PS2
    MWCCPS2,
    EEGCC,
}

impl Compiler {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "IDO" => Some(Compiler::IDO),
            "KMC" => Some(Compiler::KMC),
            "SN64" => Some(Compiler::SN64),
            "EGCS" => Some(Compiler::EGCS),
            "PSYQ" => Some(Compiler::PSYQ),
            "MWCCPS2" => Some(Compiler::MWCCPS2),
            "EEGCC" => Some(Compiler::EEGCC),
            _ => None,
        }
    }
}

// TODO: remove `#[allow(dead_code)]`
#[allow(dead_code)]
impl Compiler {
    pub const fn name(&self) -> &'static str {
        match self {
            Compiler::IDO => "IDO",
            Compiler::KMC => "KMC",
            Compiler::SN64 => "SN64",
            Compiler::EGCS => "EGCS",
            Compiler::PSYQ => "PSYQ",
            Compiler::MWCCPS2 => "MWCCPS2",
            Compiler::EEGCC => "EEGCC",
        }
    }

    pub(crate) const fn has_late_rodata(&self) -> bool {
        matches!(self, Compiler::IDO)
    }

    pub(crate) const fn pair_multiple_hi_to_same_low(&self) -> bool {
        !matches!(self, Compiler::IDO)
    }

    pub(crate) const fn allow_rdata_migration(&self) -> bool {
        matches!(self, Compiler::SN64 | Compiler::PSYQ)
    }

    // """
    // Modern GAS can handle big addends (outside the 16-bits range) for the `%lo`
    // directive just fine, but old assemblers choke on them, so we truncate them
    // to said range when building with those assemblers.
    //
    // Decomp projects usually use two assemblers:
    // - One to assemble unmigrated files, usually with modern GAS.
    // - Another one to assemble individual functions that get inserted into C
    //   files, either with asm directives from the compiler (using the built-in
    //   old assembler shipped with the old compiler) or with an external tools
    //   (like asm-proc for IDO).
    //
    // Modern GAS requires no addend truncation to produce matching output, so we
    // don't use the workaround for unmigrated asm files.
    //
    // For migrated functions we need to know if the compiler uses modern GAS or
    // old GAS. If it uses modern GAS (like IDO projects), then this flag should
    // be turned off, but if the project uses its own old assembler (like most GCC
    // based projects) then this flag needs to be turned on.
    // """
    pub(crate) const fn big_addend_workaround_for_migrated_functions(&self) -> bool {
        !matches!(self, Compiler::IDO)
    }

    // """
    // The value the compiler will use to align the `.text` section of the given
    // object.
    //
    // Used for determining `.text` file splits when disassembling full ROM images.
    //
    // The real aligment value will be computed like `1 << x`, where `x`
    // corresponds to the value given to this property.
    //
    // If a compiler emits multiple `.text` sections per object (i.e. each function
    // is emitted on its own section) then it is better to keep this value as
    // `None`, since the split detector won't give any meaningful result.
    // """
    pub(crate) const fn section_align_text(&self) -> Option<u8> {
        match self {
            Compiler::IDO => Some(4),
            Compiler::KMC => Some(4),
            Compiler::SN64 => Some(4),
            Compiler::EGCS => Some(4),
            _ => None,
        }
    }

    // """
    // The value the compiler will use to align the `.rodata` section of the given
    // object.
    //
    // Used for determining `.rodata` file splits when disassembling full ROM images.
    //
    // The real aligment value will be computed like `1 << x`, where `x`
    // corresponds to the value given to this property.
    // """
    pub(crate) const fn section_align_rodata(&self) -> Option<u8> {
        match self {
            Compiler::IDO => Some(4),
            Compiler::KMC => Some(4),
            Compiler::SN64 => Some(4),
            Compiler::EGCS => Some(4),
            _ => None,
        }
    }

    // """If True then emitting an align directive affects the assembler so it
    // aligns the section to the biggest symbol alignment.
    //
    // Some assemblers detect the biggest alignment of the symbols of the section
    // and apply said alignment to the section itself, while other assemblers
    // hardcode the section alignment even when there are symbols that have larger
    // alignment.
    //
    // We need this information to determine if we can emit alignment directives
    // than would make the section to not be aligned in the final ROM.
    // """
    pub(crate) const fn symbol_alignment_requires_aligned_section(&self) -> bool {
        matches!(self, Compiler::MWCCPS2 | Compiler::EEGCC)
    }

    const fn prev_align_function(&self) -> Option<u8> {
        match self {
            Compiler::EEGCC => Some(3),
            _ => None,
        }
    }
    const fn prev_align_jumptable(&self) -> Option<u8> {
        match self {
            Compiler::KMC => Some(3),
            Compiler::SN64 => Some(3),
            Compiler::EGCS => Some(3),
            Compiler::PSYQ => Some(3),
            Compiler::MWCCPS2 => Some(4),
            Compiler::EEGCC => Some(3),
            _ => None,
        }
    }
    // TODO: Specifying 3 as the default should be harmless. Need to investigate.
    const fn prev_align_float64(&self) -> Option<u8> {
        match self {
            Compiler::SN64 => Some(3),
            Compiler::PSYQ => Some(3),
            _ => None,
        }
    }
    const fn prev_align_c_string(&self) -> Option<u8> {
        match self {
            Compiler::EEGCC => Some(3),
            _ => Some(2),
        }
    }

    pub(crate) const fn prev_align_for_type(&self, sym_type: SymbolType) -> Option<u8> {
        match sym_type {
            SymbolType::Function => self.prev_align_function(),
            SymbolType::Jumptable => self.prev_align_jumptable(),
            SymbolType::GccExceptTable => None,
            SymbolType::BranchLabel
            | SymbolType::JumptableLabel
            | SymbolType::GccExceptTableLabel => None,
            SymbolType::Byte | SymbolType::Short | SymbolType::Word => None,
            SymbolType::DWord => None,
            SymbolType::Float32 => None,
            SymbolType::Float64 => self.prev_align_float64(),
            SymbolType::CString => self.prev_align_c_string(),
            SymbolType::UserCustom => None,
        }
    }

    const fn post_align_c_string(&self) -> Option<u8> {
        Some(2)
    }

    pub(crate) const fn post_align_for_type(&self, sym_type: SymbolType) -> Option<u8> {
        match sym_type {
            SymbolType::Function => None,
            SymbolType::Jumptable => None,
            SymbolType::GccExceptTable => None,
            SymbolType::BranchLabel
            | SymbolType::JumptableLabel
            | SymbolType::GccExceptTableLabel => None,
            SymbolType::Byte | SymbolType::Short | SymbolType::Word => None,
            SymbolType::DWord => None,
            SymbolType::Float32 => None,
            SymbolType::Float64 => None,
            SymbolType::CString => self.post_align_c_string(),
            SymbolType::UserCustom => None,
        }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl Compiler {
        #[staticmethod]
        #[pyo3(name = "from_name")]
        pub fn py_from_name(name: &str) -> Option<Self> {
            Self::from_name(name)
        }
    }
}
