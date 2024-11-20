/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use rabbitizer::{Instruction, Vram};

use crate::{context::Context, rom_address::RomAddress, sections::SectionBase};

use super::{Symbol, SymbolBase};

pub struct SymbolFunction {
    symbol_base: SymbolBase,
    instructions: Vec<Instruction>
}

impl SymbolFunction {
    pub(crate) fn new(context: &Context, instructions: Vec<Instruction>, rom: RomAddress, vram: Vram, in_section_offset: usize) -> Self {
        Self {
            symbol_base: SymbolBase::new(Some(rom), vram),
            instructions,
        }
    }
}

impl SymbolFunction {
    // TODO: maybe remove?
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

impl Symbol for SymbolFunction {
    fn symbol_base(&self) -> &SymbolBase {
        &self.symbol_base
    }
}
