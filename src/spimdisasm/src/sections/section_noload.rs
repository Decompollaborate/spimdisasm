/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_set::BTreeSet, string::String, vec::Vec};
use rabbitizer::Vram;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    address_range::AddressRange,
    context::{Context, OwnedSegmentNotFoundError},
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    symbols::{Symbol, SymbolNoload},
};

use super::Section;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionNoloadSettings {}

impl SectionNoloadSettings {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for SectionNoloadSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
#[must_use]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionNoload {
    name: String,

    vram_range: AddressRange<Vram>,

    parent_segment_info: ParentSegmentInfo,

    // in_section_offset: u32,

    //
    noload_symbols: Vec<SymbolNoload>,

    symbol_vrams: BTreeSet<Vram>,
}

impl SectionNoload {
    pub(crate) fn new(
        context: &mut Context,
        _settings: &SectionNoloadSettings,
        name: String,
        vram_range: AddressRange<Vram>,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<Self, OwnedSegmentNotFoundError> {
        assert!(
            vram_range.size().inner() != 0,
            "Can't initialize zero-sized noload section. {:?}",
            vram_range
        );

        let mut noload_symbols = Vec::new();
        let mut symbol_vrams = BTreeSet::new();

        // let owned_segment = context.find_owned_segment(&parent_segment_info)?;

        let mut symbols_info = BTreeSet::new();
        // Ensure there's a symbol at the beginning of the section.
        symbols_info.insert(vram_range.start());

        // let mut maybe_pointers_to_other_sections = Vec::new();

        // TODO: fill `symbols_info``

        let symbols_info_vec: Vec<Vram> = symbols_info.into_iter().collect();

        for (i, new_sym_vram) in symbols_info_vec.iter().enumerate() {
            let start = new_sym_vram.sub_vram(&vram_range.start()).inner() as usize;
            let new_sym_vram_end = if i + 1 < symbols_info_vec.len() {
                symbols_info_vec[i + 1]
            } else {
                vram_range.end()
            };
            debug_assert!(
                *new_sym_vram < new_sym_vram_end,
                "{:?} {} {}",
                vram_range,
                new_sym_vram,
                new_sym_vram_end
            );

            symbol_vrams.insert(*new_sym_vram);

            let /*mut*/ sym = SymbolNoload::new(context, AddressRange::new(*new_sym_vram, new_sym_vram_end), start, parent_segment_info.clone())?;

            noload_symbols.push(sym);
        }

        // let owned_segment_mut = context.find_owned_segment_mut(&parent_segment_info)?;
        // for (possible_pointer, rom_address_referencing_pointer) in maybe_pointers_to_other_sections
        // {
        //     owned_segment_mut
        //         .add_possible_pointer_in_data(possible_pointer, rom_address_referencing_pointer);
        // }

        Ok(Self {
            name,
            vram_range,
            parent_segment_info,
            noload_symbols,
            symbol_vrams,
        })
    }

    // TODO: remove
    pub fn noload_symbols(&self) -> &[SymbolNoload] {
        &self.noload_symbols
    }
}

impl Section for SectionNoload {
    fn name(&self) -> &str {
        &self.name
    }

    fn vram_range(&self) -> &AddressRange<Vram> {
        &self.vram_range
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    #[must_use]
    fn section_type(&self) -> SectionType {
        SectionType::Bss
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.noload_symbols
    }

    fn symbols_vrams(&self) -> &BTreeSet<Vram> {
        &self.symbol_vrams
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl SectionNoloadSettings {
        #[new]
        pub fn py_new() -> Self {
            Self::new()
        }
    }

    #[pymethods]
    impl SectionNoload {
        #[pyo3(name = "sym_count")]
        pub fn py_sym_count(&self) -> usize {
            self.noload_symbols.len()
        }

        /*
        #[pyo3(name = "display_sym")]
        pub fn py_display_sym(
            &self,
            context: &Context,
            index: usize,
            settings: &SymDataDisplaySettings,
        ) -> Option<String> {
            self.noload_symbols
                .get(index)
                .map(|sym| sym.display(context, settings).to_string())
        }
        */
    }
}
