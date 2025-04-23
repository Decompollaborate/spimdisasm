/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use object::{
    elf,
    read::elf::{ElfFile32, Sym},
};
use spimdisasm::addresses::{GlobalOffsetTable, GotGlobalEntry, GotLocalEntry, Vram};

use crate::{dynamic_section::DynamicSection, utils};

pub fn parse_got(
    elf_file: &ElfFile32,
    dynamic: &DynamicSection,
    got_vram: Vram,
    raw_got: &[u32],
) -> GlobalOffsetTable {
    let local_gotno = dynamic.local_gotno() as usize;
    let gotsym = dynamic.gotsym() as usize;
    let mut raw_got = raw_got.iter();

    // Consume the first `local_gotno` elements of `raw_got`
    let locals = raw_got
        .by_ref()
        .take(local_gotno)
        .map(|v| GotLocalEntry::new(*v))
        .collect();

    let elf_endian = elf_file.endian();
    let dynsym = elf_file.elf_dynamic_symbol_table();
    let dynstr = dynsym.strings();

    // `zip` the dynamic symtab starting at gotsym with the remaining `raw_got`
    let globals = dynsym
        .iter()
        .skip(gotsym)
        .zip(raw_got)
        .map(|(entry, initial)| {
            let sym_val = entry.st_value(elf_endian);
            let undef_com_or_abs = matches!(
                entry.st_shndx(elf_endian),
                elf::SHN_UNDEF | elf::SHN_COMMON | elf::SHN_ABS
            );

            let raw_name = utils::pretty_unwrap(entry.name(elf_endian, dynstr));
            let sym_name = utils::pretty_unwrap(String::from_utf8(raw_name.into()));
            GotGlobalEntry::new(*initial, sym_val, undef_com_or_abs, sym_name)
        })
        .collect();

    GlobalOffsetTable::new(got_vram, locals, globals)
}
