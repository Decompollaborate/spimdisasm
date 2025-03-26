/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    collections::addended_ordered_map::FindSettings,
    context::Context,
    metadata::{LabelType, ReferrerInfo, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    relocation::{RelocReferencedSym, RelocationInfo, RelocationType},
    section_type::SectionType,
    str_decoding::Encoding,
    symbols::{
        display::{
            InternalSymDisplSettings, SymDataDisplay, SymDataDisplaySettings, SymDisplayError,
        },
        InvalidRelocForSectionError, OwnedSymbolNotFoundError, RomSymbol, RomSymbolProcessed,
        Symbol, SymbolPostProcessError, SymbolProcessed, UnalignedUserRelocError,
    },
};

#[derive(Debug, Clone)]
pub struct DataSymProcessed {
    ranges: RomVramRange,
    raw_bytes: Arc<[u8]>,
    parent_segment_info: ParentSegmentInfo,
    section_type: SectionType,
    encoding: Encoding,

    relocs: Arc<[Option<RelocationInfo>]>,
}

impl DataSymProcessed {
    pub(crate) fn new(
        context: &mut Context,
        ranges: RomVramRange,
        raw_bytes: Arc<[u8]>,
        parent_segment_info: ParentSegmentInfo,
        section_type: SectionType,
        encoding: Encoding,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<Self, SymbolPostProcessError> {
        let mut relocs = Self::generate_relocs(context, &ranges, &raw_bytes, &parent_segment_info)?;

        if !relocs.is_empty() {
            for (reloc_rom, reloc_info) in user_relocs.range(*ranges.rom()) {
                if reloc_rom.inner() % 4 != 0 {
                    return Err(SymbolPostProcessError::UnalignedUserReloc(
                        UnalignedUserRelocError::new(*reloc_rom, reloc_info.reloc_type()),
                    ));
                }

                if !reloc_info.reloc_type().valid_for_data_sym() {
                    return Err(SymbolPostProcessError::InvalidRelocForSection(
                        InvalidRelocForSectionError::new(
                            *reloc_rom,
                            reloc_info.reloc_type(),
                            section_type,
                        ),
                    ));
                }

                let reloc_index = (*reloc_rom - ranges.rom().start()).inner() as usize / 4;
                assert!(reloc_index < relocs.len());
                relocs[reloc_index] = if reloc_info.reloc_type().is_none() {
                    None
                } else {
                    Some(reloc_info.clone())
                };
            }
        }

        Self::update_referenced_symbols(context, &ranges, &raw_bytes, &parent_segment_info)?;

        Ok({
            Self {
                ranges,
                raw_bytes,
                parent_segment_info,
                section_type,
                encoding,
                relocs: relocs.into(),
            }
        })
    }

    fn generate_relocs(
        context: &mut Context,
        ranges: &RomVramRange,
        raw_bytes: &[u8],
        parent_segment_info: &ParentSegmentInfo,
    ) -> Result<Vec<Option<RelocationInfo>>, SymbolPostProcessError> {
        if ranges.rom().start().inner() % 4 != 0 {
            return Ok(Vec::new());
        }

        let mut relocs = vec![None; raw_bytes.len() / 4];

        let self_vram = ranges.vram().start();

        let mut referenced_labels_owned_segment = Vec::new();
        let mut referenced_labels_refer_segment = Vec::new();

        let owned_segment = context.find_owned_segment(parent_segment_info)?;
        let metadata = owned_segment
            .find_symbol(ranges.vram().start(), FindSettings::new(false))
            .map_or_else(|| Err(OwnedSymbolNotFoundError::new()), Ok)?;
        let global_config = context.global_config();
        let endian = global_config.endian();
        let gp_value = global_config.gp_config().map(|x| x.gp_value());

        let sym_type = metadata.sym_type();
        let add_gp_to_pointed_data = metadata.add_gp_to_pointed_data();

        let should_search_for_address = sym_type.is_none_or(|x| x.can_reference_symbols());
        let is_table = sym_type.is_some_and(|x| x.is_table());
        let find_settings = FindSettings::new(!is_table && metadata.allow_ref_with_addend());

        if should_search_for_address {
            let reloc_type = if add_gp_to_pointed_data {
                RelocationType::R_MIPS_GPREL32
            } else {
                RelocationType::R_MIPS_32
            };

            for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
                let current_rom = ranges.rom().start() + Size::new(i as u32 * 4);
                let word = endian.word_from_bytes(word_bytes);
                let word_vram = if let (true, Some(gp_value)) = (add_gp_to_pointed_data, gp_value) {
                    // `as i32` should be doing a two complement conversion.
                    Vram::new(gp_value.inner().wrapping_add_signed(word as i32))
                } else {
                    Vram::new(word)
                };

                if owned_segment.is_vram_ignored(word_vram) {
                    continue;
                }

                if is_table {
                    let (label, is_owned_segment) = if owned_segment.in_vram_range(word_vram) {
                        (owned_segment.find_label(word_vram), true)
                    } else {
                        (
                            context.find_label_from_any_segment(
                                word_vram,
                                parent_segment_info,
                                |_| true,
                            ),
                            false,
                        )
                    };
                    let valid_reference = label.is_some_and(|other_metadata| {
                        other_metadata.label_type() != LabelType::Branch
                    });

                    if valid_reference {
                        relocs[i] =
                            Some(reloc_type.new_reloc_info(RelocReferencedSym::Label(word_vram)));

                        if is_owned_segment {
                            &mut referenced_labels_owned_segment
                        } else {
                            &mut referenced_labels_refer_segment
                        }
                        .push((
                            word_vram,
                            ReferrerInfo::new_data(
                                self_vram,
                                parent_segment_info.clone(),
                                current_rom,
                            ),
                        ));
                    }
                } else {
                    let valid_reference = if owned_segment.in_vram_range(word_vram) {
                        owned_segment.find_symbol(word_vram, find_settings)
                    } else {
                        context.find_symbol_from_any_segment(
                            word_vram,
                            parent_segment_info,
                            find_settings,
                            |other_metadata| {
                                if other_metadata.sym_type() == Some(SymbolType::Function) {
                                    // Avoid referencing addends of functions
                                    other_metadata.vram() == word_vram
                                } else {
                                    true
                                }
                            },
                        )
                    }
                    .is_some_and(|other_metadata| {
                        other_metadata.vram() == word_vram
                            || other_metadata
                                .sym_type()
                                .is_none_or(|sym_typ| sym_typ.may_have_addend())
                    });

                    if valid_reference {
                        relocs[i] =
                            Some(reloc_type.new_reloc_info(RelocReferencedSym::Address(word_vram)));
                    }
                }
            }
        }

        // Tell labels they have been referenced
        let owned_segment_mut = context.find_owned_segment_mut(parent_segment_info)?;
        for (label_vram, referrer) in referenced_labels_owned_segment {
            if let Some(label) = owned_segment_mut.find_label_mut(label_vram) {
                label.add_referenced_info(referrer);
            }
        }

        for (label_vram, referrer) in referenced_labels_refer_segment {
            let referenced_segment_mut =
                context.find_referenced_segment_mut(label_vram, parent_segment_info);
            if let Some(label) = referenced_segment_mut.find_label_mut(label_vram) {
                label.add_referenced_info(referrer);
            }
        }

        Ok(relocs)
    }

