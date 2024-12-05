/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    context::Context,
    metadata::segment_metadata::FindSettings,
    size::Size,
    symbols::{RomSymbol, Symbol, SymbolData},
};

use super::SymCommonDisplaySettings;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SymDataDisplaySettings {
    common: SymCommonDisplaySettings,
}

impl Default for SymDataDisplaySettings {
    fn default() -> Self {
        Self::new()
    }
}

impl SymDataDisplaySettings {
    pub fn new() -> Self {
        Self {
            common: SymCommonDisplaySettings::new(),
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

        #[cfg(not(feature = "pyo3"))]
        {
            write!(f, ".globl {}{}", name, self.settings.common.line_end())?;
            write!(f, "{}:{}", name, self.settings.common.line_end())?;
        }
        #[cfg(feature = "pyo3")]
        {
            write!(f, "dlabel {}{}", name, self.settings.common.line_end())?;
        }

        let mut size = Size::new(0);

        let ranges = self.sym.rom_vram_range();
        let rom = ranges.rom().start();
        let vram = ranges.vram().start();
        for (i, byte) in self.sym.raw_bytes().iter().enumerate() {
            let offset = Size::new(i as u32);

            self.settings
                .common
                .display_asm_comment(f, Some(rom + offset), vram + offset, None)?;
            write!(f, ".byte 0x{:02X}{}", byte, self.settings.common.line_end())?;

            size += Size::new(1);
            if Some(size) == metadata.size() {
                write!(
                    f,
                    ".size {}, . - {}{}",
                    name,
                    name,
                    self.settings.common.line_end()
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl SymDataDisplaySettings {
        #[new]
        pub fn py_new() -> Self {
            Self::new()
        }
    }
}
