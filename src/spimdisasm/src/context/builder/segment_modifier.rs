/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use rabbitizer::Vram;

use crate::{
    metadata::{GeneratedBy, SegmentMetadata, SymbolMetadata},
    rom_address::RomAddress,
};

pub struct SegmentModifier<'seg> {
    segment: &'seg mut SegmentMetadata,
}

impl<'seg> SegmentModifier<'seg> {
    pub(crate) const fn new(segment: &'seg mut SegmentMetadata) -> Self {
        Self { segment }
    }
}

impl SegmentModifier<'_> {
    pub fn add_symbol(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_symbol(vram, rom, GeneratedBy::UserDeclared, None, false);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_function(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_function(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_branch_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_branch_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_jumptable(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_jumptable(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_jumptable_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_jumptable_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_gcc_except_table(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_gcc_except_table(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }

    pub fn add_gcc_except_table_label(
        &mut self,
        name: String,
        vram: Vram,
        rom: Option<RomAddress>,
    ) -> &mut SymbolMetadata {
        let sym = self
            .segment
            .add_gcc_except_table_label(vram, rom, GeneratedBy::UserDeclared);
        *sym.user_declared_name_mut() = Some(name);
        sym
    }
}
