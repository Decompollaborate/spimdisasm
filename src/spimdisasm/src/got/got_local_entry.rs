/* SPDX-FileCopyrightText: © 2025-2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

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
