/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::BTreeSet;

use alloc::string::String;

use rabbitizer::vram::VramOffset;
use rabbitizer::Vram;

use crate::parent_segment_info::ParentSegmentInfo;
use crate::rom_address::RomAddress;
use crate::size::Size;
use crate::symbols::Symbol;

pub struct SectionBase {
    name: String,

    rom: Option<RomAddress>,
    vram: Vram,

    // in_section_offset: u32,
    // section_type: SectionType,

    //
    parent_segment_info: ParentSegmentInfo,
}

impl SectionBase {
    pub fn new(
        name: String,
        rom: Option<RomAddress>,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Self {
        Self {
            name,
            rom,
            vram,
            parent_segment_info,
        }
    }
}

impl SectionBase {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn vram_offset(&self, local_offset: usize) -> Vram {
        self.vram.add_offset(&VramOffset::new(local_offset as i32))
    }
    pub const fn rom_offset(&self, local_offset: usize) -> Option<RomAddress> {
        if let Some(rom) = self.rom {
            Some(rom.add_size(&Size::new(local_offset as u32)))
        } else {
            None
        }
    }
    pub fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }
}

pub trait Section {
    fn section_base(&self) -> &SectionBase;
    fn symbol_list(&self) -> &[impl Symbol];
    fn symbols_vrams(&self) -> &BTreeSet<Vram>;
}
