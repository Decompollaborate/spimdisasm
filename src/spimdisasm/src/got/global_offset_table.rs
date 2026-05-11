/* SPDX-FileCopyrightText: © 2025-2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;

use address_space::{AddressRange, Size, Vram};

use super::{GlobalOffsetTableIter, GotGlobalEntry, GotLocalEntry, GotRequestedAddress};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalOffsetTable {
    /// The address of the `.got` section in the elf
    vram: AddressRange<Vram>,
    locals: Vec<GotLocalEntry>,
    globals: Vec<GotGlobalEntry>,
}

impl GlobalOffsetTable {
    #[must_use]
    pub fn new(vram: Vram, locals: Vec<GotLocalEntry>, globals: Vec<GotGlobalEntry>) -> Self {
        let count = locals.len() + globals.len();
        let vram = AddressRange::new_size(vram, Size::new(count as u32 * 4))
            .expect("GOT table is too large?");

        Self {
            vram,
            locals,
            globals,
        }
    }

    #[must_use]
    pub const fn vram(&self) -> AddressRange<Vram> {
        self.vram
    }
    #[must_use]
    pub const fn locals(&self) -> &Vec<GotLocalEntry> {
        &self.locals
    }
    #[must_use]
    pub const fn globals(&self) -> &Vec<GotGlobalEntry> {
        &self.globals
    }

    #[must_use]
    pub(crate) fn request_address(&self, vram: Vram) -> Option<GotRequestedAddress<'_>> {
        if !self.vram.in_range(vram) {
            return None;
        }

        let diff = (vram - self.vram.start()).inner();
        if diff < 0 {
            return None;
        }

        let index = (diff / 4) as usize;
        if let Some(x) = self.locals.get(index) {
            if index == 0 {
                Some(GotRequestedAddress::LazyResolver(x))
            } else {
                Some(GotRequestedAddress::Local(x))
            }
        } else {
            let global_index = index - self.locals.len();
            self.globals
                .get(global_index)
                .map(GotRequestedAddress::Global)
        }
    }

    pub fn iter(&self) -> GlobalOffsetTableIter<'_> {
        GlobalOffsetTableIter::new(self)
    }
}
