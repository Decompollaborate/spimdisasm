/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use alloc::string::String;

use crate::{
    context::Context,
    metadata::segment_metadata::FindSettings,
    symbols::{Symbol, SymbolData},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymDataDisplaySettings {
    line_end: Option<String>,
}

impl Default for SymDataDisplaySettings {
    fn default() -> Self {
        Self::new()
    }
}

impl SymDataDisplaySettings {
    pub fn new() -> Self {
        Self { line_end: None }
    }

    pub(crate) fn line_end(&self) -> &str {
        if let Some(line_end) = &self.line_end {
            line_end
        } else {
            "\n"
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq)]
pub struct SymDataDisplay<'ctx, 'sym, 'flg> {
    context: &'ctx Context,
    sym: &'sym SymbolData,
    settings: &'flg SymDataDisplaySettings,
}

impl<'ctx, 'sym, 'flg> SymDataDisplay<'ctx, 'sym, 'flg> {
    pub(crate) fn new(
        context: &'ctx Context,
        sym: &'sym SymbolData,
        settings: &'flg SymDataDisplaySettings,
    ) -> Self {
        Self {
            context,
            sym,
            settings,
        }
    }
}

impl fmt::Display for SymDataDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owned_segment = self
            .context
            .find_owned_segment(self.sym.parent_segment_info())?;
        let find_settings = FindSettings::default().with_allow_addend(false);
        let metadata = owned_segment
            .find_symbol(self.sym.vram_range().start(), find_settings)
            .ok_or(fmt::Error)?;

        let name = metadata.display_name();
        write!(f, ".globl {}{}", name, self.settings.line_end())?;

        write!(f, "{}:{}", name, self.settings.line_end())?;

        for byte in self.sym.raw_bytes() {
            write!(f, ".byte 0x{:02X}{}", byte, self.settings.line_end())?;
        }

        Ok(())
    }
}
