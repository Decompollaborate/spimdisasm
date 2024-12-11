/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rabbitizer::Vram;

use crate::{
    config::Endian,
    context::Context,
    metadata::{segment_metadata::FindSettings, SymbolType},
    rom_address::RomAddress,
    size::Size,
    symbols::{RomSymbol, Symbol, SymbolData},
};

use super::{sym_common_display::WordComment, SymCommonDisplaySettings};

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

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::No,
        )?;
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

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::No,
        )?;
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

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::U32(word),
        )?;

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

    fn display_as_float32(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: RomAddress,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let word = self.endian.word_from_bytes(&self.sym.raw_bytes()[i..i + 4]);
        let float32 = f32::from_bits(word);
        if float32.is_nan() || float32.is_infinite() {
            return self.display_as_word(f, i, current_rom, current_vram);
        }

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::U32(word),
        )?;
        write!(f, ".float {:?}{}", float32, self.settings.common.line_end())?;

        Ok(4)
    }

    fn display_as_float64(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: RomAddress,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let dword = self
            .endian
            .dword_from_bytes(&self.sym.raw_bytes()[i..i + 8]);
        let float64 = f64::from_bits(dword);
        if float64.is_nan() || float64.is_infinite() {
            return self.display_as_word(f, i, current_rom, current_vram);
        }

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::U64(dword),
        )?;
        write!(
            f,
            ".double {:?}{}",
            float64,
            self.settings.common.line_end()
        )?;

        Ok(8)
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
        let sym_type = metadata.sym_type();

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
        let bytes_len = self.sym.raw_bytes().len();
        while i < bytes_len {
            let offset = Size::new(i as u32);
            let current_rom = rom + offset;
            let current_vram = vram + offset;
            let x = current_rom.inner();

            // Check if we have less bytes than a word left.
            let advance = match bytes_len - i {
                1 => self.display_as_byte(f, i, current_rom, current_vram)?,
                2 | 3 => {
                    if sym_type == Some(&SymbolType::Byte) || self.is_byte(i) {
                        self.display_as_byte(f, i, current_rom, current_vram)?
                    } else {
                        self.display_as_short(f, i, current_rom, current_vram)?
                    }
                }
                _ => {
                    // Try to display according to the given type.
                    match sym_type {
                        Some(
                            SymbolType::Function
                            | SymbolType::BranchLabel
                            | SymbolType::JumptableLabel
                            | SymbolType::GccExceptTableLabel,
                        ) if x % 4 == 0 => {
                            // This should be cod, how did this end up here?
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        // TODO: consider adding a specialized thing for tables?
                        Some(SymbolType::Jumptable | SymbolType::GccExceptTable) if x % 4 == 0 => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        Some(SymbolType::Byte) => {
                            self.display_as_byte(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::Short) if x % 2 == 0 => {
                            self.display_as_short(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::Word) if x % 4 == 0 => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::DWord) if x % 8 == 0 && bytes_len - i >= 8 => {
                            // Maybe display DWords with `https://sourceware.org/binutils/docs/as/Quad.html`?
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        Some(SymbolType::Float32) if x % 4 == 0 => {
                            self.display_as_float32(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::Float64) if x % 8 == 0 && bytes_len - i >= 8 => {
                            self.display_as_float64(f, i, current_rom, current_vram)?
                        }
                        // TODO
                        Some(SymbolType::CString) if x % 4 == 0 => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        // Maybe someday add support for custom structs?
                        Some(SymbolType::UserCustom) if x % 4 == 0 => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        None | Some(_) => {
                            // heuristic game to guess on what's best for this data
                            if self.is_byte(i) {
                                self.display_as_byte(f, i, current_rom, current_vram)?
                            } else if self.is_short(i) {
                                self.display_as_short(f, i, current_rom, current_vram)?
                            } else {
                                self.display_as_word(f, i, current_rom, current_vram)?
                            }
                        }
                    }
                }
            };

            debug_assert!(advance > 0);

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
