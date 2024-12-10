/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rabbitizer::Vram;

use crate::{
    config::Endian,
    context::Context,
    metadata::segment_metadata::FindSettings,
    rom_address::RomAddress,
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
    endian: Endian,
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
            endian: context.global_config().endian(),
        }
    }
}

impl SymDataDisplay<'_, '_, '_> {
    fn is_byte(&self, i: usize) -> bool {
        // TODO: implement access types and such
        if i % 2 != 0 {
            true
        } else if self.sym.rom_range().start().inner() % 2 != 0 {
            true
        } else if self.sym.size().inner() as usize - i < 2 {
            true
        } else {
            false
        }
    }

    fn display_as_byte(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: RomAddress,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let byte = self.sym.raw_bytes()[i];

        self.settings
            .common
            .display_asm_comment(f, Some(current_rom), current_vram, None)?;
        write!(f, ".byte 0x{:02X}{}", byte, self.settings.common.line_end())?;

        Ok(1)
    }

    fn is_short(&self, i: usize) -> bool {
        let rom = self.sym.rom_range().start();

        if i % 4 != 0 && i % 2 == 0 {
            true
        } else if rom.inner() % 4 != 0 && rom.inner() % 2 == 0 {
            true
        } else if self.sym.size().inner() as usize - i < 4 {
            true
        } else {
            false
        }
    }

    fn display_as_short(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: RomAddress,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let short = self
            .endian
            .short_from_bytes(&self.sym.raw_bytes()[i..i + 2]);

        self.settings
            .common
            .display_asm_comment(f, Some(current_rom), current_vram, None)?;
        write!(
            f,
            ".short 0x{:04X}{}",
            short,
            self.settings.common.line_end()
        )?;

        Ok(2)
    }

    fn display_as_word(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: RomAddress,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let word = self.endian.word_from_bytes(&self.sym.raw_bytes()[i..i + 4]);

        self.settings
            .common
            .display_asm_comment(f, Some(current_rom), current_vram, Some(word))?;

        if let Some(rel) = self.sym.relocs()[i / 4]
            .as_ref()
            .filter(|x| !x.reloc_type().is_none())
            .and_then(|x| x.display(self.context, self.sym.parent_segment_info()))
        {
            write!(f, "{}", rel)?;
        } else {
            write!(f, ".word 0x{:08X}", word)?;
        }

        write!(f, "{}", self.settings.common.line_end())?;

        Ok(4)
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

        self.settings
            .common
            .display_sym_property_comments(f, metadata, &owned_segment)?;
        self.settings.common.display_symbol_name(
            f,
            self.context.global_config(),
            &name,
            metadata,
            false,
        )?;

        let ranges = self.sym.rom_vram_range();
        let rom = ranges.rom().start();
        let vram = ranges.vram().start();
        let mut i = 0;
        while i < self.sym.raw_bytes().len() {
            let offset = Size::new(i as u32);
            let current_rom = rom + offset;
            let current_vram = vram + offset;

            let advance = if self.is_byte(i) {
                self.display_as_byte(f, i, current_rom, current_vram)?
            } else if self.is_short(i) {
                self.display_as_short(f, i, current_rom, current_vram)?
            } else {
                self.display_as_word(f, i, current_rom, current_vram)?
            };

            i += advance;
        }

        self.settings
            .common
            .display_sym_end(f, self.context.global_config(), &name, metadata)?;

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
