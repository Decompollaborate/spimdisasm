/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use core::fmt;
use rabbitizer::Vram;

use super::{AddressRange, Size};

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
        let vram = AddressRange::new(vram, vram + Size::new(count as u32 * 4));

        Self {
            vram,
            locals,
            globals,
        }
    }

    #[must_use]
    pub fn vram(&self) -> AddressRange<Vram> {
        self.vram
    }
    #[must_use]
    pub fn locals(&self) -> &Vec<GotLocalEntry> {
        &self.locals
    }
    #[must_use]
    pub fn globals(&self) -> &Vec<GotGlobalEntry> {
        &self.globals
    }

    #[must_use]
    pub(crate) fn request_address(&self, vram: Vram) -> Option<GotRequestedAddress> {
        if !self.vram.in_range(vram) {
            return None;
        }

        let diff = (vram - self.vram.start()).inner();
        if diff < 0 {
            return None;
        }

        let index = (diff / 4) as usize;
        if let Some(x) = self.locals.get(index) {
            Some(GotRequestedAddress::Local(*x))
        } else {
            let global_index = index - self.locals.len();
            self.globals
                .get(global_index)
                .map(|x| GotRequestedAddress::Global(*x))
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GotLocalEntry {
    inner: u32,
}

impl GotLocalEntry {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }

    #[must_use]
    pub const fn inner(&self) -> u32 {
        self.inner
    }

    #[must_use]
    pub(crate) const fn address(&self) -> u32 {
        self.inner()
    }
}

impl fmt::Debug for GotLocalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GotLocalEntry {{ 0x{:08X} }}", self.inner)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GotGlobalEntry {
    initial: u32,
    sym_val: u32,
    undef_or_com: bool,
}

impl GotGlobalEntry {
    #[must_use]
    pub const fn new(initial: u32, sym_val: u32, undef_or_com: bool) -> Self {
        Self {
            initial,
            sym_val,
            undef_or_com,
        }
    }

    #[must_use]
    pub const fn initial(&self) -> u32 {
        self.initial
    }

    #[must_use]
    pub const fn undef_or_com(&self) -> bool {
        self.undef_or_com
    }

    #[must_use]
    pub(crate) const fn address(&self) -> u32 {
        if self.sym_val != 0 && !self.undef_or_com {
            self.sym_val
        } else {
            self.initial
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GotRequestedAddress {
    // TODO: consider using references here instead of copying the data
    Local(GotLocalEntry),
    Global(GotGlobalEntry),
}

impl GotRequestedAddress {
    #[must_use]
    pub(crate) const fn address(&self) -> u32 {
        match self {
            GotRequestedAddress::Local(x) => x.address(),
            GotRequestedAddress::Global(x) => x.address(),
        }
    }

    #[must_use]
    pub(crate) const fn is_local(&self) -> bool {
        matches!(self, GotRequestedAddress::Local(_))
    }
    #[must_use]
    #[expect(dead_code)] // TODO
    pub(crate) const fn is_global(&self) -> bool {
        matches!(self, GotRequestedAddress::Global(_))
    }
}
