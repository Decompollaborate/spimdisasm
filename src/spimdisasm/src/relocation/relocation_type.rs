/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use super::{RelocReferencedSym, RelocationInfo};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[allow(non_camel_case_types)] // TODO: remove?
pub enum RelocationType {
    R_MIPS_NONE = 0,     // No reloc
    R_MIPS_16 = 1,       // Direct 16 bit
    R_MIPS_32 = 2,       // Direct 32 bit
    R_MIPS_REL32 = 3,    // PC relative 32 bit
    R_MIPS_26 = 4,       // Direct 26 bit shifted
    R_MIPS_HI16 = 5,     // High 16 bit
    R_MIPS_LO16 = 6,     // Low 16 bit
    R_MIPS_GPREL16 = 7,  // GP relative 16 bit
    R_MIPS_LITERAL = 8,  // 16 bit literal entry
    R_MIPS_GOT16 = 9,    // 16 bit GOT entry
    R_MIPS_PC16 = 10,    // PC relative 16 bit
    R_MIPS_CALL16 = 11,  // 16 bit GOT entry for function
    R_MIPS_GPREL32 = 12, // GP relative 32 bit

    R_MIPS_GOT_HI16 = 22,
    R_MIPS_GOT_LO16 = 23,
    R_MIPS_CALL_HI16 = 30,
    R_MIPS_CALL_LO16 = 31,

    R_CUSTOM_CONSTANT_HI = -1,
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
    pub fn is_none(&self) -> bool {
        *self == RelocationType::R_MIPS_NONE
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
    pub fn new_reloc_info(self, referenced_sym: RelocReferencedSym) -> RelocationInfo {
        RelocationInfo::new(self, referenced_sym)
    }
}
