/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;

use crate::{
    addresses::{AddressRange, Vram},
    collections::unordered_set::UnorderedSet,
    context::Context,
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    sections::{Section, SectionPostProcessError, SectionProcessed},
    symbols::{
        before_proc::NobitsSym, processed::NobitsSymProcessed, Symbol, SymbolPostProcessError,
        SymbolPreprocessed, SymbolProcessed,
    },
};

const SECTION_TYPE: SectionType = SectionType::Bss;

#[derive(Debug, Clone)]
#[must_use]
pub struct NobitsSectionProcessed {
    name: Arc<str>,
    vram_range: AddressRange<Vram>,
    parent_segment_info: ParentSegmentInfo,
    nobits_symbols: Arc<[NobitsSymProcessed]>,
    symbol_vrams: UnorderedSet<Vram>,
}

impl NobitsSectionProcessed {
    pub(crate) fn new(
        context: &mut Context,
        name: Arc<str>,
        vram_range: AddressRange<Vram>,
        parent_segment_info: ParentSegmentInfo,
        nobits_symbols: Vec<NobitsSym>,
        symbol_vrams: UnorderedSet<Vram>,
    ) -> Result<Self, SectionPostProcessError> {
        let user_relocs = BTreeMap::new();
        let nobits_symbols = nobits_symbols
            .into_iter()
            .map(|x| x.post_process(context, &user_relocs))
            .collect::<Result<Arc<[NobitsSymProcessed]>, SymbolPostProcessError>>()?;

        Ok(Self {
            name,
            vram_range,
            parent_segment_info,
            nobits_symbols,
            symbol_vrams,
        })
    }
}

impl NobitsSectionProcessed {
    pub fn nobits_symbols(&self) -> &[NobitsSymProcessed] {
        &self.nobits_symbols
    }
}

impl Section for NobitsSectionProcessed {
    fn name(&self) -> Arc<str> {
        self.name.clone()
    }

    fn vram_range(&self) -> &AddressRange<Vram> {
        &self.vram_range
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    #[must_use]
    fn section_type(&self) -> SectionType {
        SECTION_TYPE
    }

    fn symbol_list(&self) -> &[impl Symbol] {
        &self.nobits_symbols
    }

    fn symbols_vrams(&self) -> &UnorderedSet<Vram> {
        &self.symbol_vrams
    }
}
impl SectionProcessed for NobitsSectionProcessed {
    fn symbol_list(&self) -> &[impl SymbolProcessed] {
        &self.nobits_symbols
    }
}

impl hash::Hash for NobitsSectionProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.vram_range.hash(state);
    }
}
impl PartialEq for NobitsSectionProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.vram_range == other.vram_range
    }
}
impl PartialOrd for NobitsSectionProcessed {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // Compare segment info first, so symbols get sorted by segment
        match self
            .parent_segment_info
            .partial_cmp(&other.parent_segment_info)
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.vram_range.partial_cmp(&other.vram_range)
    }
}
