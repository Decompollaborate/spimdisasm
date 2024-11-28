/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

use crate::{
    address_range::AddressRange, parent_segment_info::ParentSegmentInfo, rom_address::RomAddress,
    size::Size,
};

pub trait Symbol {
    #[must_use]
    fn vram_range(&self) -> AddressRange<Vram>;

    #[must_use]
    fn size(&self) -> Size {
        self.vram_range().size()
    }

    #[must_use]
    fn parent_segment_info(&self) -> &ParentSegmentInfo;
}

pub trait RomSymbol: Symbol {
    #[must_use]
    fn rom_range(&self) -> AddressRange<RomAddress>;

    #[must_use]
    fn vram_from_rom(&self, rom: RomAddress) -> Option<Vram> {
        self.rom_range().in_range(rom).then(|| {
            let diff = rom - self.rom_range().start();
            self.vram_range().start() + diff
        })
    }

    #[must_use]
    fn rom_from_vram(&self, vram: Vram) -> Option<RomAddress> {
        self.vram_range().in_range(vram).then(|| {
            let diff = vram - self.vram_range().start();
            self.rom_range().start() + diff.try_into().expect("Shouldn't panic")
        })
    }
}
