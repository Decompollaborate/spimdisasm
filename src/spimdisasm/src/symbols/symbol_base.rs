/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

use crate::rom_address::RomAddress;

pub struct SymbolBase {
    rom: Option<RomAddress>,
    vram: Vram,
}

impl SymbolBase {
    pub(crate) fn new(rom: Option<RomAddress>, vram: Vram) -> Self {
        Self { rom, vram }
    }

    pub fn rom(&self) -> Option<RomAddress> {
        self.rom
    }
    pub fn vram(&self) -> Vram {
        self.vram
    }
}

pub trait Symbol {
    /*
    fn bytes(&self) -> &[u8];

    fn size(&self) -> usize {
        self.bytes().len()
    }

    fn sizew(&self) -> usize {
        self.size() / 4
    }
    */

    fn symbol_base(&self) -> &SymbolBase;

    fn rom(&self) -> Option<RomAddress> {
        self.symbol_base().rom()
    }
    fn vram(&self) -> Vram {
        self.symbol_base().vram()
    }
}
