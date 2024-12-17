/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    string::String,
    vec::Vec,
};
use rabbitizer::Vram;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    address_range::AddressRange,
    config::Compiler,
    context::{Context, OwnedSegmentNotFoundError},
    metadata::{segment_metadata::FindSettings, GeneratedBy, ParentSectionMetadata, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    rom_address::RomAddress,
    rom_vram_range::RomVramRange,
    section_type::SectionType,
    size::Size,
    symbols::{symbol_data::SymbolDataProperties, Symbol, SymbolData},
};

use super::{trait_section::RomSection, Section};

#[derive(Debug, Clone, Hash, PartialEq)]
#[must_use]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionData {
    name: String,

    ranges: RomVramRange,

    parent_segment_info: ParentSegmentInfo,

    // in_section_offset: u32,
    section_type: SectionType,

    //
    data_symbols: Vec<SymbolData>,

    symbol_vrams: BTreeSet<Vram>,
}

impl SectionData {
    // TODO: fix
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &mut Context,
        settings: &SectionDataSettings,
        name: String,
        raw_bytes: &[u8],
        rom: RomAddress,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
        section_type: SectionType,
    ) -> Result<Self, OwnedSegmentNotFoundError> {
        assert!(
            !raw_bytes.is_empty(),
            "Can't initialize a section with empty bytes. {:?} {:?}",
            rom,
            vram
        );
        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let mut data_symbols = Vec::new();
        let mut symbol_vrams = BTreeSet::new();

        let owned_segment = context.find_owned_segment(&parent_segment_info)?;

        let mut symbols_info = BTreeSet::new();
        // Ensure there's a symbol at the beginning of the section.
        symbols_info.insert(vram);

        let mut maybe_pointers_to_other_sections = Vec::new();

        let mut auto_pads: BTreeMap<Vram, Vram> = BTreeMap::new();

        let mut remaining_string_size = -1;
        let mut prev_string_vram = vram;
        let mut pointers_in_data_to_remove = BTreeSet::new();
        let mut syms_to_drop = BTreeSet::new();

        // Look for stuff that looks like addresses which point to symbols on this section
        let displacement = (4 - (vram.inner() % 4) as usize) % 4;
        // TODO: check for symbols in the displacement and everything that the `chunk_exact` may have left out
        for (i, word_bytes) in raw_bytes[displacement..].chunks_exact(4).enumerate() {
            let local_offset = i * 4 + displacement;

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

            // Avoid symbols in the middle of strings
            if remaining_string_size < 0 {
                if let Some(sym) = a {
                    if sym.sym_type() == Some(&SymbolType::CString) {
                        if let Some(str_end) =
                            raw_bytes[local_offset..].iter().position(|x| *x == 0)
                        {
                            remaining_string_size = str_end as i32;
                            prev_string_vram = current_vram;

                            symbols_info.insert(current_vram);

                            let next_vram = if let Some(size) = sym.user_declared_size() {
                                current_vram + size
                            } else {
                                current_vram + Size::new((str_end + 1).next_multiple_of(4) as u32)
                            };
                            if ((next_vram - vram).inner() as usize) < raw_bytes.len() {
                                // Avoid generating a symbol at the end of the section
                                symbols_info.insert(next_vram);
                                auto_pads.insert(next_vram, current_vram);
                            }
                        }
                    }
                }
            }

            if remaining_string_size < 0 {
                if b.is_none() && c.is_none() && d.is_none() {
                    // There's no symbol in between

                    let should_search_for_address = match a {
                        None => true,
                        Some(metadata) => metadata
                            .sym_type()
                            .is_none_or(|x| x.can_reference_symbols()),
                    };

                    if should_search_for_address {
                        // TODO: improve heuristic to determine if should search for symbols
                        let word = context.global_config().endian().word_from_bytes(word_bytes);
                        let word_vram = Vram::new(word);
                        if vram_range.in_range(word_vram) {
                            // Vram is contained in this section
                            if let Some(sym) = owned_segment.find_symbol(
                                word_vram,
                                FindSettings::default().with_allow_addend(true),
                            ) {
                                if sym.vram() == word_vram {
                                    // Only count this symbol if it doesn't have an addend.
                                    // If it does have an addend then it may be part of a larger symbol.
                                    symbols_info.insert(word_vram);
                                }
                            } else {
                                symbols_info.insert(word_vram);
                            }
                        } else {
                            let current_rom = rom + (current_vram - vram).try_into().expect("This should not panic because `current_vram` should always be greter or equal to `vram`");
                            let sym = context
                                .find_referenced_segment(word_vram, &parent_segment_info)
                                .and_then(|seg| {
                                    seg.find_symbol(word_vram, FindSettings::default())
                                });
                            if sym.is_none() {
                                maybe_pointers_to_other_sections.push((word_vram, current_rom));
                            }
                        }
                    }
                }

                for (x_vram, x) in [(current_vram, a), (b_vram, b), (c_vram, c), (d_vram, d)] {
                    if let Some(sym) = x {
                        symbols_info.insert(sym.vram());
                        if let Some(size) = sym.user_declared_size() {
                            let next_vram = sym.vram() + size;
                            if ((next_vram - vram).inner() as usize) < raw_bytes.len() {
                                // Avoid generating a symbol at the end of the section
                                symbols_info.insert(next_vram);
                                auto_pads.insert(next_vram, sym.vram());
                            }
                        }
                    } else if owned_segment.is_vram_a_possible_pointer_in_data(x_vram) {
                        symbols_info.insert(x_vram);
                    }
                }
            } else {
                for (x_vram, x) in [(current_vram, a), (b_vram, b), (c_vram, c), (d_vram, d)] {
                    if x_vram == prev_string_vram {
                        continue;
                    }

                    if let Some(sym) = x {
                        if sym.generated_by() == GeneratedBy::Autogenerated {
                            syms_to_drop.insert(x_vram);
                        }
                    }
                    pointers_in_data_to_remove.insert(x_vram);
                }
            }

            remaining_string_size -= 4;
        }

        let symbols_info_vec: Vec<Vram> = symbols_info.into_iter().collect();

        for (i, new_sym_vram) in symbols_info_vec.iter().enumerate() {
            let start = new_sym_vram.sub_vram(&vram).inner() as usize;
            let end = if i + 1 < symbols_info_vec.len() {
                symbols_info_vec[i + 1].sub_vram(&vram).inner() as usize
            } else {
                raw_bytes.len()
            };
            debug_assert!(
                start < end,
                "{:?} {} {} {} {}",
                rom,
                vram,
                start,
                end,
                raw_bytes.len()
            );

            let sym_rom = rom + Size::new(start as u32);

            symbol_vrams.insert(*new_sym_vram);

            let properties = SymbolDataProperties {
                parent_metadata: ParentSectionMetadata::new(
                    name.clone(),
                    vram,
                    parent_segment_info.clone(),
                ),
                compiler: settings.compiler,
                auto_pad_by: auto_pads.get(new_sym_vram).copied(),
            };
            let /*mut*/ sym = SymbolData::new(context, raw_bytes[start..end].into(), sym_rom, *new_sym_vram, start, parent_segment_info.clone(), section_type, properties)?;

            data_symbols.push(sym);
        }

        let owned_segment_mut = context.find_owned_segment_mut(&parent_segment_info)?;
        for (possible_pointer, rom_address_referencing_pointer) in maybe_pointers_to_other_sections
        {
            owned_segment_mut
                .add_possible_pointer_in_data(possible_pointer, rom_address_referencing_pointer);
        }
        for possible_pointer in pointers_in_data_to_remove {
            owned_segment_mut.drop_possible_pointer_in_data(possible_pointer);
        }
        for sym_to_drop in syms_to_drop {
            owned_segment_mut.drop_symbol(sym_to_drop);
        }

        Ok(Self {
            name,
            ranges,
            parent_segment_info,
            section_type,
            data_symbols,
            symbol_vrams,
        })
    }

    pub fn data_symbols(&self) -> &[SymbolData] {
        &self.data_symbols
    }
}

