/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use object::{
    elf,
    read::elf::{ElfFile32, Sym},
};
use spimdisasm::addresses::Vram;

use crate::dynamic_section::DynamicSection;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlobalOffsetTable {
    locals: Vec<GotLocalEntry>,
    globals: Vec<GotGlobalEntry>,
}

impl GlobalOffsetTable {
    pub fn new(locals: Vec<GotLocalEntry>, globals: Vec<GotGlobalEntry>) -> Self {
        Self { locals, globals }
    }

    pub fn locals(&self) -> &Vec<GotLocalEntry> {
        &self.locals
    }
    pub fn globals(&self) -> &Vec<GotGlobalEntry> {
        &self.globals
    }
}

impl GlobalOffsetTable {
    pub fn parse(elf_file: &ElfFile32, dynamic: &DynamicSection, raw_got: &[u32]) -> Option<Self> {
        let (local_gotno, gotsym) =
            if let (Some(local_gotno), Some(gotsym)) = (dynamic.local_gotno(), dynamic.gotsym()) {
                (local_gotno as usize, gotsym as usize)
            } else {
                return None;
            };

        let mut raw_got = raw_got.iter();

        // Consume the first `local_gotno` elements of `raw_got`
        let locals = raw_got
            .by_ref()
            .take(local_gotno)
            .map(|v| GotLocalEntry::new(*v))
            .collect();

        let elf_endian = elf_file.endian();
        let dynsym = elf_file.elf_dynamic_symbol_table();
        // `zip` the dynamic symtab starting at gotsym with the remaining `raw_got`
        let globals = dynsym
            .iter()
            .skip(gotsym)
            .zip(raw_got)
            .map(|(entry, initial)| {
                let sym_val = Vram::new(entry.st_value(elf_endian));
                let undef_or_com =
                    matches!(entry.st_shndx(elf_endian), elf::SHN_UNDEF | elf::SHN_COMMON);

                GotGlobalEntry::new(*initial, sym_val, undef_or_com)
            })
            .collect();

        Some(Self::new(locals, globals))
    }
}

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
    #[expect(dead_code)]
    pub const fn inner(&self) -> u32 {
        self.inner
    }
}

impl fmt::Debug for GotLocalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GotLocalEntry {{ 0x{:08X} }}", self.inner)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GotGlobalEntry {
    initial: u32,
    sym_val: Vram,
    undef_or_com: bool,
}

impl GotGlobalEntry {
    #[must_use]
    pub const fn new(initial: u32, sym_val: Vram, undef_or_com: bool) -> Self {
        Self {
            initial,
            sym_val,
            undef_or_com,
        }
    }

    /*
    #[must_use]
    pub const fn inner(&self) -> u32 {
        self.inner
    }
    */
}
