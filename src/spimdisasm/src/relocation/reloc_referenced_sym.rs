/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use crate::addresses::Vram;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RelocReferencedSym {
    // TODO: rename to Symbol
    Address {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    Label(Vram),
    SymName(Arc<str>, i64),
}

impl RelocReferencedSym {
    pub const fn new_address(vram: Vram) -> Self {
        Self::Address {
            unaddended_vram: vram,
            addended_vram: vram,
        }
    }
}
