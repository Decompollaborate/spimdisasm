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
        before_proc::FunctionSym, processed::FunctionSymProcessed, Symbol, SymbolPostProcessError,
        SymbolProcessed,
    },
};

const SECTION_TYPE: SectionType = SectionType::Text;

#[derive(Debug, Clone)]
#[must_use]
pub struct ExecutableSectionProcessed {
    name: Arc<str>,
    ranges: RomVramRange,
    parent_segment_info: ParentSegmentInfo,
    functions: Arc<[FunctionSymProcessed]>,
    symbol_vrams: UnorderedSet<Vram>,
}

impl ExecutableSectionProcessed {
    pub(crate) fn new(
        context: &mut Context,
        name: Arc<str>,
        ranges: RomVramRange,
        parent_segment_info: ParentSegmentInfo,
        functions: Vec<FunctionSym>,
        symbol_vrams: UnorderedSet<Vram>,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<Self, SectionPostProcessError> {
        let functions = functions
            .into_iter()
            .map(|x| x.post_process(context, user_relocs))
            .collect::<Result<Arc<[FunctionSymProcessed]>, SymbolPostProcessError>>()?;

        Ok(Self {
            name,
            ranges,
            parent_segment_info,
            functions,
            symbol_vrams,
        })
    }
}

impl ExecutableSectionProcessed {
    pub fn functions(&self) -> &[FunctionSymProcessed] {
        &self.functions
    }
}

impl Section for ExecutableSectionProcessed {
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
        SECTION_TYPE
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.functions
    }

    fn symbols_vrams(&self) -> &UnorderedSet<Vram> {
        &self.symbol_vrams
    }
}
impl RomSection for ExecutableSectionProcessed {
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SectionProcessed for ExecutableSectionProcessed {
    fn symbol_list(&self) -> &[impl SymbolProcessed] {
        &self.functions
    }
}
impl RomSectionProcessed for ExecutableSectionProcessed {}

impl hash::Hash for ExecutableSectionProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for ExecutableSectionProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for ExecutableSectionProcessed {
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
