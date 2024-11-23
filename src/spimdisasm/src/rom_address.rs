/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
