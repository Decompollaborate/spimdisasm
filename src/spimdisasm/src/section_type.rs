/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum SectionType {
    Text = 1,
    Data,
    Rodata,
    Bss,
    // Reloc,
    GccExceptTable,
}

impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionType::Text => write!(f, "Text"),
            SectionType::Data => write!(f, "Data"),
            SectionType::Rodata => write!(f, "Rodata"),
            SectionType::Bss => write!(f, "Bss"),
            SectionType::GccExceptTable => write!(f, "GccExceptTable"),
        }
    }
}
