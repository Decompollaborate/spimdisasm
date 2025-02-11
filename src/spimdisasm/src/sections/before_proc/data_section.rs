/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use core::hash;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::StringGuesserFlags,
    collections::{
        addended_ordered_map::FindSettings, unordered_map::UnorderedMap,
        unordered_set::UnorderedSet,
    },
    config::{Compiler, Endian},
    context::Context,
    metadata::{ParentSectionMetadata, SegmentMetadata, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
    sections::{
        processed::DataSectionProcessed, RomSection, RomSectionPreprocessed, Section,
        SectionCreationError, SectionPostProcessError, SectionPreprocessed,
    },
    str_decoding::Encoding,
    symbols::{
        before_proc::{data_sym::DataSymProperties, DataSym},
        Symbol, SymbolPreprocessed,
    },
};

#[derive(Debug, Clone)]
#[must_use]
pub struct DataSection {
    name: String,

    ranges: RomVramRange,

    parent_segment_info: ParentSegmentInfo,

    // in_section_offset: u32,
    section_type: SectionType,

    //
    data_symbols: Vec<DataSym>,

    symbol_vrams: UnorderedSet<Vram>,
}

impl DataSection {
    // TODO: fix
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &mut Context,
        settings: &DataSectionSettings,
        name: String,
        raw_bytes: &[u8],
        rom: Rom,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
        section_type: SectionType,
    ) -> Result<Self, SectionCreationError> {
        if raw_bytes.is_empty() {
            return Err(SectionCreationError::EmptySection { name, vram });
        }
        if (rom.inner() % 4) != (vram.inner() % 4) {
            // TODO: Does this check make sense? It would be weird if this kind of section existed, wouldn't it?
            return Err(SectionCreationError::RomVramAlignmentMismatch {
                name,
                rom,
                vram,
                multiple_of: 4,
            });
        }

        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        // Ensure there's a symbol at the beginning of the section.
        context
            .find_owned_segment_mut(&parent_segment_info)?
            .add_symbol(vram, false)?;

        let owned_segment = context.find_owned_segment(&parent_segment_info)?;

        let (symbols_info_vec, auto_pads) = Self::find_symbols(
            owned_segment,
            settings,
            raw_bytes,
            vram_range,
            section_type,
            context.global_config().endian(),
        );

        let mut data_symbols = Vec::new();
        let mut symbol_vrams = UnorderedSet::new();

        for (i, (new_sym_vram, sym_type)) in symbols_info_vec.iter().enumerate() {
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

            let properties = DataSymProperties {
                parent_metadata: ParentSectionMetadata::new(
                    name.clone(),
                    vram,
                    parent_segment_info.clone(),
                ),
                compiler: settings.compiler,
                auto_pad_by: auto_pads.get(new_sym_vram).copied(),
                detected_type: *sym_type,
                encoding: settings.encoding,
            };
            let /*mut*/ sym = DataSym::new(context, raw_bytes[start..end].into(), sym_rom, *new_sym_vram, start, parent_segment_info.clone(), section_type, properties)?;

            data_symbols.push(sym);
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

    #[allow(clippy::type_complexity)]
    fn find_symbols(
        owned_segment: &SegmentMetadata,
        settings: &DataSectionSettings,
        raw_bytes: &[u8],
        vram_range: AddressRange<Vram>,
        section_type: SectionType,
        endian: Endian,
    ) -> (Vec<(Vram, Option<SymbolType>)>, UnorderedMap<Vram, Vram>) {
        let mut symbols_info = BTreeMap::new();
        // Ensure there's a symbol at the beginning of the section.
        symbols_info.insert(vram_range.start(), None);
        let mut auto_pads = UnorderedMap::new();

        if vram_range.start().inner() % 4 != 0 || section_type == SectionType::GccExceptTable {
            // Not word-aligned, so I don't think it would make sense to look for pointers.
            // Fallback to a simpler algorithm.
            // Alternatively, if this is a except table section then avoid looking doing analisys,
            // since we know it can only contain table(s) and DataSym will make sure to produce the
            // labels.

            for reference in owned_segment.find_references_range(vram_range) {
                let reference_vram = reference.vram();
                symbols_info.insert(reference_vram, reference.sym_type());
                if let Some(size) = reference.size() {
                    let next_vram = reference_vram + size;
                    if vram_range.in_range(next_vram) {
                        symbols_info.insert(next_vram, None);
                        auto_pads.insert(next_vram, reference_vram);
                    }
                }
            }

            return (symbols_info.into_iter().collect(), auto_pads);
        }

        let mut remaining_string_size = 0;

        let mut prev_sym_info: Option<(Vram, Option<SymbolType>, Option<Size>)> = None;
        // If true: the previous symbol made us thought we may be in late_rodata
        let mut maybe_reached_late_rodata = false;
        // If true, we are sure we are in late_rodata
        let mut reached_late_rodata = false;

        let mut float_counter = 0;
        let mut float_padding_counter = 0;

        // Look for stuff that looks like addresses which point to symbols on this section
        for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
            let local_offset = i * 4;

            let current_vram = vram_range.start() + Size::new(local_offset as u32);
            let b_vram = current_vram + Size::new(1);
            let c_vram = current_vram + Size::new(2);
            let d_vram = current_vram + Size::new(3);

            let prev_sym_ended_here = if prev_sym_info
                .is_some_and(|(v, _, s)| s.is_some_and(|s| current_vram >= v + s))
            {
                // If symbol has a given size then get rid of the info as soon as we pass the end of it.
                prev_sym_info = None;
                true
            } else {
                false
            };

            if remaining_string_size <= 0 {
                let a = owned_segment.find_reference(current_vram, FindSettings::new(false));
                let b = owned_segment.find_reference(b_vram, FindSettings::new(false));
                let c = owned_segment.find_reference(c_vram, FindSettings::new(false));
                let d = owned_segment.find_reference(d_vram, FindSettings::new(false));

                if b.is_none() && c.is_none() && d.is_none() {
                    // There's no symbol in between
                    let word = endian.word_from_bytes(word_bytes);

                    let current_type = match a {
                        None => prev_sym_info.and_then(|x| x.1),
                        Some(wrapper) => wrapper.sym_type(),
                    };

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

                    let should_search_for_address =
                        current_type.is_none_or(|x| x.can_reference_symbols());

                    let word_vram = Vram::new(word);
                    let word_ref = owned_segment.find_reference(word_vram, FindSettings::new(true));

                    if should_search_for_address {
                        // TODO: improve heuristic to determine if should search for symbols
                        if !owned_segment.is_vram_ignored(word_vram)
                            && vram_range.in_range(word_vram)
                        {
                            // Vram is contained in this section
                            if word_ref.is_none_or(|x| {
                                x.vram() == word_vram || current_type.is_some_and(|t| t.is_table())
                            }) {
                                // Only count this symbol if it doesn't have an addend.
                                // If it does have an addend then it may be part of a larger symbol.
                                symbols_info.entry(word_vram).or_default();
                            }
                        }
                    }

                    if !owned_segment.is_vram_ignored(current_vram)
                        && word_ref.is_none_or(|x| match x.sym_type() {
                            Some(SymbolType::Function) => x.vram() != word_vram,
                            Some(t) => {
                                if t.is_label() {
                                    x.vram() != word_vram
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        })
                    {
                        if let Some((str_size, next_vram)) = Self::try_to_guess_c_string(
                            owned_segment,
                            current_vram,
                            settings,
                            &raw_bytes[local_offset..],
                            vram_range,
                            maybe_reached_late_rodata || reached_late_rodata,
                            prev_sym_ended_here,
                        ) {
                            remaining_string_size = str_size;

                            *symbols_info.entry(current_vram).or_default() =
                                Some(SymbolType::CString);
                            if !auto_pads.contains_key(&current_vram) {
                                auto_pads.insert(current_vram, current_vram);
                            }

                            if vram_range.in_range(next_vram)
                                && !owned_segment.is_vram_ignored(next_vram)
                            {
                                // Avoid generating a symbol at the end of the section
                                symbols_info.entry(next_vram).or_default();
                                auto_pads.insert(next_vram, current_vram);
                            }

                            // Next symbol should not be affected by this string.
                            prev_sym_info = Some((
                                current_vram,
                                Some(SymbolType::CString),
                                Some((next_vram - current_vram).try_into().unwrap()),
                            ));
                        }
                    }
                }

                if prev_sym_info
                    .is_none_or(|(v, _, size)| size.is_none_or(|s| current_vram > v + s))
                {
                    for (x_vram, x) in [(current_vram, a), (b_vram, b), (c_vram, c), (d_vram, d)] {
                        if owned_segment.is_vram_ignored(x_vram) {
                            continue;
                        }

                        if let Some(reference) = x {
                            symbols_info.entry(reference.vram()).or_default();

                            if let Some(size) = reference.user_declared_size() {
                                let next_vram = reference.vram() + size;

                                // Avoid generating a symbol at the end of the section
                                if vram_range.in_range(next_vram) {
                                    let allow_next = match reference.sym_type() {
                                        Some(SymbolType::CString) => next_vram.inner() % 4 == 0,
                                        _ => true,
                                    };
                                    if allow_next {
                                        symbols_info.entry(next_vram).or_default();
                                        auto_pads.insert(next_vram, reference.vram());
                                    }
                                }
                            }
                            prev_sym_info = Some((x_vram, reference.sym_type(), reference.size()));
                        }
                    }
                }
            }

            maybe_reached_late_rodata = false;
            if !reached_late_rodata
                && section_type == SectionType::Rodata
                && prev_sym_info
                    .is_some_and(|x| x.1.is_some_and(|x| x.is_late_rodata(settings.compiler())))
            {
                if prev_sym_info.is_some_and(|x| x.1 == Some(SymbolType::Jumptable)) {
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

        if let Some(compiler) = settings.compiler {
            if compiler.prev_align_for_type(SymbolType::CString) > Some(2) {
                for (v, padded_by) in &auto_pads {
                    if owned_segment
                        .find_reference(*v, FindSettings::new(false))
                        .is_some()
                    {
                        continue;
                    }

                    let mut range = symbols_info.range(padded_by..);

                    // Make sure this is the symbol that created this pad, and make sure it is a string
                    if range.next().is_none_or(|(padder_vram, padder_type)| {
                        padder_vram != padded_by || padder_type != &Some(SymbolType::CString)
                    }) {
                        continue;
                    }

                    if range.next().is_none_or(|x| x.0 != v) {
                        // Somehow the symbol after wasn't the pad?
                        continue;
                    }

                    let after_pad = range.next();

                    if let Some((after_pad_vram, after_pad_type)) = after_pad {
                        if after_pad_type.is_some_and(|sym_type| {
                            compiler.prev_align_for_type(sym_type).is_some_and(|x| {
                                v.inner().next_multiple_of(1 << x) == after_pad_vram.inner()
                                    && after_pad_vram.inner() % (1 << x) == 0
                            })
                        }) {
                            // check every padding byte is zero
                            let start = (*v - vram_range.start()).inner() as usize;
                            let end = (*after_pad_vram - vram_range.start()).inner() as usize;
                            if raw_bytes[start..end].iter().all(|x| *x == 0) {
                                symbols_info.remove(v);
                            }
                        }
                    } else {
                        // There are no symbols left?
                    }
                }
            }
        }

        (symbols_info.into_iter().collect(), auto_pads)
    }

    fn try_to_guess_c_string(
        owned_segment: &SegmentMetadata,
        current_vram: Vram,
        settings: &DataSectionSettings,
        sub_raw_bytes: &[u8],
        vram_range: AddressRange<Vram>,
        reached_late_rodata: bool,
        prev_sym_ended_here: bool,
    ) -> Option<(i32, Vram)> {
        let current_ref = owned_segment.find_reference(current_vram, FindSettings::new(true));

        // Avoid guessing strings in the middle of other symbols
        if current_ref.is_some_and(|x| x.vram() != current_vram) {
            return None;
        }

        let guessed_size = settings.string_guesser_flags.guess(
            current_ref,
            current_vram,
            sub_raw_bytes,
            settings.encoding,
            settings.compiler(),
            reached_late_rodata,
            prev_sym_ended_here,
        );

        match guessed_size {
            Ok(str_size) => {
                let str_sym_size = str_size.next_multiple_of(4);
                let mut in_between_range = owned_segment.find_references_range(AddressRange::new(
                    current_vram + Size::new(1),
                    current_vram + Size::new(str_sym_size as u32),
                ));

                if in_between_range.next().is_none() {
                    // Check if there is already another symbol after the current one and before the end of the string,
                    // in which case we say this symbol should not be a string

                    let next_vram = Self::next_vram_for_c_string(
                        current_vram + Size::new(str_sym_size as u32),
                        owned_segment,
                        settings,
                        &sub_raw_bytes[str_sym_size..],
                        vram_range,
                    );

                    Some((str_size as i32, next_vram))
                } else {
                    None
                }
            }

            Err(_e) => None,
        }
    }

    fn next_vram_for_c_string(
        next_vram: Vram,
        owned_segment: &SegmentMetadata,
        settings: &DataSectionSettings,
        next_raw_bytes: &[u8],
        vram_range: AddressRange<Vram>,
    ) -> Vram {
        if 4 > next_raw_bytes.len() {
            // There's no more bytes available, so just return this
            return next_vram;
        }
        if Endian::Big.word_from_bytes(next_raw_bytes) != 0 {
            return next_vram;
        }

        // Next word is zero, which means it could be padding bytes, so we have to check
        // if it may be an actual symbol by checking if anything references it

        let compiler = if let Some(compiler) = settings.compiler {
            compiler
        } else {
            // We need can only do alignment analysis if we know the current compiler
            return next_vram;
        };

        if owned_segment
            .find_reference(next_vram, FindSettings::new(false))
            .is_none_or(|x| x.reference_counter() == 0 && !x.is_user_declared())
        {
            if let Some(shift) = compiler.prev_align_for_type(SymbolType::CString) {
                let str_alignment = 1 << shift;

                if next_vram.inner() % str_alignment != 0 {
                    // Some compilers align strings to 8, leaving some annoying padding.
                    // We try to check if the next symbol is aligned, and if that's the case then grab the
                    // padding into this symbol.

                    let next_next_vram =
                        Vram::new(next_vram.inner().next_multiple_of(str_alignment));
                    if vram_range.in_range(next_next_vram) {
                        let next_next_ref = owned_segment
                            .find_references_range(AddressRange::new(
                                next_vram,
                                next_next_vram + Size::new(1),
                            ))
                            .next();

                        if next_next_ref.is_none_or(|x| {
                            x.sym_type().is_some_and(|sym_type| {
                                compiler.prev_align_for_type(sym_type) >= Some(shift)
                            })
                        }) {
                            return next_vram + Size::new(4);
                        }
                    } else if vram_range.end() == next_next_vram {
                        // trailing padding, lets just add it to this string
                        // TODO: change to next_next_vram
                        return next_vram + Size::new(4);
                    }
                }
            }

            // Look for the next known symbol
            if let Some(next_next_ref) = owned_segment
                .find_references_range(AddressRange::new(
                    next_vram + Size::new(1),
                    vram_range.end(),
                ))
                .next()
            {
                let next_next_vram = next_next_ref.vram();

                // Only eat padding in word-sized amounts
                if next_next_vram.inner() % 4 == 0 {
                    // Make sure everything inside this range is zero
                    if next_raw_bytes[..(next_next_vram - next_vram).inner() as usize]
                        .iter()
                        .all(|x| *x == 0)
                    {
                        if let Some(shift_value) = next_next_ref
                            .sym_type()
                            .and_then(|sym_type| compiler.prev_align_for_type(sym_type))
                        {
                            if next_next_vram.inner()
                                == next_vram.inner().next_multiple_of(1 << shift_value)
                            {
                                // All the data between the end of this string and the next real
                                // symbol is just padding generated by the alignment of the next symbol

                                return next_next_vram;
                            }
                        }
                    }
                }
            }
        }

        next_vram
    }
}

impl DataSection {
    pub fn data_symbols(&self) -> &[DataSym] {
        &self.data_symbols
    }
}

impl DataSection {
    pub fn post_process(
        self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<DataSectionProcessed, SectionPostProcessError> {
        DataSectionProcessed::new(
            context,
            self.name,
            self.ranges,
            self.parent_segment_info,
            self.section_type,
            self.data_symbols,
            self.symbol_vrams,
            user_relocs,
        )
    }
}

impl Section for DataSection {
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
}
impl RomSection for DataSection {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SectionPreprocessed for DataSection {
    fn symbol_list(&self) -> &[impl SymbolPreprocessed] {
        &self.data_symbols
    }
}
impl RomSectionPreprocessed for DataSection {}

impl hash::Hash for DataSection {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for DataSection {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for DataSection {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // Compare segment info first, so symbols get sorted by segment
        match self
            .parent_segment_info
            .partial_cmp(&other.parent_segment_info)
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.ranges.partial_cmp(&other.ranges)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct DataSectionSettings {
    compiler: Option<Compiler>,
    string_guesser_flags: StringGuesserFlags,
    encoding: Encoding,
}

impl DataSectionSettings {
    pub fn new(compiler: Option<Compiler>) -> Self {
        Self {
            compiler,
            string_guesser_flags: StringGuesserFlags::default(),
            encoding: Encoding::default(),
        }
    }

    pub fn compiler(&self) -> Option<Compiler> {
        self.compiler
    }

    pub fn string_guesser_flags(&self) -> StringGuesserFlags {
        self.string_guesser_flags
    }
    pub fn set_string_guesser_flags(&mut self, string_guesser_flags: StringGuesserFlags) {
        self.string_guesser_flags = string_guesser_flags;
    }
    pub fn with_string_guesser_flags(self, string_guesser_flags: StringGuesserFlags) -> Self {
        Self {
            string_guesser_flags,
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
    use super::*;

    #[pymethods]
    impl DataSectionSettings {
        #[new]
        #[pyo3(signature = (compiler))]
        pub fn py_new(compiler: Option<Compiler>) -> Self {
            Self::new(compiler)
        }

        #[pyo3(name = "set_string_guesser_flags")]
        pub fn py_set_string_guesser_flags(&mut self, string_guesser_flags: StringGuesserFlags) {
            self.set_string_guesser_flags(string_guesser_flags)
        }

        #[pyo3(name = "set_encoding")]
        pub fn py_set_encoding(&mut self, encoding: Encoding) {
            self.set_encoding(encoding);
        }
    }
}