impl Section for SectionData {
    fn name(&self) -> &str {
        &self.name
    }

    fn vram_range(&self) -> &AddressRange<Vram> {
        self.ranges.vram()
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    #[must_use]
    fn section_type(&self) -> SectionType {
        self.section_type
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.data_symbols
    }

    fn symbols_vrams(&self) -> &BTreeSet<Vram> {
        &self.symbol_vrams
    }
}

impl RomSection for SectionData {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionDataSettings {
    compiler: Option<Compiler>,
}

impl SectionDataSettings {
    pub fn new(compiler: Option<Compiler>) -> Self {
        Self { compiler }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use crate::symbols::display::{SymDataDisplaySettings, SymDisplayError};

    use super::*;

    #[pymethods]
    impl SectionDataSettings {
        #[new]
        #[pyo3(signature = (compiler))]
        pub fn py_new(compiler: Option<Compiler>) -> Self {
            Self::new(compiler)
        }
    }

    #[pymethods]
    impl SectionData {
        #[pyo3(name = "sym_count")]
        pub fn py_sym_count(&self) -> usize {
            self.data_symbols.len()
        }

        #[pyo3(name = "display_sym")]
        pub fn py_display_sym(
            &self,
            context: &Context,
            index: usize,
            settings: &SymDataDisplaySettings,
        ) -> Result<Option<String>, SymDisplayError> {
            let sym = self.data_symbols.get(index);

            Ok(if let Some(sym) = sym {
                Some(sym.display(context, settings)?.to_string())
            } else {
                None
            })
        }
    }
}