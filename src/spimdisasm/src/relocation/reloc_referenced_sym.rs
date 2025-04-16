/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use crate::addresses::Vram;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RelocReferencedSym {
    Address {
        unaddended_address: Vram,
        addended_address: Vram,
    },
    Label(Vram),
    SymName(Arc<str>, i64),
}

impl RelocReferencedSym {
    pub const fn new_address(address: Vram) -> Self {
        Self::Address {
            unaddended_address: address,
            addended_address: address,
        }
    }
}
