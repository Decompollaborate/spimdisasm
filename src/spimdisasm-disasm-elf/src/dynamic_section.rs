/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use object::{elf, read::elf::ElfFile32, Object, ObjectSection};
use spimdisasm::{addresses::GpValue, config::Endian};

use crate::{elf_section_type::ElfSectionType, utils};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicSection {
    // These three members are required to have according to the abi.
    pltgot: u32,
    local_gotno: u32,
    gotsym: u32,
}

impl DynamicSection {
    pub fn parse(elf_file: &ElfFile32) -> Option<Self> {
        for section in elf_file.sections() {
            let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
            let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

            let section_type =
                ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
            if let Some(ElfSectionType::Dynamic) = section_type {
                let data = utils::pretty_unwrap(section.data());
                let endian = utils::endian_to_endian(elf_file.endian());

                return Self::parse_impl(data, endian);
            }
        }

        None
    }

    fn parse_impl(data: &[u8], endian: Endian) -> Option<Self> {
        let mut pltgot = None;
        let mut local_gotno = None;
        let mut gotsym = None;

        for entry in data.chunks_exact(8) {
            let tag = endian.word_from_bytes(&entry[0..]);
            let val = endian.word_from_bytes(&entry[4..]);

            match tag {
                elf::DT_PLTGOT => pltgot = Some(val),
                elf::DT_MIPS_LOCAL_GOTNO => local_gotno = Some(val),
                elf::DT_MIPS_GOTSYM => gotsym = Some(val),
                _ => {}
            }
        }

        if let (Some(pltgot), Some(local_gotno), Some(gotsym)) = (pltgot, local_gotno, gotsym) {
            Some(Self {
                pltgot,
                local_gotno,
                gotsym,
            })
        } else {
            None
        }
    }

    pub fn gp(&self) -> GpValue {
        GpValue::new(self.pltgot + 0x7FF0)
    }
    pub fn local_gotno(&self) -> u32 {
        self.local_gotno
    }
    pub fn gotsym(&self) -> u32 {
        self.gotsym
    }
}
