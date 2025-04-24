/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};
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

    pub fn iter(&self) -> GlobalOffsetTableIter {
        GlobalOffsetTableIter::new(self)
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GotGlobalEntry {
    initial: u32,
    sym_val: u32,
    undef_com_or_abs: bool,
    sym_name: Arc<str>,
}

impl GotGlobalEntry {
    #[must_use]
    pub fn new<T>(initial: u32, sym_val: u32, undef_com_or_abs: bool, sym_name: T) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self::new_impl(initial, sym_val, undef_com_or_abs, sym_name.into())
    }

    #[must_use]
    const fn new_impl(
        initial: u32,
        sym_val: u32,
        undef_com_or_abs: bool,
        sym_name: Arc<str>,
    ) -> Self {
        Self {
            initial,
            sym_val,
            undef_com_or_abs,
            sym_name,
        }
    }

    #[must_use]
    pub const fn initial(&self) -> u32 {
        self.initial
    }

    #[must_use]
    pub const fn undef_com_or_abs(&self) -> bool {
        self.undef_com_or_abs
    }

    #[must_use]
    pub fn sym_name(&self) -> Arc<str> {
        self.sym_name.clone()
    }

    #[must_use]
    pub const fn address(&self) -> u32 {
        if self.sym_val != 0 && !self.undef_com_or_abs {
            self.sym_val
        } else {
            self.initial
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GotRequestedAddress<'got> {
    // TODO: consider using references here instead of copying the data
    LazyResolver(&'got GotLocalEntry),
    Local(&'got GotLocalEntry),
    Global(&'got GotGlobalEntry),
}

impl GotRequestedAddress<'_> {
    #[must_use]
    pub(crate) const fn address(&self) -> u32 {
        match self {
            GotRequestedAddress::LazyResolver(x) => x.address(),
            GotRequestedAddress::Local(x) => x.address(),
            GotRequestedAddress::Global(x) => x.address(),
        }
    }
}

#[must_use]
pub struct GlobalOffsetTableIter<'got> {
    current_vram: Vram,
    got: &'got GlobalOffsetTable,
}

impl<'got> GlobalOffsetTableIter<'got> {
    const fn new(got: &'got GlobalOffsetTable) -> Self {
        Self {
            current_vram: got.vram().start(),
            got,
        }
    }
}

impl<'got> Iterator for GlobalOffsetTableIter<'got> {
    type Item = (Vram, GotRequestedAddress<'got>);

    fn next(&mut self) -> Option<Self::Item> {
        let current_vram = self.current_vram;

        if let Some(requested_address) = self.got.request_address(current_vram) {
            self.current_vram += Size::new(4);
            Some((current_vram, requested_address))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = (self.got.vram().end() - self.current_vram).inner() / 4;
        let left = left.max(0) as usize;

        (left, Some(left))
    }
}
