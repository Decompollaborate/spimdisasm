/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use object::{elf, read::elf::ElfFile32, Object, ObjectSection};

use crate::{elf_section_type::ElfSectionType, utils};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicSection {
    pltgot: Option<u32>,
    local_gotno: Option<u32>,
    gotsym: Option<u32>,
}

impl DynamicSection {
    pub fn parse(elf_file: &ElfFile32) -> Option<Self> {
        let endian = utils::endian_to_endian(elf_file.endian());

        for section in elf_file.sections() {
            let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
            let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

            let section_type =
                ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
            if let Some(ElfSectionType::Dynamic) = section_type {
                let mut pltgot = None;
                let mut local_gotno = None;
                let mut gotsym = None;

                let data = utils::pretty_unwrap(section.data());
                for entry in data.windows(8) {
                    let tag = endian.word_from_bytes(&entry[0..]);
                    let val = endian.word_from_bytes(&entry[4..]);

                    match tag {
                        elf::DT_PLTGOT => pltgot = Some(val),
                        elf::DT_MIPS_LOCAL_GOTNO => local_gotno = Some(val),
                        elf::DT_MIPS_GOTSYM => gotsym = Some(val),
                        _ => {}
                    }
                }

                return Some(Self {
                    pltgot,
                    local_gotno,
                    gotsym,
                });
            }
        }

        None
    }

    pub fn gp(&self) -> Option<u32> {
        self.pltgot.map(|x| x + 0x7FF0)
    }
    pub fn local_gotno(&self) -> Option<u32> {
        self.local_gotno
    }
    pub fn gotsym(&self) -> Option<u32> {
        self.gotsym
    }
}
