/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::hash;

use crate::{
    addresses::{AddressRange, Size, Vram},
    config::Compiler,
    context::Context,
    metadata::{ParentSectionMetadata, SymbolMetadata},
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    symbols::{processed::NoloadSymProcessed, SymbolPreprocessed},
};

use crate::symbols::{Symbol, SymbolCreationError, SymbolPostProcessError};

const SECTION_TYPE: SectionType = SectionType::Bss;

#[derive(Debug, Clone)]
pub struct NoloadSym {
    vram_range: AddressRange<Vram>,
    parent_segment_info: ParentSegmentInfo,
}

impl NoloadSym {
    pub(crate) fn new(
        context: &mut Context,
        vram_range: AddressRange<Vram>,
        _in_section_offset: usize,
        parent_segment_info: ParentSegmentInfo,
        properties: NoloadSymProperties,
    ) -> Result<Self, SymbolCreationError> {
        let owned_segment = context.find_owned_segment_mut(&parent_segment_info)?;
        let metadata = owned_segment.add_self_symbol(
            vram_range.start(),
            None,
            vram_range.size(),
            SECTION_TYPE,
            &parent_segment_info,
            None,
            |_| Size::new(0),
        )?;

        properties.apply_to_metadata(metadata);

        Ok(Self {
            vram_range,
            parent_segment_info,
        })
    }
}

impl NoloadSym {
    pub(crate) fn post_process(
        self,
        context: &mut Context,
    ) -> Result<NoloadSymProcessed, SymbolPostProcessError> {
        NoloadSymProcessed::new(context, self.vram_range, self.parent_segment_info)
    }
}

impl Symbol for NoloadSym {
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
}
impl SymbolPreprocessed for NoloadSym {}

impl hash::Hash for NoloadSym {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.vram_range.hash(state);
    }
}
impl PartialEq for NoloadSym {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.vram_range == other.vram_range
    }
}
impl PartialOrd for NoloadSym {
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

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) struct NoloadSymProperties {
    pub parent_metadata: ParentSectionMetadata,
    pub compiler: Option<Compiler>,
    pub auto_pad_by: Option<Vram>,
}

impl NoloadSymProperties {
    fn apply_to_metadata(self, metadata: &mut SymbolMetadata) {
        metadata.set_parent_metadata(self.parent_metadata);

        if let Some(compiler) = self.compiler {
            metadata.set_compiler(compiler);
        }

        if let Some(auto_pad_by) = self.auto_pad_by {
            metadata.set_auto_created_pad_by(auto_pad_by);
        }
    }
}
