/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use spimdisasm::{addresses::GpValue, config::Endian};

use crate::elf_section_type::{ElfSectionType, RawElfSection};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MipsReginfo {
    ri_gprmask: u32,
    ri_cprmask: [u32; 4],
    ri_gp_value: Option<GpValue>,
}

impl MipsReginfo {
    #[must_use]
    pub fn parse_from_raw_section(raw_elf_section: RawElfSection) -> Option<Self> {
        assert_eq!(raw_elf_section.section_type(), ElfSectionType::MipsReginfo);

        Self::parse_impl(raw_elf_section.data(), raw_elf_section.endian())
    }

    #[must_use]
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

        let ri_gp_value = {
            let word = endian.word_from_bytes(
                iter.next()
                    .expect("Shouldn't panic since size was already checked"),
            );
            if word != 0 {
                Some(GpValue::new(word))
            } else {
                None
            }
        };

        Some(Self {
            ri_gprmask,
            ri_cprmask,
            ri_gp_value,
        })
    }

    #[must_use]
    pub fn ri_gp_value(&self) -> Option<GpValue> {
        self.ri_gp_value
    }
}
