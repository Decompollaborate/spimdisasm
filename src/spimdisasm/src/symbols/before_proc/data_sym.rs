/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use core::hash;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    config::{Compiler, Endian},
    context::Context,
    metadata::{GeneratedBy, ParentSectionMetadata, ReferrerInfo, SymbolMetadata, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
    str_decoding::Encoding,
    symbols::{processed::DataSymProcessed, RomSymbolPreprocessed, SymbolPreprocessed},
};

use crate::symbols::{
    trait_symbol::RomSymbol, Symbol, SymbolCreationError, SymbolPostProcessError,
};

#[derive(Debug, Clone)]
pub struct DataSym {
    ranges: RomVramRange,
    raw_bytes: Arc<[u8]>,
    parent_segment_info: ParentSegmentInfo,
    section_type: SectionType,
    encoding: Encoding,
}

impl DataSym {
    // TODO
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &mut Context,
        raw_bytes: Arc<[u8]>,
        rom: Rom,
        vram: Vram,
        _in_section_offset: usize,
        parent_segment_info: ParentSegmentInfo,
        section_type: SectionType,
        properties: DataSymProperties,
    ) -> Result<Self, SymbolCreationError> {
        let size = Size::new(raw_bytes.len() as u32);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let global_config = context.global_config();
        let endian = global_config.endian();
        let gp_value = global_config.gp_config().map(|x| x.gp_value());
        let encoding = properties.encoding;

        let symbol_name_generation_settings =
            global_config.symbol_name_generation_settings().clone();
        let owned_segment = context.find_owned_segment_mut(&parent_segment_info)?;

        let sym_type = if section_type == SectionType::GccExceptTable {
            //Some(SymbolType::GccExceptTable)
            None
        } else {
            None
        };

        let metadata = owned_segment.add_self_symbol(
            vram,
            Some(rom),
            size,
            section_type,
            sym_type,
            |metadata| {
                count_padding(
                    &raw_bytes,
                    metadata.user_declared_size(),
                    metadata.sym_type(),
                    endian,
                    rom,
                )
            },
            symbol_name_generation_settings,
        )?;

        properties.apply_to_metadata(metadata);
        let add_gp_to_pointed_data = metadata.add_gp_to_pointed_data();

        let sym_type = metadata.sym_type();

        let should_search_for_address = sym_type.is_none_or(|x| x.can_reference_symbols());
        let table_label = SymbolType::label_for_table(sym_type);

        // TODO: improve heuristic to determine if should search for symbols
        if rom.inner() % 4 == 0 && should_search_for_address {
            for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
                let word = endian.word_from_bytes(word_bytes);
                if word == 0 {
                    continue;
                }
                let word_vram = if let (true, Some(gp_value)) = (add_gp_to_pointed_data, gp_value) {
                    // `as i32` should be doing a two complement conversion.
                    Vram::new(gp_value.inner().wrapping_add_signed(word as i32))
                } else {
                    Vram::new(word)
                };
                let offset = Size::new(i as u32);

                if !owned_segment.is_vram_ignored(word_vram)
                    && owned_segment.in_vram_range(word_vram)
                {
                    if let Some(label) = table_label {
                        let referenced_info = ReferrerInfo::new_data(
                            ranges.vram().start(),
                            parent_segment_info.clone(),
                            ranges.rom().start() + offset,
                        );
                        owned_segment.add_label(word_vram, label, referenced_info)?;
                    }
                } else {
                    // TODO
                }
            }
        }

        Ok(Self {
            ranges,
            raw_bytes,
            parent_segment_info,
            section_type,
            encoding,
        })
    }
}

impl DataSym {
    pub(crate) fn post_process(
        self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<DataSymProcessed, SymbolPostProcessError> {
        DataSymProcessed::new(
            context,
            self.ranges,
            self.raw_bytes,
            self.parent_segment_info,
            self.section_type,
            self.encoding,
            user_relocs,
        )
    }
}

impl Symbol for DataSym {
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
}

impl RomSymbol for DataSym {
    #[must_use]
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}

impl SymbolPreprocessed for DataSym {}
impl RomSymbolPreprocessed for DataSym {}

impl hash::Hash for DataSym {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for DataSym {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for DataSym {
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

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) struct DataSymProperties {
    pub parent_metadata: ParentSectionMetadata,
    pub compiler: Option<Compiler>,
    pub auto_pad_by: Option<Vram>,
    pub detected_type: Option<SymbolType>,
    pub encoding: Encoding,
}

impl DataSymProperties {
    fn apply_to_metadata(self, metadata: &mut SymbolMetadata) {
        metadata.set_parent_metadata(self.parent_metadata);

        if let Some(compiler) = self.compiler {
            metadata.set_compiler(compiler);
        }

        if let Some(auto_pad_by) = self.auto_pad_by {
            metadata.set_auto_created_pad_by(auto_pad_by);
        }

        if let Some(detected_type) = self.detected_type {
            metadata.set_type(detected_type, GeneratedBy::Autogenerated);
        }
    }
}

fn count_padding(
    raw_bytes: &[u8],
    user_declared_size: Option<Size>,
    typ: Option<SymbolType>,
    endian: Endian,
    rom: Rom,
) -> Size {
    if user_declared_size.is_some() {
        return Size::new(0);
    }

    let mut count: u32 = 0;

    match typ {
        Some(SymbolType::UserCustom) => {}
        Some(SymbolType::CString) => {
            for byte in raw_bytes.iter().rev() {
                if *byte != 0 {
                    break;
                }
                count += 1;
            }
            count = count.saturating_sub(1);
        }
        Some(SymbolType::Float64 | SymbolType::DWord) => {
            if raw_bytes.len() > 8 {
                for byte_group in raw_bytes[8..].chunks_exact(8).rev() {
                    let dword = endian.dword_from_bytes(byte_group);
                    if dword != 0 {
                        break;
                    }
                    count += 8;
                }
            }
        }
        Some(
            SymbolType::Float32
            | SymbolType::Word
            | SymbolType::Jumptable
            | SymbolType::GccExceptTable,
        ) => {
            if raw_bytes.len() > 4 {
                for byte_group in raw_bytes[4..].chunks_exact(4).rev() {
                    let word = endian.word_from_bytes(byte_group);
                    if word != 0 {
                        break;
                    }
                    count += 4;
                }
            }
        }
        // TODO: Should count padding for those bytes and shorts? And how?
        Some(SymbolType::Byte) => {}
        Some(SymbolType::Short) => {}
        Some(SymbolType::Function) => {}
        Some(SymbolType::VirtualTable) => {}
        None => {
            // Treat it as word-sized if the alignement and size allow it.
            if raw_bytes.len() > 4 && raw_bytes.len() % 4 == 0 && rom.inner() % 4 == 0 {
                for byte_group in raw_bytes[4..].chunks_exact(4).rev() {
                    let word = endian.word_from_bytes(byte_group);
                    if word != 0 {
                        break;
                    }
                    count += 4;
                }
            }
        }
    }

    Size::new(count)
}
