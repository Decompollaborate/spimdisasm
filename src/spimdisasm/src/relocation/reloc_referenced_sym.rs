/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use crate::addresses::Vram;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RelocReferencedSym {
    Address(Vram),
    SymName(Arc<str>, i32),
}
