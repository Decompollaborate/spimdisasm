/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::{RelocReferencedSym, RelocationInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum RelocationType {
    /// Official description: No reloc.
    ///
    ///
    R_MIPS_NONE = 0,
    /// Official description: Direct 16 bit.
    ///
    /// TODO: figure out what is this for. The MIPS ABI pdf doesn't explain it.
    R_MIPS_16 = 1,
    /// Official description: Direct 32 bit.
    ///
    /// Used for symbols references in `.data`-like sections.
    R_MIPS_32 = 2,
    /// Official description: PC relative 32 bit.
    ///
    /// Dynamic linker shenanigans.
    R_MIPS_REL32 = 3,
    /// Official description: Direct 26 bit shifted.
    ///
    /// Direct function calls (`jal`s, `j`s, etc).
    R_MIPS_26 = 4,
    /// Official description: High 16 bit.
    ///
    /// `%hi` reloc to be used on `lui`s.
    R_MIPS_HI16 = 5,
    /// Official description: Low 16 bit.
    ///
    /// The `%lo` pairing of either a `R_MIPS_HI16` or a `R_MIPS_GOT16` that is referencing a local symbol.
    R_MIPS_LO16 = 6,
    /// Official description: GP relative 16 bit.
    ///
    /// Reference "small symbols", symbols present on small data sections (`.sdata`, `.sbss`, `.scommon`, etc.).
    ///
    /// `$gp` relative.
    R_MIPS_GPREL16 = 7,
    /// Official description: 16 bit literal entry.
    ///
    /// TODO: figure out what is this for. The MIPS ABI pdf doesn't explain it.
    R_MIPS_LITERAL = 8,
    /// Official description: 16 bit GOT entry.
    ///
    /// Used for instructions referencing the "global offset table" (GOT).
    ///
    /// `$gp` relative.
    R_MIPS_GOT16 = 9,
    /// Official description: PC relative 16 bit.
    ///
    /// Branches.
    R_MIPS_PC16 = 10,
    /// Official description: 16 bit GOT entry for function.
    ///
    /// Used to load the address of a function from the GOT, which will be later called with `jalr`.
    ///
    /// `$gp` relative.
    R_MIPS_CALL16 = 11,
    /// Official description: GP relative 32 bit.
    ///
    /// Like `R_MIPS_GPREL32`, but GOT-relative.
    R_MIPS_GPREL32 = 12,

    /// Yet another way of loading `$gp` relative symbols.
    ///
    /// ```mips
    /// lui         $reg, %got_hi(sym)
    /// addu        $reg, $reg, $gp
    /// lw          $reg2, %got_lo(sym)($reg)
    /// ```
    R_MIPS_GOT_HI16 = 22,
    /// Yet another way of loading `$gp` relative symbols.
    ///
    /// ```mips
    /// lui         $reg, %got_hi(sym)
    /// addu        $reg, $reg, $gp
    /// lw          $reg2, %got_lo(sym)($reg)
    /// ```
    R_MIPS_GOT_LO16 = 23,

    /// Yet another way of loading `$gp` relative functions.
    ///
    /// ```mips
    /// lui         $reg, %got_hi(function)
    /// addu        $reg, $reg, $gp
    /// lw          $reg2, %got_lo(sym)($reg)
    /// jalr        $reg2
    ///  nop
    /// ```
    R_MIPS_CALL_HI16 = 30,
    /// Yet another way of loading `$gp` relative functions.
    ///
    /// ```mips
    /// lui         $reg, %got_hi(function)
    /// addu        $reg, $reg, $gp
    /// lw          $reg2, %got_lo(sym)($reg)
    /// jalr        $reg2
    ///  nop
    /// ```
    R_MIPS_CALL_LO16 = 31,

    /// A hack to allow emitting hi/lo paired constants.
    R_CUSTOM_CONSTANT_HI = -1,
    /// A hack to allow emitting hi/lo paired constants.
    R_CUSTOM_CONSTANT_LO = -2,
}

impl RelocationType {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match *self {
            RelocationType::R_MIPS_NONE => "R_MIPS_NONE",
            RelocationType::R_MIPS_16 => "R_MIPS_16",
            RelocationType::R_MIPS_32 => "R_MIPS_32",
            RelocationType::R_MIPS_REL32 => "R_MIPS_REL32",
            RelocationType::R_MIPS_26 => "R_MIPS_26",
            RelocationType::R_MIPS_HI16 => "R_MIPS_HI16",
            RelocationType::R_MIPS_LO16 => "R_MIPS_LO16",
            RelocationType::R_MIPS_GPREL16 => "R_MIPS_GPREL16",
            RelocationType::R_MIPS_LITERAL => "R_MIPS_LITERAL",
            RelocationType::R_MIPS_GOT16 => "R_MIPS_GOT16",
            RelocationType::R_MIPS_PC16 => "R_MIPS_PC16",
            RelocationType::R_MIPS_CALL16 => "R_MIPS_CALL16",
            RelocationType::R_MIPS_GPREL32 => "R_MIPS_GPREL32",
            RelocationType::R_MIPS_GOT_HI16 => "R_MIPS_GOT_HI16",
            RelocationType::R_MIPS_GOT_LO16 => "R_MIPS_GOT_LO16",
            RelocationType::R_MIPS_CALL_HI16 => "R_MIPS_CALL_HI16",
            RelocationType::R_MIPS_CALL_LO16 => "R_MIPS_CALL_LO16",
            RelocationType::R_CUSTOM_CONSTANT_HI => "R_CUSTOM_CONSTANT_HI",
            RelocationType::R_CUSTOM_CONSTANT_LO => "R_CUSTOM_CONSTANT_LO",
        }
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "R_MIPS_NONE" => Some(RelocationType::R_MIPS_NONE),
            "R_MIPS_16" => Some(RelocationType::R_MIPS_16),
            "R_MIPS_32" => Some(RelocationType::R_MIPS_32),
            "R_MIPS_REL32" => Some(RelocationType::R_MIPS_REL32),
            "R_MIPS_26" => Some(RelocationType::R_MIPS_26),
            "R_MIPS_HI16" => Some(RelocationType::R_MIPS_HI16),
            "R_MIPS_LO16" => Some(RelocationType::R_MIPS_LO16),
            "R_MIPS_GPREL16" => Some(RelocationType::R_MIPS_GPREL16),
            "R_MIPS_LITERAL" => Some(RelocationType::R_MIPS_LITERAL),
            "R_MIPS_GOT16" => Some(RelocationType::R_MIPS_GOT16),
            "R_MIPS_PC16" => Some(RelocationType::R_MIPS_PC16),
            "R_MIPS_CALL16" => Some(RelocationType::R_MIPS_CALL16),
            "R_MIPS_GPREL32" => Some(RelocationType::R_MIPS_GPREL32),
            "R_MIPS_GOT_HI16" => Some(RelocationType::R_MIPS_GOT_HI16),
            "R_MIPS_GOT_LO16" => Some(RelocationType::R_MIPS_GOT_LO16),
            "R_MIPS_CALL_HI16" => Some(RelocationType::R_MIPS_CALL_HI16),
            "R_MIPS_CALL_LO16" => Some(RelocationType::R_MIPS_CALL_LO16),
            "R_CUSTOM_CONSTANT_HI" => Some(RelocationType::R_CUSTOM_CONSTANT_HI),
            "R_CUSTOM_CONSTANT_LO" => Some(RelocationType::R_CUSTOM_CONSTANT_LO),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, RelocationType::R_MIPS_NONE)
    }

    #[must_use]
    pub(crate) fn uses_parenthesis(&self) -> bool {
        match *self {
            RelocationType::R_MIPS_NONE => false,
            RelocationType::R_MIPS_16 => false,
            RelocationType::R_MIPS_32 => false,
            RelocationType::R_MIPS_REL32 => false,
            RelocationType::R_MIPS_26 => false,
            RelocationType::R_MIPS_HI16 => true,
            RelocationType::R_MIPS_LO16 => true,
            RelocationType::R_MIPS_GPREL16 => true,
            RelocationType::R_MIPS_LITERAL => false,
            RelocationType::R_MIPS_GOT16 => true,
            RelocationType::R_MIPS_PC16 => false,
            RelocationType::R_MIPS_CALL16 => true,
            RelocationType::R_MIPS_GPREL32 => false,
            RelocationType::R_MIPS_GOT_HI16 => true,
            RelocationType::R_MIPS_GOT_LO16 => true,
            RelocationType::R_MIPS_CALL_HI16 => true,
            RelocationType::R_MIPS_CALL_LO16 => true,
            RelocationType::R_CUSTOM_CONSTANT_HI => true,
            RelocationType::R_CUSTOM_CONSTANT_LO => true,
        }
    }

    #[must_use]
    pub fn allow_addends_on_ref(&self) -> bool {
        // TODO: should R_MIPS_LITERAL be in this list?
        !matches!(
            self,
            RelocationType::R_MIPS_26
                | RelocationType::R_MIPS_PC16
                | RelocationType::R_MIPS_CALL16
                | RelocationType::R_MIPS_CALL_HI16
                | RelocationType::R_MIPS_CALL_LO16
        )
    }

    #[must_use]
    pub(crate) fn valid_for_function(&self) -> bool {
        match *self {
            RelocationType::R_MIPS_NONE => true,
            RelocationType::R_MIPS_16 => true, // TODO: check
            RelocationType::R_MIPS_32 => false,
            RelocationType::R_MIPS_REL32 => false, // TODO: check
            RelocationType::R_MIPS_26 => true,
            RelocationType::R_MIPS_HI16 => true,
            RelocationType::R_MIPS_LO16 => true,
            RelocationType::R_MIPS_GPREL16 => true,
            RelocationType::R_MIPS_LITERAL => true, // TODO: check
            RelocationType::R_MIPS_GOT16 => true,
            RelocationType::R_MIPS_PC16 => true,
            RelocationType::R_MIPS_CALL16 => true,
            RelocationType::R_MIPS_GPREL32 => false,
            RelocationType::R_MIPS_GOT_HI16 => true,
            RelocationType::R_MIPS_GOT_LO16 => true,
            RelocationType::R_MIPS_CALL_HI16 => true,
            RelocationType::R_MIPS_CALL_LO16 => true,
            RelocationType::R_CUSTOM_CONSTANT_HI => true,
            RelocationType::R_CUSTOM_CONSTANT_LO => true,
        }
    }

    #[must_use]
    pub(crate) fn valid_for_data_sym(&self) -> bool {
        match *self {
            RelocationType::R_MIPS_NONE => true,
            RelocationType::R_MIPS_16 => true, // TODO: check
            RelocationType::R_MIPS_32 => true,
            RelocationType::R_MIPS_REL32 => true, // TODO: check
            RelocationType::R_MIPS_26 => false,
            RelocationType::R_MIPS_HI16 => false,
            RelocationType::R_MIPS_LO16 => false,
            RelocationType::R_MIPS_GPREL16 => false,
            RelocationType::R_MIPS_LITERAL => true, // TODO: check
            RelocationType::R_MIPS_GOT16 => false,
            RelocationType::R_MIPS_PC16 => false,
            RelocationType::R_MIPS_CALL16 => false,
            RelocationType::R_MIPS_GPREL32 => true,
            RelocationType::R_MIPS_GOT_HI16 => false,
            RelocationType::R_MIPS_GOT_LO16 => false,
            RelocationType::R_MIPS_CALL_HI16 => false,
            RelocationType::R_MIPS_CALL_LO16 => false,
            RelocationType::R_CUSTOM_CONSTANT_HI => false,
            RelocationType::R_CUSTOM_CONSTANT_LO => false,
        }
    }

    #[must_use]
    pub fn new_reloc_info(self, referenced_sym: RelocReferencedSym) -> RelocationInfo {
        RelocationInfo::new(self, referenced_sym)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl RelocationType {
        #[pyo3(name = "from_name")]
        #[staticmethod]
        pub fn py_from_name(name: &str) -> Option<Self> {
            Self::from_name(name)
        }
    }
}
