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
            InternalSymDisplSettings, SymDisplayError, SymNobitsDisplay, SymNobitsDisplaySettings,
        },
        Symbol, SymbolPostProcessError, SymbolProcessed,
    },
};

const SECTION_TYPE: SectionType = SectionType::Bss;

#[derive(Debug, Clone)]
pub struct NobitsSymProcessed {
    vram_range: AddressRange<Vram>,
    parent_segment_info: ParentSegmentInfo,
}

impl NobitsSymProcessed {
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

impl<'ctx, 'sym, 'flg> NobitsSymProcessed {
    pub fn display(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg SymNobitsDisplaySettings,
    ) -> Result<SymNobitsDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        self.display_internal(context, settings, InternalSymDisplSettings::new(false))
    }

    pub(crate) fn display_internal(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg SymNobitsDisplaySettings,
        internal_settings: InternalSymDisplSettings,
    ) -> Result<SymNobitsDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        SymNobitsDisplay::new(context, self, settings, internal_settings)
    }
}

impl Symbol for NobitsSymProcessed {
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
impl SymbolProcessed for NobitsSymProcessed {}

impl hash::Hash for NobitsSymProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.vram_range.hash(state);
    }
}
impl PartialEq for NobitsSymProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.vram_range == other.vram_range
    }
}
impl PartialOrd for NobitsSymProcessed {
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
