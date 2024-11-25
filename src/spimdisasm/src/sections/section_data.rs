/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_set::BTreeSet, string::String, vec::Vec};
use rabbitizer::Vram;

use crate::{
    context::{Context, OwnedSegmentNotFoundError},
    metadata::segment_metadata::FindSettings,
    parent_segment_info::ParentSegmentInfo,
    rom_address::RomAddress,
    size::Size,
    symbols::{Symbol, SymbolData},
};

use super::{Section, SectionBase};

pub struct SectionDataSettings {}

impl SectionDataSettings {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct SectionData {
    section_base: SectionBase,

    data_symbols: Vec<SymbolData>,

    // TODO: maybe move to SectionBase or just Section?
    symbol_vrams: BTreeSet<Vram>,
}

impl SectionData {
    pub fn new(
        context: &mut Context,
        _settings: SectionDataSettings,
        name: String,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<Self, OwnedSegmentNotFoundError> {
        assert!(
            !raw_bytes.is_empty(),
            "Can't initialize a section with empty bytes. {:?} {:?}",
            rom,
            vram
        );

        let section_size = Size::new(raw_bytes.len() as u32);
        let vram_end = vram + section_size;

        let mut data_symbols = Vec::new();
        let mut symbol_vrams = BTreeSet::new();

        let owned_segment = context.find_owned_segment(&parent_segment_info)?;

        let mut symbols_info = BTreeSet::new();
        // Ensure there's a symbol at the beginning of the section.
        symbols_info.insert(vram);

        let mut maybe_pointers_to_other_sections = Vec::new();

        // Look for stuff that looks like addresses which point to symbols on this section
        let displacement = (4 - (vram.inner() % 4) as usize) % 4;
        // TODO: check for symbols in the displacement and everything that the `chunk_exact` may have left out
        for (i, word_bytes) in raw_bytes[displacement..].chunks_exact(4).enumerate() {
            let local_offset = i * 4 + displacement;
            /*
            let word = context.global_config().endian().word_from_bytes(word_bytes);

            owned_segment.find_symbol(word, FindSettings::new());
            */
            let current_vram = vram + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);
            let a = owned_segment.find_symbol(
                current_vram,
                FindSettings::default().with_allow_addend(false),
            );
            let b =
                owned_segment.find_symbol(b_vram, FindSettings::default().with_allow_addend(false));
            let c =
                owned_segment.find_symbol(c_vram, FindSettings::default().with_allow_addend(false));
            let d =
                owned_segment.find_symbol(d_vram, FindSettings::default().with_allow_addend(false));

            if b.is_none() && c.is_none() && d.is_none() {
                // There's no symbol in between

                let word = context.global_config().endian().word_from_bytes(word_bytes);
                let word_vram = Vram::new(word);
                if vram <= word_vram && word_vram < vram_end {
                    // Vram is contained in this section
                    if let Some(sym) = owned_segment.find_symbol(
                        current_vram,
                        FindSettings::default().with_allow_addend(true),
                    ) {
                        if sym.vram() == word_vram {
                            // Only count this symbol if it doesn't have an addend
                            symbols_info.insert(word_vram);
                        }
                    }
                } else {
                    let current_rom = rom + (current_vram - vram).try_into().expect("This should not panic because `current_vram` should always be greter or equal to `vram`");
                    maybe_pointers_to_other_sections.push((word_vram, current_rom));
                }
            }

            for (x_vram, x) in [(current_vram, a), (b_vram, b), (c_vram, c), (d_vram, d)] {
                if let Some(sym) = x {
                    symbols_info.insert(sym.vram());
                } else if owned_segment.is_vram_a_possible_pointer_in_data(x_vram) {
                    symbols_info.insert(x_vram);
                }
            }
        }

        let symbols_info_vec: Vec<Vram> = symbols_info.into_iter().collect();

        for (i, new_vram_sym) in symbols_info_vec.iter().enumerate() {
            let start = new_vram_sym.sub_vram(&vram).inner() as usize;
            let end = if i + 1 < symbols_info_vec.len() {
                symbols_info_vec[i + 1].sub_vram(&vram).inner() as usize
            } else {
                raw_bytes.len()
            };
            debug_assert!(start < end, "{:?} {} {} {}", rom, vram, start, end);

            let local_offset = start * 4;
            let rom = rom + Size::new(local_offset as u32);

            symbol_vrams.insert(*new_vram_sym);

            // TODO: get rid of unwrap?
            let /*mut*/ sym = SymbolData::new(context, raw_bytes[start..end].into(), rom, *new_vram_sym, local_offset, &parent_segment_info)?;

            data_symbols.push(sym);
        }

        let owned_segment_mut = context.find_owned_segment_mut(&parent_segment_info)?;
        for (possible_pointer, rom_address_referencing_pointer) in maybe_pointers_to_other_sections
        {
            owned_segment_mut
                .add_possible_pointer_in_data(possible_pointer, rom_address_referencing_pointer);
        }

        Ok(Self {
            section_base: SectionBase::new(name, Some(rom), vram, parent_segment_info),
            data_symbols,
            symbol_vrams,
        })
    }

    pub fn name(&self) -> &str {
        self.section_base.name()
    }

    // TODO: remove
    pub fn data_symbols(&self) -> &[SymbolData] {
        &self.data_symbols
    }
}

impl Section for SectionData {
    fn section_base(&self) -> &SectionBase {
        &self.section_base
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.data_symbols
    }

    fn symbols_vrams(&self) -> &BTreeSet<Vram> {
        &self.symbol_vrams
    }
}
