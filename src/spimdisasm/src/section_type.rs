/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SectionType {
    Text = 1,
    Data,
    Rodata,
    Bss,
    // Reloc,
    GccExceptTable,
}