/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, ops};

use crate::size::Size;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RomAddress {
    inner: u32,
}

impl RomAddress {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }

    pub const fn inner(&self) -> u32 {
        self.inner
    }
}

impl RomAddress {
    pub const fn add_size(&self, size: &Size) -> Self {
        size.add_rom(self)
    }
}

impl ops::Sub<RomAddress> for RomAddress {
    type Output = Size;

    fn sub(self, rhs: RomAddress) -> Self::Output {
        Self::Output::new(self.inner - rhs.inner)
    }
}

impl fmt::Debug for RomAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RomAddress {{ 0x{:08X} }}", self.inner)
    }
}
