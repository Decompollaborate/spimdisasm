/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::sync::Arc;

use object::ObjectSection;
use spimdisasm::{
    addresses::{Rom, Size, Vram},
    config::Endian,
};

use crate::utils;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawElfSection<'data> {
    section_type: ElfSectionType,
    name: Arc<str>,
    address: Vram,
    offset: Rom,
    size: Size,
    // TODO: Consider adding the other stuff from the section header (ES Flg Lk Inf Al)
    data: &'data [u8],
    endian: Endian,
}

impl<'data> RawElfSection<'data> {
    pub fn new(
        elf_section: object::read::elf::ElfSection<
            'data,
            '_,
            object::elf::FileHeader32<object::Endianness>,
        >,
    ) -> Option<Self> {
        let elf_endian = elf_section.elf_file().endian();
        let endian = utils::endian_to_endian(elf_endian);

        let elf_section_header = elf_section.elf_section_header();

        let size = {
            let sh_size = elf_section_header.sh_size.get(elf_endian);
            if sh_size == 0 {
                return None;
            } else {
                Size::new(sh_size)
            }
        };

        let name = utils::pretty_unwrap(elf_section.name());

        let sh_flags = elf_section_header.sh_flags.get(elf_endian);
        let sh_type = elf_section_header.sh_type.get(elf_endian);

        ElfSectionType::new(sh_type, sh_flags, name).map(|section_type| {
            let name = name.into();
            let address = Vram::new(elf_section_header.sh_addr.get(elf_endian));
            let offset = Rom::new(elf_section_header.sh_offset.get(elf_endian));

            let data = utils::pretty_unwrap(elf_section.data());

            Self {
                section_type,
                name,
                address,
                offset,
                size,
                data,
                endian,
            }
        })
    }

    #[must_use]
    pub fn section_type(&self) -> ElfSectionType {
        self.section_type
    }
    #[must_use]
    pub fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    #[must_use]
    pub fn address(&self) -> Vram {
        self.address
    }
    #[must_use]
    pub fn offset(&self) -> Rom {
        self.offset
    }
    #[must_use]
    pub fn size(&self) -> Size {
        self.size
    }
    #[must_use]
    pub fn data(&self) -> &[u8] {
        self.data
    }
    #[must_use]
    pub fn endian(&self) -> Endian {
        self.endian
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElfSectionType {
    Progbits(ProgbitsType),
    Nobits,
    Reloc,
    MipsReginfo,
    Dynamic,
}

impl ElfSectionType {
    pub fn new(sh_type: u32, sh_flags: u32, name: &str) -> Option<Self> {
        match sh_type {
            object::elf::SHT_PROGBITS => {
                Some(ElfSectionType::Progbits(ProgbitsType::new(sh_flags, name)))
            }
            object::elf::SHT_NOBITS => Some(ElfSectionType::Nobits),
            object::elf::SHT_REL => Some(ElfSectionType::Reloc),
            object::elf::SHT_MIPS_REGINFO if name == ".reginfo" => {
                Some(ElfSectionType::MipsReginfo)
            }
            object::elf::SHT_DYNAMIC if name == ".dynamic" => Some(ElfSectionType::Dynamic),
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
        } else if sh_flags & object::elf::SHF_ALLOC == 0 {
            ProgbitsType::Unknown
        } else if sh_flags & object::elf::SHF_EXECINSTR != 0 {
            ProgbitsType::Text
        } else if sh_flags & object::elf::SHF_WRITE != 0 {
            ProgbitsType::Data
        } else {
            ProgbitsType::Rodata
        }
    }
}
