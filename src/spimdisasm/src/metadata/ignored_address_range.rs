/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{Size, Vram},
    collections::addended_ordered_map::SizedValue,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct IgnoredAddressRange {
    vram: Vram,
    size: Size,
}

impl IgnoredAddressRange {
    pub(crate) const fn new(vram: Vram, size: Size) -> Self {
        Self { vram, size }
    }

    pub const fn vram(&self) -> Vram {
        self.vram
    }
    pub const fn size(&self) -> Size {
        self.size
    }
}

impl SizedValue for IgnoredAddressRange {
    fn size(&self) -> Size {
        self.size
    }
}
