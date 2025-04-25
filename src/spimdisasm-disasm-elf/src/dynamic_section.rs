/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use spimdisasm::{addresses::GpValue, config::Endian};

use crate::elf_section_type::{ElfSectionType, RawElfSection};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicSection {
    // These three members are required to have according to the abi.
    pltgot: u32,
    local_gotno: u32,
    gotsym: u32,
}

impl DynamicSection {
    #[must_use]
    pub fn parse_from_raw_section(raw_elf_section: RawElfSection) -> Option<Self> {
        assert_eq!(raw_elf_section.section_type(), ElfSectionType::Dynamic);

        Self::parse_impl(raw_elf_section.data(), raw_elf_section.endian())
    }

    fn parse_impl(data: &[u8], endian: Endian) -> Option<Self> {
        let mut pltgot = None;
        let mut local_gotno = None;
        let mut gotsym = None;

        for entry in data.chunks_exact(8) {
            let tag = endian.word_from_bytes(&entry[0..]);
            let val = endian.word_from_bytes(&entry[4..]);

            match tag {
                object::elf::DT_PLTGOT => pltgot = Some(val),
                object::elf::DT_MIPS_LOCAL_GOTNO => local_gotno = Some(val),
                object::elf::DT_MIPS_GOTSYM => gotsym = Some(val),
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

    pub fn canonical_gp(&self) -> GpValue {
        GpValue::new(self.pltgot + 0x7FF0)
    }
    pub fn local_gotno(&self) -> u32 {
        self.local_gotno
    }
    pub fn gotsym(&self) -> u32 {
        self.gotsym
    }
}
