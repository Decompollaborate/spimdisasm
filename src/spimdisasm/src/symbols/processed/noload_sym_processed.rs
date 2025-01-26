/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::hash;

use crate::{
    addresses::{AddressRange, Vram},
    context::Context,
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    symbols::{
        display::{
            InternalSymDisplSettings, SymDisplayError, SymNoloadDisplay, SymNoloadDisplaySettings,
        },
        Symbol, SymbolPostProcessError, SymbolProcessed,
    },
};

const SECTION_TYPE: SectionType = SectionType::Bss;

#[derive(Debug, Clone)]
pub struct NoloadSymProcessed {
    vram_range: AddressRange<Vram>,
    parent_segment_info: ParentSegmentInfo,
}

impl NoloadSymProcessed {
    pub(crate) fn new(
        _context: &mut Context,
        vram_range: AddressRange<Vram>,
        parent_segment_info: ParentSegmentInfo,
    ) -> Result<Self, SymbolPostProcessError> {
        Ok(Self {
            vram_range,
            parent_segment_info,
        })
    }
}

impl<'ctx, 'sym, 'flg> NoloadSymProcessed {
    pub fn display(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg SymNoloadDisplaySettings,
    ) -> Result<SymNoloadDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        self.display_internal(context, settings, InternalSymDisplSettings::new(false))
    }

    pub(crate) fn display_internal(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg SymNoloadDisplaySettings,
        internal_settings: InternalSymDisplSettings,
    ) -> Result<SymNoloadDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        SymNoloadDisplay::new(context, self, settings, internal_settings)
    }
}

impl Symbol for NoloadSymProcessed {
    fn vram_range(&self) -> &AddressRange<Vram> {
        &self.vram_range
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    fn section_type(&self) -> SectionType {
        SECTION_TYPE
    }
}
impl SymbolProcessed for NoloadSymProcessed {}

impl hash::Hash for NoloadSymProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.vram_range.hash(state);
    }
}
impl PartialEq for NoloadSymProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.vram_range == other.vram_range
    }
}
impl PartialOrd for NoloadSymProcessed {
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