    fn update_referenced_symbols(
        context: &mut Context,
        ranges: &RomVramRange,
        raw_bytes: &[u8],
        parent_segment_info: &ParentSegmentInfo,
    ) -> Result<(), SymbolPostProcessError> {
        let rom = ranges.rom().start();
        let vram = ranges.vram().start();
        let endian = context.global_config().endian();

        let owned_segment = context.find_owned_segment_mut(parent_segment_info)?;
        let metadata = owned_segment.find_symbol(vram, FindSettings::new(false));

        let should_search_for_address =
            metadata.is_some_and(|x| x.sym_type().is_none_or(|x| x.can_reference_symbols()));

        if rom.inner() % 4 == 0 && should_search_for_address {
            for (i, word_bytes) in raw_bytes.chunks_exact(4).enumerate() {
                let word = endian.word_from_bytes(word_bytes);
                let word_vram = Vram::new(word);
                let offset = Size::new(i as u32);

                if owned_segment.in_vram_range(word_vram) {
                    if let Some(sym_metadata) =
                        owned_segment.find_symbol_mut(word_vram, FindSettings::new(true))
                    {
                        sym_metadata.add_reference_symbol(
                            vram,
                            parent_segment_info.clone(),
                            rom + offset,
                        );
                    }
                } else {
                    // TODO
                }
            }
        }

        Ok(())
    }
}

impl DataSymProcessed {
    pub(crate) fn raw_bytes(&self) -> &[u8] {
        &self.raw_bytes
    }

    pub(crate) fn encoding(&self) -> Encoding {
        self.encoding
    }
}

impl<'ctx, 'sym, 'flg> DataSymProcessed {
    pub fn display(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg SymDataDisplaySettings,
    ) -> Result<SymDataDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        self.display_internal(context, settings, InternalSymDisplSettings::new(false))
    }

    pub(crate) fn display_internal(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg SymDataDisplaySettings,
        internal_settings: InternalSymDisplSettings,
    ) -> Result<SymDataDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        SymDataDisplay::new(context, self, settings, internal_settings)
    }
}

impl Symbol for DataSymProcessed {
    fn vram_range(&self) -> &AddressRange<Vram> {
        self.ranges.vram()
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    fn section_type(&self) -> SectionType {
        self.section_type
    }
}
impl RomSymbol for DataSymProcessed {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SymbolProcessed for DataSymProcessed {}
impl RomSymbolProcessed for DataSymProcessed {
    fn relocs(&self) -> &[Option<RelocationInfo>] {
        &self.relocs
    }
}

impl hash::Hash for DataSymProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for DataSymProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for DataSymProcessed {
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
