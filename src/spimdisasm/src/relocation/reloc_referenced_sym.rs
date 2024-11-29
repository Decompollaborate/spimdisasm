/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;
use rabbitizer::Vram;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RelocReferencedSym {
    Address(Vram),
    SymName(String),
}
