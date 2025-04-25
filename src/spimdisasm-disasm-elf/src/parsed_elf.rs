/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{
    collections::{btree_map, BTreeMap, HashMap, HashSet},
    sync::Arc,
};

use object::{read::elf::ElfFile32, Object};
use spimdisasm::{
    addresses::{GlobalOffsetTable, GotGlobalEntry, GotLocalEntry, GpValue, Rom, Size, Vram},
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
    config::Endian,
};

use crate::{
    dynamic_section::DynamicSection,
    elf_section_type::{ElfSectionType, ProgbitsType, RawElfSection},
    elf_symbol::{ElfSymSectionIndex, ElfSymType, ElfSymbol},
    mips_reginfo::MipsReginfo,
    utils,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedElf {
    kind: object::ObjectKind,
    endian: Endian,

    executable_sections: Vec<ElfExecutableSection>,
    data_sections: Vec<ElfDataSection>,
    nobits_sections: Vec<ElfNobitsSection>,

    symbols: HashMap<u32, SymbolsPerValue>,
    labels: HashMap<u32, SymbolsPerValue>,

    mips_reginfo: Option<MipsReginfo>,

    got: Option<GlobalOffsetTable>,
    dynamic: Option<DynamicSection>,
    got_global_symbols: Vec<GotGlobalSymbol>,
}

impl ParsedElf {
    pub fn parse_elf(elf_file: ElfFile32) -> Self {
        let kind = elf_file.kind();
        let elf_endian = elf_file.endian();
        let endian = utils::endian_to_endian(elf_endian);

        let mut executable_sections = Vec::new();
        let mut data_sections = Vec::new();
        let mut nobits_sections = Vec::new();

        let mut raw_got_info = None;

        let mut mips_reginfo = None;
        let mut dynamic = None;

        for elf_section in elf_file.sections() {
            if let Some(raw_elf_section) = RawElfSection::new(elf_section) {
                match raw_elf_section.section_type() {
                    ElfSectionType::Progbits(progbits_type) => match progbits_type {
                        ProgbitsType::Text => {
                            executable_sections.push(ElfExecutableSection::new(raw_elf_section));
                        }
                        ProgbitsType::Data | ProgbitsType::Rodata => {
                            data_sections.push(ElfDataSection::new(raw_elf_section));
                        }
                        ProgbitsType::Got => {
                            utils::set_valid_option_on_none(
                                &mut raw_got_info,
                                Some(gather_raw_got(raw_elf_section)),
                            );
                        }
                        ProgbitsType::Unknown => {
                            eprintln!("Unknown progbits: {}", raw_elf_section.name())
                        }
                    },
                    ElfSectionType::Nobits => {
                        nobits_sections.push(ElfNobitsSection::new(raw_elf_section));
                    }
                    ElfSectionType::Reloc => {
                        // TODO
                    }
                    ElfSectionType::Dynamic => {
                        utils::set_valid_option_on_none(
                            &mut dynamic,
                            DynamicSection::parse_from_raw_section(raw_elf_section),
                        );
                    }
                    ElfSectionType::MipsReginfo => {
                        utils::set_valid_option_on_none(
                            &mut mips_reginfo,
                            MipsReginfo::parse_from_raw_section(raw_elf_section),
                        );
                    }
                }
            }
        }

        let mut symbols_map = BTreeMap::new();
        let (got, got_global_symbols) =
            parse_dynsym(&mut symbols_map, &elf_file, raw_got_info, dynamic.as_ref());
        parse_symtab(&mut symbols_map, &elf_file);

        let mut seen_symbols = AddendedOrderedMap::new();

        let mut symbols = HashMap::new();
        let mut labels = HashMap::new();
        for (value, syms) in symbols_map {
            let vram = Vram::new(value);
            let syms = syms.build();
            let size = syms.syms.iter().find_map(|x| {
                // Sizes of undefined symbols can get kinda "funny".
                if !matches!(x.section_index(), ElfSymSectionIndex::Undef) {
                    x.size()
                } else {
                    None
                }
            });

            let (_, newly_created) =
                seen_symbols.find_mut_or_insert_with(vram, FindSettings::new(true), || {
                    size.unwrap_or(Size::new(1))
                });

            if newly_created {
                symbols.insert(value, syms);
            } else {
                labels.insert(value, syms);
            }
        }

        Self {
            kind,
            endian,
            executable_sections,
            data_sections,
            nobits_sections,
            symbols,
            labels,
            mips_reginfo,
            got,
            dynamic,
            got_global_symbols,
        }
    }

    #[must_use]
    pub fn gp_value(&self) -> Option<GpValue> {
        if let Some(ri_gp_value) = self.mips_reginfo.and_then(|x| x.ri_gp_value()) {
            Some(ri_gp_value)
        } else {
            self.dynamic.map(|x| x.canonical_gp())
        }
    }

    #[must_use]
    pub fn endian(&self) -> Endian {
        self.endian
    }
    #[must_use]
    pub fn executable_sections(&self) -> &[ElfExecutableSection] {
        &self.executable_sections
    }
    #[must_use]
    pub fn data_sections(&self) -> &[ElfDataSection] {
        &self.data_sections
    }
    #[must_use]
    pub fn nobits_sections(&self) -> &[ElfNobitsSection] {
        &self.nobits_sections
    }
    #[must_use]
    pub fn symbols(&self) -> &HashMap<u32, SymbolsPerValue> {
        &self.symbols
    }
    #[must_use]
    pub fn labels(&self) -> &HashMap<u32, SymbolsPerValue> {
        &self.labels
    }
    #[must_use]
    pub fn got(&self) -> Option<&GlobalOffsetTable> {
        self.got.as_ref()
    }
    #[must_use]
    pub fn got_global_symbols(&self) -> &[GotGlobalSymbol] {
        &self.got_global_symbols
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElfExecutableSection {
    name: Arc<str>,
    address: Vram,
    offset: Rom,
    size: Size,
    data: Vec<u8>,
}

impl ElfExecutableSection {
    fn new(raw_elf_section: RawElfSection) -> Self {
        assert_eq!(
            ElfSectionType::Progbits(ProgbitsType::Text),
            raw_elf_section.section_type()
        );
        let name = raw_elf_section.name();
        let address = raw_elf_section.address();
        let offset = raw_elf_section.offset();
        let size = raw_elf_section.size();
        let data = raw_elf_section.data().into();

        Self {
            name,
            address,
            offset,
            size,
            data,
        }
    }

    #[must_use]
    pub fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    #[must_use]
    pub fn address(&self) -> Vram {
        self.address
    }
    #[must_use]
    pub fn offset(&self) -> Rom {
        self.offset
    }
    #[must_use]
    pub fn size(&self) -> Size {
        self.size
    }
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElfDataSection {
    progbits_type: ProgbitsType,
    name: Arc<str>,
    address: Vram,
    offset: Rom,
    size: Size,
    data: Vec<u8>,
}

impl ElfDataSection {
    fn new(raw_elf_section: RawElfSection) -> Self {
        let progbits_type =
            if let ElfSectionType::Progbits(progbits_type) = raw_elf_section.section_type() {
                progbits_type
            } else {
                panic!()
            };
        let name = raw_elf_section.name();
        let address = raw_elf_section.address();
        let offset = raw_elf_section.offset();
        let size = raw_elf_section.size();
        let data = raw_elf_section.data().into();

        Self {
            progbits_type,
            name,
            address,
            offset,
            size,
            data,
        }
    }

    #[must_use]
    pub fn progbits_type(&self) -> ProgbitsType {
        self.progbits_type
    }
    #[must_use]
    pub fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    #[must_use]
    pub fn address(&self) -> Vram {
        self.address
    }
    #[must_use]
    pub fn offset(&self) -> Rom {
        self.offset
    }
    #[must_use]
    pub fn size(&self) -> Size {
        self.size
    }
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElfNobitsSection {
    name: Arc<str>,
    address: Vram,
    offset: Rom,
    size: Size,
}

impl ElfNobitsSection {
    fn new(raw_elf_section: RawElfSection) -> Self {
        assert_eq!(ElfSectionType::Nobits, raw_elf_section.section_type());

        let name = raw_elf_section.name();
        let address = raw_elf_section.address();
        let offset = raw_elf_section.offset();
        let size = raw_elf_section.size();

        Self {
            name,
            address,
            offset,
            size,
        }
    }

    #[must_use]
    pub fn name(&self) -> Arc<str> {
        self.name.clone()
    }
    #[must_use]
    pub fn address(&self) -> Vram {
        self.address
    }
    #[must_use]
    pub fn offset(&self) -> Rom {
        self.offset
    }
    #[must_use]
    pub fn size(&self) -> Size {
        self.size
    }
}

fn gather_raw_got(raw_elf_section: RawElfSection) -> (Vram, Vec<u32>) {
    assert_eq!(
        ElfSectionType::Progbits(ProgbitsType::Got),
        raw_elf_section.section_type()
    );

    let vram = raw_elf_section.address();
    let data = raw_elf_section.data();
    let endian = raw_elf_section.endian();

    let raw = data
        .chunks_exact(4)
        .map(|w| endian.word_from_bytes(w))
        .collect();

    (vram, raw)
}

fn parse_dynsym(
    symbols: &mut BTreeMap<u32, SymbolsPerValueBuilder>,
    elf_file: &ElfFile32,
    raw_got_info: Option<(Vram, Vec<u32>)>,
    dynamic: Option<&DynamicSection>,
) -> (Option<GlobalOffsetTable>, Vec<GotGlobalSymbol>) {
    let elf_endian = elf_file.endian();

    let dynsym = elf_file.elf_dynamic_symbol_table();
    let dynstr = dynsym.strings();

    let (got_vram, raw_got) = if let Some((got_vram, raw_got)) = raw_got_info {
        (Some(got_vram), Some(raw_got))
    } else {
        (None, None)
    };

    let (local_gotno, gotsym) = if let Some(dynamic) = dynamic {
        (dynamic.local_gotno() as usize, dynamic.gotsym() as usize)
    } else {
        // An elf file may not have a dynamic section, so instead we assume the whole table is composed of locals.
        (raw_got.as_ref().map(|x| x.len()).unwrap_or(0), usize::MAX)
    };

    let mut raw_got = raw_got.into_iter().flatten();

    // Consume the first `local_gotno` elements of `raw_got`
    let locals = raw_got
        .by_ref()
        .take(local_gotno)
        .map(GotLocalEntry::new)
        .collect();

    let mut globals = Vec::new();
    let mut got_global_symbols = Vec::new();

    for (i, sym) in dynsym.enumerate() {
        let mut elf_sym = ElfSymbol::new(sym, elf_endian, dynstr);

        if i.0 >= gotsym {
            if let Some(initial) = raw_got.next() {
                let undef_com_or_abs = matches!(
                    elf_sym.section_index(),
                    ElfSymSectionIndex::Undef
                        | ElfSymSectionIndex::Common
                        | ElfSymSectionIndex::Abs
                );

                let got_global_entry =
                    GotGlobalEntry::new(initial, elf_sym.value(), undef_com_or_abs, elf_sym.name());

                elf_sym.set_is_got_global();
                let got_global_symbol =
                    GotGlobalSymbol::new(elf_sym.clone(), got_global_entry.clone());
                got_global_symbols.push(got_global_symbol);

                globals.push(got_global_entry);
            }
        }

        // We only want **symbols** here.
        // TODO: Can we use the other stuff somehow?
        match elf_sym.typ() {
            ElfSymType::NoType => {}
            ElfSymType::Object => {}
            ElfSymType::Function => {}
            ElfSymType::Section
            | ElfSymType::File
            | ElfSymType::Common
            | ElfSymType::Tls
            | ElfSymType::Unknown(_) => continue,
        }

        match symbols.entry(elf_sym.value()) {
            btree_map::Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().insert(elf_sym);
            }
            btree_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(SymbolsPerValueBuilder::new(elf_sym));
            }
        }
    }

    let global_offset_table = got_vram.map(|vram| GlobalOffsetTable::new(vram, locals, globals));

    (global_offset_table, got_global_symbols)
}

fn parse_symtab(_symbols: &mut BTreeMap<u32, SymbolsPerValueBuilder>, _elf_file: &ElfFile32) {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolsPerValue {
    value: u32,
    syms: Vec<ElfSymbol>,
}

impl SymbolsPerValue {
    #[must_use]
    pub fn syms(&self) -> &Vec<ElfSymbol> {
        &self.syms
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolsPerValueBuilder {
    value: u32,
    syms: HashSet<ElfSymbol>,
}

impl SymbolsPerValueBuilder {
    pub fn new(elf_sym: ElfSymbol) -> Self {
        let value = elf_sym.value();
        let mut syms = HashSet::new();
        syms.insert(elf_sym);

        Self { value, syms }
    }

    pub fn insert(&mut self, elf_sym: ElfSymbol) {
        assert_eq!(self.value, elf_sym.value());
        self.syms.insert(elf_sym);
    }

    pub fn build(self) -> SymbolsPerValue {
        let mut syms: Vec<ElfSymbol> = self.syms.into_iter().collect();

        syms.sort();

        SymbolsPerValue {
            value: self.value,
            syms,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GotGlobalSymbol {
    elf_sym: ElfSymbol,
    got_entry: GotGlobalEntry,
}

impl GotGlobalSymbol {
    fn new(elf_sym: ElfSymbol, got_entry: GotGlobalEntry) -> Self {
        Self { elf_sym, got_entry }
    }

    #[must_use]
    pub fn elf_sym(&self) -> &ElfSymbol {
        &self.elf_sym
    }
    #[must_use]
    pub fn got_entry(&self) -> &GotGlobalEntry {
        &self.got_entry
    }
}
