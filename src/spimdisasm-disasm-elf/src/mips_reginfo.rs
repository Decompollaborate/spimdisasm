/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use object::{read::elf::ElfFile32, Object, ObjectSection};
use spimdisasm::{addresses::GpValue, config::Endian};

use crate::{elf_section_type::ElfSectionType, utils};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MipsReginfo {
    ri_gprmask: u32,
    ri_cprmask: [u32; 4],
    ri_gp_value: GpValue,
}

impl MipsReginfo {
    pub fn parse_from_elf(elf_file: &ElfFile32) -> Option<Self> {
        for section in elf_file.sections() {
            let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
            let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

            let section_type =
                ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
            if let Some(ElfSectionType::MipsReginfo) = section_type {
                let data = utils::pretty_unwrap(section.data());
                let endian = utils::endian_to_endian(elf_file.endian());

                return Self::parse_impl(data, endian);
            }
        }

        None
    }

    fn parse_impl(data: &[u8], endian: Endian) -> Option<Self> {
        if data.len() != 0x18 {
            // TODO: Implement errors instead of using Option?
            return None;
        }

        let mut iter = data.chunks_exact(4);

        let ri_gprmask = endian.word_from_bytes(
            iter.next()
                .expect("Shouldn't panic since size was already checked"),
        );

        let a = endian.word_from_bytes(
            iter.next()
                .expect("Shouldn't panic since size was already checked"),
        );
        let b = endian.word_from_bytes(
            iter.next()
                .expect("Shouldn't panic since size was already checked"),
        );
        let c = endian.word_from_bytes(
            iter.next()
                .expect("Shouldn't panic since size was already checked"),
        );
        let d = endian.word_from_bytes(
            iter.next()
                .expect("Shouldn't panic since size was already checked"),
        );
        let ri_cprmask = [a, b, c, d];

        let ri_gp_value = GpValue::new(
            endian.word_from_bytes(
                iter.next()
                    .expect("Shouldn't panic since size was already checked"),
            ),
        );

        Some(Self {
            ri_gprmask,
            ri_cprmask,
            ri_gp_value,
        })
    }

    #[must_use]
    pub fn ri_gp_value(&self) -> GpValue {
        self.ri_gp_value
    }
}
