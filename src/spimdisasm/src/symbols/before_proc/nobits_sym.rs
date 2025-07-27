/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::hash;

use alloc::collections::btree_map::BTreeMap;

use crate::{
    addresses::{AddressRange, Rom, Size, Vram},
    config::Compiler,
    context::Context,
    metadata::{ParentSectionMetadata, SymbolMetadata},
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
    symbols::{processed::NobitsSymProcessed, SymbolPreprocessed},
};

use crate::symbols::{Symbol, SymbolCreationError, SymbolPostProcessError};

const SECTION_TYPE: SectionType = SectionType::Bss;

#[derive(Debug, Clone)]
pub struct NobitsSym {
    vram_range: AddressRange<Vram>,
    parent_segment_info: ParentSegmentInfo,
}

impl NobitsSym {
    pub(crate) fn new(
        context: &mut Context,
        vram_range: AddressRange<Vram>,
        parent_segment_info: ParentSegmentInfo,
        properties: NobitsSymProperties,
    ) -> Result<Self, SymbolCreationError> {
        let symbol_name_generation_settings = context
            .global_config()
            .symbol_name_generation_settings()
            .clone();
        let owned_segment = context.find_owned_segment_mut(&parent_segment_info)?;
        let metadata = owned_segment.add_self_symbol(
            vram_range.start(),
            None,
            vram_range.size(),
            SECTION_TYPE,
            None,
            |_| Size::new(0),
            symbol_name_generation_settings,
        )?;

        properties.apply_to_metadata(metadata);

        Ok(Self {
            vram_range,
            parent_segment_info,
        })
    }
}

impl Symbol for NobitsSym {
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
impl SymbolPreprocessed for NobitsSym {
    type Output = NobitsSymProcessed;

    #[doc(hidden)]
    fn post_process(
        self,
        context: &mut Context,
        _user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<Self::Output, SymbolPostProcessError> {
        NobitsSymProcessed::new(context, self.vram_range, self.parent_segment_info)
    }
}

impl hash::Hash for NobitsSym {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.vram_range.hash(state);
    }
}
impl PartialEq for NobitsSym {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.vram_range == other.vram_range
    }
}
impl PartialOrd for NobitsSym {
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
pub(crate) struct NobitsSymProperties {
    pub parent_metadata: ParentSectionMetadata,
    pub compiler: Option<Compiler>,
    pub auto_pad_by: Option<Vram>,
}

impl NobitsSymProperties {
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
