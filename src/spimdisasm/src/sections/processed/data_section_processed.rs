/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Vram},
    collections::unordered_set::UnorderedSet,
    context::Context,
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
    sections::{
        RomSection, RomSectionProcessed, Section, SectionPostProcessError, SectionProcessed,
    },
    symbols::{
        before_proc::DataSym, processed::DataSymProcessed, Symbol, SymbolPostProcessError,
        SymbolProcessed,
    },
};

#[derive(Debug, Clone)]
#[must_use]
pub struct DataSectionProcessed {
    name: Arc<str>,
    ranges: RomVramRange,
    parent_segment_info: ParentSegmentInfo,
    section_type: SectionType,
    data_symbols: Arc<[DataSymProcessed]>,
    symbol_vrams: UnorderedSet<Vram>,
}

impl DataSectionProcessed {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        context: &mut Context,
        name: Arc<str>,
        ranges: RomVramRange,
        parent_segment_info: ParentSegmentInfo,
        section_type: SectionType,
        data_symbols: Vec<DataSym>,
        symbol_vrams: UnorderedSet<Vram>,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<Self, SectionPostProcessError> {
        let data_symbols = data_symbols
            .into_iter()
            .map(|x| x.post_process(context, user_relocs))
            .collect::<Result<Arc<[DataSymProcessed]>, SymbolPostProcessError>>()?;

        Ok(Self {
            name,
            ranges,
            parent_segment_info,
            section_type,
            data_symbols,
            symbol_vrams,
        })
    }
}

impl DataSectionProcessed {
    pub fn data_symbols(&self) -> &[DataSymProcessed] {
        &self.data_symbols
    }
}

impl Section for DataSectionProcessed {
    fn name(&self) -> Arc<str> {
        self.name.clone()
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
impl RomSection for DataSectionProcessed {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SectionProcessed for DataSectionProcessed {
    fn symbol_list(&self) -> &[impl SymbolProcessed] {
        &self.data_symbols
    }
}
impl RomSectionProcessed for DataSectionProcessed {}

impl hash::Hash for DataSectionProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for DataSectionProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for DataSectionProcessed {
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
