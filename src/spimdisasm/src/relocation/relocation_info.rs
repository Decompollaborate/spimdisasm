/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::Vram,
    collections::addended_ordered_map::FindSettings,
    config::Compiler,
    context::Context,
    metadata::{LabelMetadata, LabelType, SymbolMetadata, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    symbols::display::InternalSymDisplSettings,
};

use super::{RelocReferencedSym, RelocationType};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct RelocationInfo {
    reloc_type: RelocationType,
    referenced_sym: RelocReferencedSym,
}

impl RelocationInfo {
    #[must_use]
    pub(crate) fn new(reloc_type: RelocationType, referenced_sym: RelocReferencedSym) -> Self {
        Self {
            reloc_type,
            referenced_sym,
        }
    }

    #[must_use]
    pub(crate) const fn reloc_type(&self) -> RelocationType {
        self.reloc_type
    }
    //#[must_use]
    //pub(crate) const fn referenced_sym(&self) -> &RelocReferencedSym {
    //    &self.referenced_sym
    //}

    #[must_use]
    pub(crate) fn display<'ctx, 'rel, 'prnt>(
        &'rel self,
        context: &'ctx Context,
        segment_info: &'prnt ParentSegmentInfo,
        allow_ref_with_addend: bool,
        compiler: Option<Compiler>,
        internal_settings: InternalSymDisplSettings,
    ) -> Option<RelocationInfoDisplay<'ctx, 'rel, 'prnt>> {
        RelocationInfoDisplay::new(
            context,
            self,
            segment_info,
            allow_ref_with_addend,
            compiler,
            internal_settings,
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd)]
enum RelocSymState<'name, 'meta> {
    LiteralSymName(&'name str, i64),
    Sym(Vram, &'meta SymbolMetadata),
    Label(Vram, &'meta LabelMetadata),

    // Kinda useful for debugging
    SegmentNotFound(Vram),
    LabelNotFound(Vram),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct RelocationInfoDisplay<'ctx, 'rel, 'prnt> {
    rel: &'rel RelocationInfo,
    segment_info: &'prnt ParentSegmentInfo,
    reloc_sym_state: RelocSymState<'rel, 'ctx>,
    compiler: Option<Compiler>,
    internal_settings: InternalSymDisplSettings,
}

impl<'ctx, 'rel, 'prnt> RelocationInfoDisplay<'ctx, 'rel, 'prnt> {
    pub(crate) fn new(
        context: &'ctx Context,
        rel: &'rel RelocationInfo,
        segment_info: &'prnt ParentSegmentInfo,
        allow_ref_with_addend: bool,
        compiler: Option<Compiler>,
        internal_settings: InternalSymDisplSettings,
    ) -> Option<Self> {
        let reloc_sym_state = match &rel.referenced_sym {
            RelocReferencedSym::SymName(name, addend) => {
                RelocSymState::LiteralSymName(name, *addend)
            }
            RelocReferencedSym::Address(vram) => {
                let find_settings = FindSettings::new(
                    allow_ref_with_addend && rel.reloc_type.allow_addends_on_ref(),
                );
                if let Some(sym_metadata) = context.find_symbol_from_any_segment(
                    *vram,
                    segment_info,
                    find_settings,
                    |metadata| {
                        match rel.reloc_type {
                            RelocationType::R_MIPS_NONE => true, // Shouldn't be possible, but whatever.

                            RelocationType::R_MIPS_16 => true,
                            RelocationType::R_MIPS_32 => {
                                if metadata.sym_type() == Some(SymbolType::Function) {
                                    // Avoid referencing addends of functions
                                    metadata.vram() == *vram
                                } else {
                                    true
                                }
                            }

                            RelocationType::R_MIPS_REL32 => true,

                            RelocationType::R_MIPS_26 => {
                                metadata.sym_type() == Some(SymbolType::Function)
                            }

                            RelocationType::R_MIPS_HI16 | RelocationType::R_MIPS_LO16 => {
                                if metadata.sym_type() == Some(SymbolType::Function) {
                                    // Avoid referencing addends of functions
                                    metadata.vram() == *vram
                                } else {
                                    true
                                }
                            }
                            RelocationType::R_MIPS_GPREL16 => true,

                            RelocationType::R_MIPS_LITERAL => true,

                            RelocationType::R_MIPS_PC16 => {
                                metadata.sym_type() == Some(SymbolType::Function)
                            }

                            RelocationType::R_MIPS_CALL16
                            | RelocationType::R_MIPS_CALL_HI16
                            | RelocationType::R_MIPS_CALL_LO16 => {
                                // TODO: check symbol is present in the GOT.
                                metadata.sym_type() == Some(SymbolType::Function)
                            }

                            RelocationType::R_MIPS_GOT16
                            | RelocationType::R_MIPS_GPREL32
                            | RelocationType::R_MIPS_GOT_HI16
                            | RelocationType::R_MIPS_GOT_LO16 => {
                                // TODO: check symbol is present in the GOT.
                                true
                            }

                            RelocationType::R_CUSTOM_CONSTANT_HI => false,
                            RelocationType::R_CUSTOM_CONSTANT_LO => false,
                        }
                    },
                ) {
                    RelocSymState::Sym(*vram, sym_metadata)
                } else {
                    if false {
                        return None;
                    }
                    RelocSymState::SegmentNotFound(*vram)
                }
            }
            RelocReferencedSym::Label(vram) => {
                if let Some(label_metadata) =
                    context.find_label_from_any_segment(*vram, segment_info, |metadata| {
                        match rel.reloc_type {
                            RelocationType::R_MIPS_NONE => true, // Shouldn't be possible, but whatever.

                            RelocationType::R_MIPS_16 => true,
                            RelocationType::R_MIPS_32 => metadata.label_type() != LabelType::Branch,

                            RelocationType::R_MIPS_REL32 => true,

                            RelocationType::R_MIPS_26 => {
                                metadata.label_type() == LabelType::AlternativeEntry
                            }

                            RelocationType::R_MIPS_HI16 => true,
                            RelocationType::R_MIPS_LO16 => true,
                            RelocationType::R_MIPS_GPREL16 => true,

                            RelocationType::R_MIPS_LITERAL => true,

                            RelocationType::R_MIPS_PC16 => true,

                            RelocationType::R_MIPS_CALL16
                            | RelocationType::R_MIPS_CALL_HI16
                            | RelocationType::R_MIPS_CALL_LO16 => {
                                // TODO: check symbol is present in the GOT.
                                false
                            }

                            RelocationType::R_MIPS_GOT16
                            | RelocationType::R_MIPS_GPREL32
                            | RelocationType::R_MIPS_GOT_HI16
                            | RelocationType::R_MIPS_GOT_LO16 => {
                                // TODO: check symbol is present in the GOT.
                                true
                            }

                            RelocationType::R_CUSTOM_CONSTANT_HI => false,
                            RelocationType::R_CUSTOM_CONSTANT_LO => false,
                        }
                    })
                {
                    RelocSymState::Label(*vram, label_metadata)
                } else {
                    if false {
                        return None;
                    }
                    RelocSymState::LabelNotFound(*vram)
                }
            }
        };

        Some(Self {
            rel,
            segment_info,
            reloc_sym_state,
            compiler,
            internal_settings,
        })
    }
}

impl RelocationInfoDisplay<'_, '_, '_> {
    fn display_addend(&self, f: &mut fmt::Formatter<'_>, addend: i64) -> fmt::Result {
        if addend == 0 {
            Ok(())
        } else {
            if self
                .compiler
                .is_some_and(|x| x.big_addend_workaround_for_migrated_functions())
                && self.internal_settings.migrate()
                && self.rel.reloc_type == RelocationType::R_MIPS_LO16
            {
                if addend < -0x8000 {
                    return write!(f, " - (0x{:X} & 0xFFFF)", -addend);
                }
                if addend > 0x7FFF {
                    return write!(f, " + (0x{:X} & 0xFFFF)", addend);
                }
            }

            if addend < 0 {
                write!(f, " - 0x{:X}", -addend)
            } else {
                write!(f, " + 0x{:X}", addend)
            }
        }
    }
}

impl fmt::Display for RelocationInfoDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: gpRelHack
        let x = match self.rel.reloc_type {
            RelocationType::R_MIPS_NONE => "",
            RelocationType::R_MIPS_16 => "%half",
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

        let addend = match &self.reloc_sym_state {
            RelocSymState::LiteralSymName(name, addend) => {
                write!(f, "{}", name)?;
                *addend
            }
            RelocSymState::Sym(vram, sym_metadata) => {
                write!(f, "{}", sym_metadata.display_name())?;
                (*vram - sym_metadata.vram()).inner().into()
            }
            RelocSymState::Label(_vram, label_metadata) => {
                write!(f, "{}", label_metadata.display_name())?;
                0
            }
            RelocSymState::SegmentNotFound(vram) => {
                write!(f, "/* ERROR: segment for address 0x{} not found */", vram)?;
                0
            }
            RelocSymState::LabelNotFound(vram) => {
                write!(f, "/* ERROR: label for address 0x{} not found */", vram)?;
                0
            }
        };

        self.display_addend(f, addend)?;

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
