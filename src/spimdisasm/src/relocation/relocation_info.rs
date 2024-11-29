/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use rabbitizer::Vram;

use crate::{
    context::Context,
    metadata::{segment_metadata::FindSettings, SymbolMetadata},
    parent_segment_info::ParentSegmentInfo,
};

use super::{RelocReferencedSym, RelocationType};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelocationInfo {
    reloc_type: RelocationType,
    referenced_sym: RelocReferencedSym,
    addend: i32,
}

impl RelocationInfo {
    #[must_use]
    pub fn new(
        reloc_type: RelocationType,
        referenced_sym: RelocReferencedSym,
        addend: i32,
    ) -> Self {
        Self {
            reloc_type,
            referenced_sym,
            addend,
        }
    }

    #[must_use]
    pub const fn reloc_type(&self) -> RelocationType {
        self.reloc_type
    }
    #[must_use]
    pub const fn referenced_sym(&self) -> &RelocReferencedSym {
        &self.referenced_sym
    }
    #[must_use]
    pub const fn addend(&self) -> i32 {
        self.addend
    }

    pub fn display<'ctx, 'rel, 'prnt>(
        &'rel self,
        context: &'ctx Context,
        segment_info: &'prnt ParentSegmentInfo,
    ) -> Option<RelocationInfoDisplay<'ctx, 'rel, 'prnt>> {
        RelocationInfoDisplay::new(context, self, segment_info)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd)]
enum RelocSymState<'name, 'meta> {
    LiteralSymName(&'name str),
    Sym(&'meta SymbolMetadata),
    // Kinda useful for debugging
    SymbolNotFound(Vram),
    // Kinda useful for debugging
    SegmentNotFound(Vram),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct RelocationInfoDisplay<'ctx, 'rel, 'prnt> {
    rel: &'rel RelocationInfo,
    segment_info: &'prnt ParentSegmentInfo,
    reloc_sym_state: RelocSymState<'rel, 'ctx>,
}

impl<'ctx, 'rel, 'prnt> RelocationInfoDisplay<'ctx, 'rel, 'prnt> {
    pub fn new(
        context: &'ctx Context,
        rel: &'rel RelocationInfo,
        segment_info: &'prnt ParentSegmentInfo,
    ) -> Option<Self> {
        let reloc_sym_state = match &rel.referenced_sym {
            RelocReferencedSym::SymName(name) => RelocSymState::LiteralSymName(name),
            RelocReferencedSym::Address(vram) => {
                if let Some(referenced_segment) =
                    context.find_referenced_segment(*vram, segment_info)
                {
                    if let Some(sym_metadata) = referenced_segment
                        .find_symbol(*vram, FindSettings::new().with_allow_addend(false))
                    {
                        RelocSymState::Sym(sym_metadata)
                    } else {
                        // TODO: make this a setting
                        if false {
                            return None;
                        }
                        RelocSymState::SymbolNotFound(*vram)
                    }
                } else {
                    if false {
                        return None;
                    }
                    RelocSymState::SegmentNotFound(*vram)
                }
            }
        };

        Some(Self {
            rel,
            segment_info,
            reloc_sym_state,
        })
    }
}

impl fmt::Display for RelocationInfoDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: gpRelHack
        let x = match self.rel.reloc_type {
            RelocationType::R_MIPS_NONE => "",
            RelocationType::R_MIPS_16 => "",
            RelocationType::R_MIPS_32 => ".word ",
            RelocationType::R_MIPS_REL32 => "",
            RelocationType::R_MIPS_26 => "",
            RelocationType::R_MIPS_HI16 => "%hi",
            RelocationType::R_MIPS_LO16 => "%lo",
            RelocationType::R_MIPS_GPREL16 => "%gp_rel",
            RelocationType::R_MIPS_LITERAL => "",
            RelocationType::R_MIPS_GOT16 => "%got",
            RelocationType::R_MIPS_PC16 => "",
            RelocationType::R_MIPS_CALL16 => "%call16",
            RelocationType::R_MIPS_GPREL32 => ".gpword ",
            RelocationType::R_MIPS_GOT_HI16 => "%got_hi",
            RelocationType::R_MIPS_GOT_LO16 => "%got_lo",
            RelocationType::R_MIPS_CALL_HI16 => "%call_hi",
            RelocationType::R_MIPS_CALL_LO16 => "%call_lo",
            RelocationType::R_CUSTOM_CONSTANT_HI => "",
            RelocationType::R_CUSTOM_CONSTANT_LO => "",
        };
        write!(f, "{}", x)?;
        if self.rel.reloc_type.uses_parenthesis() {
            write!(f, "(")?;
        }

        match &self.reloc_sym_state {
            RelocSymState::LiteralSymName(name) => write!(f, "{}", name)?,
            RelocSymState::Sym(sym_metadata) => write!(f, "{}", sym_metadata.display_name())?,
            RelocSymState::SymbolNotFound(vram) => {
                write!(f, "/* ERROR: symbol for address 0x{} not found */", vram)?
            }
            RelocSymState::SegmentNotFound(vram) => {
                write!(f, "/* ERROR: segment for address 0x{} not found */", vram)?
            }
        }

        let addend = self.rel.addend;
        if addend != 0 {
            /*
            if GlobalConfig.COMPILER.value.bigAddendWorkaroundForMigratedFunctions and isSplittedSymbol:
                if self.relocType == RelocType.MIPS_LO16:
                    if self.addend < -0x8000:
                        return f"{name} - (0x{-self.addend:X} & 0xFFFF)"
                    if self.addend > 0x7FFF:
                        return f"{name} + (0x{self.addend:X} & 0xFFFF)"
            */

            if addend < 0 {
                write!(f, " - 0x{:X}", -addend)?;
            } else {
                write!(f, " + 0x{:X}", addend)?;
            }
        }

        let x = match self.rel.reloc_type {
            RelocationType::R_MIPS_NONE => "",
            RelocationType::R_MIPS_16 => "",
            RelocationType::R_MIPS_32 => "",
            RelocationType::R_MIPS_REL32 => "",
            RelocationType::R_MIPS_26 => "",
            RelocationType::R_MIPS_HI16 => "",
            RelocationType::R_MIPS_LO16 => "",
            RelocationType::R_MIPS_GPREL16 => "",
            RelocationType::R_MIPS_LITERAL => "",
            RelocationType::R_MIPS_GOT16 => "",
            RelocationType::R_MIPS_PC16 => "",
            RelocationType::R_MIPS_CALL16 => "",
            RelocationType::R_MIPS_GPREL32 => "",
            RelocationType::R_MIPS_GOT_HI16 => "",
            RelocationType::R_MIPS_GOT_LO16 => "",
            RelocationType::R_MIPS_CALL_HI16 => "",
            RelocationType::R_MIPS_CALL_LO16 => "",
            RelocationType::R_CUSTOM_CONSTANT_HI => " >> 16",
            RelocationType::R_CUSTOM_CONSTANT_LO => " & 0xFFFF",
        };
        write!(f, "{}", x)?;
        if self.rel.reloc_type.uses_parenthesis() {
            write!(f, ")")?;
        }

        Ok(())
    }
}
