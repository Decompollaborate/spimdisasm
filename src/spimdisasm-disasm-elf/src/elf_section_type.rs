/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use object::elf;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElfSectionType {
    Progbits(ProgbitsType),
    Nobits,
    Reloc,
    DynSym,
    Dynamic,
}

impl ElfSectionType {
    pub fn new(sh_type: u32, sh_flags: u32, name: &str) -> Option<Self> {
        match sh_type {
            elf::SHT_PROGBITS => Some(ElfSectionType::Progbits(ProgbitsType::new(sh_flags, name))),
            elf::SHT_NOBITS => Some(ElfSectionType::Nobits),
            elf::SHT_REL => Some(ElfSectionType::Reloc),
            elf::SHT_DYNSYM => Some(ElfSectionType::DynSym),
            elf::SHT_DYNAMIC => {
                if name == ".dynamic" {
                    Some(ElfSectionType::Dynamic)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgbitsType {
    Text,
    Data,
    Rodata,
    Got,
    Unknown,
}

impl ProgbitsType {
    fn new(sh_flags: u32, name: &str) -> Self {
        if name == ".got" {
            ProgbitsType::Got
        } else if matches!(name, ".rodata" | ".rdata") {
            // Some compilers set the Write flag on .rodata sections, so we need to hardcode
            // a special check to distinguish it properly
            ProgbitsType::Rodata
        } else if sh_flags & elf::SHF_ALLOC == 0 {
            ProgbitsType::Unknown
        } else if sh_flags & elf::SHF_EXECINSTR != 0 {
            ProgbitsType::Text
        } else if sh_flags & elf::SHF_WRITE != 0 {
            ProgbitsType::Data
        } else {
            ProgbitsType::Rodata
        }
    }
}
