/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::StringGuesserLevel,
    collections::{
        addended_ordered_map::FindSettings, unordered_map::UnorderedMap,
        unordered_set::UnorderedSet,
    },
    config::Compiler,
    context::Context,
    metadata::{ParentSectionMetadata, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    str_decoding::Encoding,
    symbols::{symbol_data::SymbolDataProperties, Symbol, SymbolData},
};

use super::{trait_section::RomSection, Section, SectionCreationError, SectionPostProcessError};

#[derive(Debug, Clone, PartialEq)]
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

    symbol_vrams: UnorderedSet<Vram>,
}

impl SectionData {
    // TODO: fix
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &mut Context,
        settings: &SectionDataSettings,
        name: String,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
        section_type: SectionType,
    ) -> Result<Self, SectionCreationError> {
        if raw_bytes.is_empty() {
            return Err(SectionCreationError::EmptySection { name });
        }

        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let mut data_symbols = Vec::new();
        let mut symbol_vrams = UnorderedSet::new();

        let owned_segment = context.find_owned_segment(&parent_segment_info)?;

        let mut symbols_info = BTreeMap::new();
        // Ensure there's a symbol at the beginning of the section.
        symbols_info.insert(vram, (None,));

        let mut maybe_pointers_to_other_sections = Vec::new();

        let mut auto_pads: UnorderedMap<Vram, Vram> = UnorderedMap::new();

        let mut remaining_string_size = 0;

        let mut prev_sym_type: Option<SymbolType> = None;
        // If true: the previous symbol made us thought we may be in late_rodata
        let mut maybe_reached_late_rodata = false;
        // If true, we are sure we are in late_rodata
        let mut reached_late_rodata = false;

        let mut float_counter = 0;
        let mut float_padding_counter = 0;

        // Look for stuff that looks like addresses which point to symbols on this section
        let displacement = (4 - (vram.inner() % 4) as usize) % 4;
        // TODO: check for symbols in the displacement and everything that the `chunk_exact` may have left out
        for (i, word_bytes) in raw_bytes[displacement..].chunks_exact(4).enumerate() {
            let local_offset = i * 4 + displacement;

            let current_vram = vram + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);

            // Avoid symbols in the middle of strings
            if remaining_string_size <= 0 {
                let current_ref =
                    owned_segment.find_reference(current_vram, FindSettings::new(true));

                if current_ref.is_none_or(|x| x.vram() == current_vram) {
                    if let Some(str_size) = settings.string_guesser_level.guess(
                        current_ref,
                        current_vram,
                        &raw_bytes[local_offset..],
                        settings.encoding,
                        maybe_reached_late_rodata || reached_late_rodata,
                    ) {
                        let str_sym_size = str_size.next_multiple_of(4);
                        let in_between_sym = owned_segment.find_reference(
                            current_vram + Size::new(str_sym_size as u32 - 1),
                            FindSettings::new(true).with_reject_sizeless_addended(false),
                        );

                        if in_between_sym.is_none_or(|x| {
                            let other_sym_vram = x.vram();

                            match other_sym_vram.cmp(&current_vram) {
                                core::cmp::Ordering::Greater => false,
                                core::cmp::Ordering::Equal => true,
                                core::cmp::Ordering::Less => {
                                    x.size().is_some_and(|x| other_sym_vram + x <= current_vram)
                                }
                            }
                        }) {
                            // Check if there is already another symbol after the current one and before the end of the string,
                            // in which case we say this symbol should not be a string

                            remaining_string_size = str_size as i32;

                            symbols_info.insert(current_vram, (Some(SymbolType::CString),));
                            if !auto_pads.contains_key(&current_vram) {
                                auto_pads.insert(current_vram, current_vram);
                            }

                            let next_vram = current_vram + Size::new(str_sym_size as u32);
                            if ((next_vram - vram).inner() as usize) < raw_bytes.len() {
                                // Avoid generating a symbol at the end of the section
                                symbols_info.insert(next_vram, (None,));
                                auto_pads.insert(next_vram, current_vram);
                            }

                            // Next symbol should not be affected by this string.
                            prev_sym_type = None;
                        }
                    }
                }
            }

            if remaining_string_size <= 0 {
                let a = owned_segment.find_reference(current_vram, FindSettings::new(false));
                let b = owned_segment.find_reference(b_vram, FindSettings::new(false));
                let c = owned_segment.find_reference(c_vram, FindSettings::new(false));
                let d = owned_segment.find_reference(d_vram, FindSettings::new(false));

                if b.is_none() && c.is_none() && d.is_none() {
                    // There's no symbol in between

                    let current_type = match a {
                        None => prev_sym_type,
                        Some(wrapper) => wrapper.sym_type(),
                    };
                    let should_search_for_address =
                        current_type.is_none_or(|x| x.can_reference_symbols());

                    let endian = context.global_config().endian();
                    let word = endian.word_from_bytes(word_bytes);
                    if should_search_for_address {
                        // TODO: improve heuristic to determine if should search for symbols
                        let word_vram = Vram::new(word);
                        if vram_range.in_range(word_vram) {
                            // Vram is contained in this section
                            if let Some(reference) =
                                owned_segment.find_reference(word_vram, FindSettings::new(true))
                            {
                                if reference.vram() == word_vram {
                                    // Only count this symbol if it doesn't have an addend.
                                    // If it does have an addend then it may be part of a larger symbol.
                                    symbols_info.insert(word_vram, (None,));
                                }
                            } else {
                                symbols_info.insert(word_vram, (None,));
                            }
                        } else {
                            let current_rom = rom + (current_vram - vram).try_into().expect("This should not panic because `current_vram` should always be greter or equal to `vram`");
                            let reference = context.find_symbol_from_any_segment(
                                word_vram,
                                &parent_segment_info,
                                FindSettings::new(true),
                            );
                            if reference.is_none() {
                                maybe_pointers_to_other_sections.push((word_vram, current_rom));
                            }
                        }
                    }

                    if maybe_reached_late_rodata
                        && matches!(
                            current_type,
                            Some(SymbolType::Float32 | SymbolType::Float64)
                        )
                        && a.is_some()
                    {
                        reached_late_rodata = true;
                    }

                    if let Some(a) = a {
                        if matches!(
                            a.sym_type(),
                            Some(SymbolType::Float32 | SymbolType::Float64)
                        ) {
                            float_counter = 1;
                            float_padding_counter = 0;
                        } else {
                            float_counter = 0;
                            float_padding_counter = 0;
                        }
                    } else if current_type == Some(SymbolType::Float32) {
                        float_counter += 1;
                        if word == 0 {
                            float_padding_counter += 1;
                        }
                    } else if current_type == Some(SymbolType::Float64) {
                        if current_vram.inner() % 8 == 0 {
                            if local_offset + 8 <= raw_bytes.len() {
                                float_counter += 1;
                                if endian
                                    .dword_from_bytes(&raw_bytes[local_offset..local_offset + 8])
                                    == 0
                                {
                                    float_padding_counter += 1;
                                }
                            } else {
                                float_counter = 0;
                                float_padding_counter = 0;
                            }
                        }
                    } else {
                        float_counter = 0;
                        float_padding_counter = 0;
                    }
                }

                for (x_vram, x) in [(current_vram, a), (b_vram, b), (c_vram, c), (d_vram, d)] {
                    if let Some(reference) = x {
                        symbols_info.insert(reference.vram(), (None,));
                        if let Some(size) = reference.user_declared_size() {
                            let next_vram = reference.vram() + size;
                            if ((next_vram - vram).inner() as usize) < raw_bytes.len() {
                                // Avoid generating a symbol at the end of the section
                                symbols_info.insert(next_vram, (None,));
                                auto_pads.insert(next_vram, reference.vram());
                            }
                        }
                        prev_sym_type = reference.sym_type();
                    } else if owned_segment.is_vram_a_possible_pointer_in_data(x_vram) {
                        symbols_info.insert(x_vram, (None,));
                    }
                }
            }

            maybe_reached_late_rodata = false;
            if !reached_late_rodata
                && section_type == SectionType::Rodata
                && prev_sym_type.is_some_and(|x| x.is_late_rodata(settings.compiler()))
            {
                if prev_sym_type == Some(SymbolType::Jumptable) {
                    reached_late_rodata = true;
                } else if float_padding_counter + 1 == float_counter {
                    // Finding a float or a double is not proof enough to say we are in late_rodata, because we
                    // can be after a const array of floats/doubles.
                    // An example of this is the libultra file `xldtob`.
                    // It is okay for late rodata floats to have padding, but if a float has non-zero padding
                    // it means it isn't a late_rodata float.
                    maybe_reached_late_rodata = true;
                }
            }
            remaining_string_size -= 4;
        }

        let symbols_info_vec: Vec<(Vram, (Option<SymbolType>,))> =
            symbols_info.into_iter().collect();

        for (i, (new_sym_vram, extra_info)) in symbols_info_vec.iter().enumerate() {
            let start = new_sym_vram.sub_vram(&vram).inner() as usize;
            let end = if i + 1 < symbols_info_vec.len() {
                symbols_info_vec[i + 1].0.sub_vram(&vram).inner() as usize
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
                detected_type: extra_info.0,
                encoding: settings.encoding,
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

impl SectionData {
    pub fn post_process(&mut self, context: &mut Context) -> Result<(), SectionPostProcessError> {
        for sym in self.data_symbols.iter_mut() {
            sym.post_process(context)?;
        }

        Ok(())
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

    fn symbols_vrams(&self) -> &UnorderedSet<Vram> {
        &self.symbol_vrams
    }

    fn post_process(&mut self, context: &mut Context) -> Result<(), SectionPostProcessError> {
        self.post_process(context)
    }
}

impl RomSection for SectionData {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionDataSettings {
    compiler: Option<Compiler>,
    string_guesser_level: StringGuesserLevel,
    encoding: Encoding,
}

impl SectionDataSettings {
    pub fn new(compiler: Option<Compiler>) -> Self {
        Self {
            compiler,
            string_guesser_level: StringGuesserLevel::default(),
            encoding: Encoding::default(),
        }
    }

    pub fn compiler(&self) -> Option<Compiler> {
        self.compiler
    }

    pub fn string_guesser_level(&self) -> StringGuesserLevel {
        self.string_guesser_level
    }
    pub fn set_string_guesser_level(&mut self, string_guesser_level: StringGuesserLevel) {
        self.string_guesser_level = string_guesser_level;
    }
    pub fn with_string_guesser_level(self, string_guesser_level: StringGuesserLevel) -> Self {
        Self {
            string_guesser_level,
            ..self
        }
    }

    pub fn encoding(&self) -> Encoding {
        self.encoding
    }
    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.encoding = encoding;
    }
    pub fn with_encoding(self, encoding: Encoding) -> Self {
        Self { encoding, ..self }
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

        #[pyo3(name = "set_string_guesser_level")]
        pub fn py_set_string_guesser_level(&mut self, string_guesser_level: StringGuesserLevel) {
            self.set_string_guesser_level(string_guesser_level)
        }

        #[pyo3(name = "set_encoding")]
        pub fn py_set_encoding(&mut self, encoding: Encoding) {
            self.set_encoding(encoding);
        }
    }

    #[pymethods]
    impl SectionData {
        #[pyo3(name = "post_process")]
        fn py_post_process(
            &mut self,
            context: &mut Context,
        ) -> Result<(), SectionPostProcessError> {
            self.post_process(context)
        }

        #[pyo3(name = "sym_count")]
        pub fn py_sym_count(&self) -> usize {
            self.data_symbols.len()
        }

        #[pyo3(name = "get_sym_info")]
        pub fn py_get_sym_info(
            &self,
            context: &Context,
            index: usize,
        ) -> Option<(
            u32,
            Option<Rom>,
            Option<SymbolType>,
            Option<Size>,
            bool,
            usize,
            Option<String>,
        )> {
            let sym = self.data_symbols.get(index);

            if let Some(sym) = sym {
                let metadata = sym.find_own_metadata(context);

                Some((
                    metadata.vram().inner(),
                    metadata.rom(),
                    metadata.sym_type(),
                    metadata.size(),
                    metadata.is_defined(),
                    metadata.reference_counter(),
                    metadata.parent_metadata().and_then(|x| {
                        x.parent_segment_info()
                            .overlay_category_name()
                            .map(|x| x.inner().to_owned())
                    }),
                ))
            } else {
                None
            }
        }

        #[pyo3(name = "set_sym_name")]
        pub fn py_set_sym_name(&mut self, context: &mut Context, index: usize, new_name: String) {
            let sym = self.data_symbols.get(index);

            if let Some(sym) = sym {
                let metadata = sym.find_own_metadata_mut(context);

                *metadata.user_declared_name_mut() = Some(new_name);
            }
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

        #[pyo3(name = "label_count_for_sym")]
        pub fn py_label_count_for_sym(&self, _sym_index: usize) -> usize {
            0
        }
    }
}
