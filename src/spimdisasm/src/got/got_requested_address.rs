/* SPDX-FileCopyrightText: © 2025-2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use super::{GotGlobalEntry, GotLocalEntry};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GotRequestedAddress<'got> {
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
