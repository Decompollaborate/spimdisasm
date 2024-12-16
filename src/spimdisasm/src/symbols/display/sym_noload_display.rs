/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    context::Context,
    metadata::{segment_metadata::FindSettings, SegmentMetadata, SymbolMetadata},
    symbols::{Symbol, SymbolNoload},
};

use super::{
    sym_common_display::WordComment, sym_display_error::SymDisplayError, SymCommonDisplaySettings,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SymNoloadDisplaySettings {
    common: SymCommonDisplaySettings,
}

impl Default for SymNoloadDisplaySettings {
    fn default() -> Self {
        Self::new()
    }
}

impl SymNoloadDisplaySettings {
    pub fn new() -> Self {
        Self {
            common: SymCommonDisplaySettings::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub struct SymNoloadDisplay<'ctx, 'sym, 'flg> {
    context: &'ctx Context,
    sym: &'sym SymbolNoload,
    settings: &'flg SymNoloadDisplaySettings,

    owned_segment: &'ctx SegmentMetadata,
    metadata: &'ctx SymbolMetadata,
}

impl<'ctx, 'sym, 'flg> SymNoloadDisplay<'ctx, 'sym, 'flg> {
    pub(crate) fn new(
        context: &'ctx Context,
        sym: &'sym SymbolNoload,
        settings: &'flg SymNoloadDisplaySettings,
    ) -> Result<Self, SymDisplayError> {
        let owned_segment = context.find_owned_segment(sym.parent_segment_info())?;
        let find_settings = FindSettings::default().with_allow_addend(false);
        let metadata = owned_segment
            .find_symbol(sym.vram_range().start(), find_settings)
            .ok_or(SymDisplayError::SelfSymNotFound())?;

        Ok(Self {
            context,
            sym,
            settings,
            owned_segment,
            metadata,
        })
    }
}

impl fmt::Display for SymNoloadDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.metadata.display_name();

        self.settings
            .common
            .display_sym_property_comments(f, self.metadata, self.owned_segment)?;
        self.settings.common.display_symbol_name(
            f,
            self.context.global_config(),
            &name,
            self.metadata,
            false,
        )?;

        self.settings.common.display_asm_comment(
            f,
            None,
            self.sym.vram_range().start(),
            WordComment::No,
        )?;
        write!(
            f,
            ".space {}{}",
            self.sym.size(),
            self.settings.common.line_end()
        )?;

        Ok(())
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl SymNoloadDisplaySettings {
        #[new]
        pub fn py_new() -> Self {
            Self::new()
        }
    }
}
