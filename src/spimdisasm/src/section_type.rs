/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SectionType {
    Text = 1,
    Data,
    Rodata,
    Bss,
    // Reloc,
    GccExceptTable,
}
