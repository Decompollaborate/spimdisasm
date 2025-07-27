/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Size, Vram},
    collections::addended_ordered_map::FindSettings,
    config::Endian,
    context::Context,
    metadata::{SegmentMetadata, SymbolMetadata, SymbolType},
    symbols::{processed::DataSymProcessed, RomSymbol, RomSymbolProcessed, Symbol},
};

use super::{
    sym_common_display::WordComment, sym_display_error::SymDisplayError, InternalSymDisplSettings,
    SymCommonDisplaySettings,
};

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

    pub fn set_rom_comment_width(&mut self, rom_comment_width: u8) {
        self.common.set_rom_comment_width(rom_comment_width);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub struct SymDataDisplay<'ctx, 'sym, 'flg> {
    context: &'ctx Context,
    sym: &'sym DataSymProcessed,
    settings: &'flg SymDataDisplaySettings,
    endian: Endian,

    owned_segment: &'ctx SegmentMetadata,
    metadata: &'ctx SymbolMetadata,

    internal_settings: InternalSymDisplSettings,
}

impl<'ctx, 'sym, 'flg> SymDataDisplay<'ctx, 'sym, 'flg> {
    pub(crate) fn new(
        context: &'ctx Context,
        sym: &'sym DataSymProcessed,
        settings: &'flg SymDataDisplaySettings,

        internal_settings: InternalSymDisplSettings,
    ) -> Result<Self, SymDisplayError> {
        let owned_segment = context.find_owned_segment(sym.parent_segment_info())?;
        let find_settings = FindSettings::new(false);
        let metadata = owned_segment
            .find_symbol(sym.vram_range().start(), find_settings)
            .ok_or(SymDisplayError::SelfSymNotFound())?;

        Ok(Self {
            context,
            sym,
            settings,
            endian: context.global_config().endian(),
            owned_segment,
            metadata,

            internal_settings,
        })
    }

    #[must_use]
    pub(crate) fn sym(&self) -> &'sym DataSymProcessed {
        self.sym
    }

    #[must_use]
    pub(crate) fn settings_common(&self) -> &'flg SymCommonDisplaySettings {
        &self.settings.common
    }
}

impl SymDataDisplay<'_, '_, '_> {
    fn display_sym_warnings(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if false {
            let ranges = self.sym.rom_vram_range();
            let rom = ranges.rom().start();
            let size = self.sym.raw_bytes().len();

            if let Some(sym_type) = self.metadata.sym_type() {
                match sym_type {
                    SymbolType::Byte => Ok(()),
                    SymbolType::Short if rom.inner() % 2 != 0 || size % 2 != 0 => {
                        write!(f, "/* This symbol has type {:?}, but it is not possible to emit it due to wrong alignment or size */{}", sym_type, self.settings.common.line_end())
                    }
                    SymbolType::Word | SymbolType::Float32 | SymbolType::CString
                        if rom.inner() % 4 != 0 || size % 4 != 0 =>
                    {
                        write!(f, "/* This symbol has type {:?}, but it is not possible to emit it due to wrong alignment or size */{}", sym_type, self.settings.common.line_end())
                    }
                    SymbolType::Function if rom.inner() % 4 != 0 || size % 4 != 0 => {
                        write!(f, "/* This symbol has type {:?}, but it is not possible to emit it due to wrong alignment or size */{}", sym_type, self.settings.common.line_end())
                    }
                    SymbolType::Jumptable | SymbolType::GccExceptTable
                        if rom.inner() % 4 != 0 || size % 4 != 0 =>
                    {
                        write!(f, "/* This symbol has type {:?}, but it is not possible to emit it due to wrong alignment or size */{}", sym_type, self.settings.common.line_end())
                    }
                    SymbolType::DWord | SymbolType::Float64
                        if rom.inner() % 8 != 0 || size % 8 != 0 =>
                    {
                        write!(f, "/* This symbol has type {:?}, but it is not possible to emit it due to wrong alignment or size */{}", sym_type, self.settings.common.line_end())
                    }

                    SymbolType::UserCustom => Ok(()),

                    SymbolType::Short
                    | SymbolType::Word
                    | SymbolType::Float32
                    | SymbolType::CString
                    | SymbolType::VirtualTable
                    | SymbolType::Function
                    | SymbolType::Jumptable
                    | SymbolType::GccExceptTable
                    | SymbolType::DWord
                    | SymbolType::Float64 => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    #[allow(clippy::if_same_then_else)]
    #[allow(clippy::needless_bool)]
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
        current_rom: Rom,
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

    #[allow(clippy::if_same_then_else)]
    #[allow(clippy::needless_bool)]
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
        current_rom: Rom,
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
        current_rom: Rom,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let arr_bytes = &self.sym.raw_bytes()[i..i + 4];
        let word = self.endian.word_from_bytes(arr_bytes);

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::U32(
                arr_bytes
                    .try_into()
                    .expect("Should be an array of the correct dimensions"),
            ),
        )?;

        if let Some(rel) = self.sym.relocs()[i / 4]
            .as_ref()
            .filter(|x| !x.reloc_type().is_none())
            .and_then(|x| {
                x.display(
                    self.context,
                    self.sym.parent_segment_info(),
                    !self.metadata.sym_type().is_some_and(|x| x.is_table())
                        && self.metadata.allow_ref_with_addend(),
                    self.metadata.compiler(),
                    self.internal_settings,
                )
            })
        {
            write!(f, "{rel}")?;
        } else {
            write!(f, ".word 0x{word:08X}")?;
        }

        write!(f, "{}", self.settings.common.line_end())?;

        Ok(4)
    }

    fn display_as_float32(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: Rom,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let arr_bytes = &self.sym.raw_bytes()[i..i + 4];
        let word = self.endian.word_from_bytes(arr_bytes);
        let float32 = f32::from_bits(word);
        if float32.is_nan() || float32.is_infinite() {
            return self.display_as_word(f, i, current_rom, current_vram);
        }

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::U32(
                arr_bytes
                    .try_into()
                    .expect("Should be an array of the correct dimensions"),
            ),
        )?;
        write!(f, ".float {:?}{}", float32, self.settings.common.line_end())?;

        Ok(4)
    }

    fn display_as_float64(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: Rom,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let arr_bytes = &self.sym.raw_bytes()[i..i + 8];
        let dword = self.endian.dword_from_bytes(arr_bytes);
        let float64 = f64::from_bits(dword);
        if float64.is_nan() || float64.is_infinite() {
            return self.display_as_word(f, i, current_rom, current_vram);
        }

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::U64(
                arr_bytes
                    .try_into()
                    .expect("Should be an array of the correct dimensions"),
            ),
        )?;
        write!(
            f,
            ".double {:?}{}",
            float64,
            self.settings.common.line_end()
        )?;

        Ok(8)
    }

    fn display_as_c_string(
        &self,
        f: &mut fmt::Formatter<'_>,
        i: usize,
        current_rom: Rom,
        current_vram: Vram,
    ) -> Result<usize, fmt::Error> {
        let bytes = &self.sym.raw_bytes()[i..];
        let str_end = if let Some(str_end) = bytes.iter().position(|x| *x == b'\0') {
            str_end
        } else {
            // write!(f, "/* Invalid string due to missing nul terminator */{}", self.settings.common.line_end())?;
            return self.display_as_word(f, i, current_rom, current_vram);
        };

        let decoded =
            if let Some(decoded) = self.sym.encoding().decode_to_strings_vec(&bytes[..str_end]) {
                decoded
            } else {
                // write!(f, "/* Invalid string due to decoding error */{}", self.settings.common.line_end())?;
                return self.display_as_word(f, i, current_rom, current_vram);
            };

        self.settings.common.display_asm_comment(
            f,
            Some(current_rom),
            current_vram,
            WordComment::No,
        )?;
        let mut first = true;
        for (sub_str, finished) in decoded {
            if !first {
                self.settings.common.display_asm_indendation(f)?;
            }
            // TODO: maybe change to `.string` instead of `.asciz`?
            let directive = if finished { ".asciz" } else { ".ascii" };
            write!(
                f,
                "{} \"{}\"{}",
                directive,
                sub_str,
                self.settings.common.line_end()
            )?;
            first = false;
        }

        let real_end = (str_end + 1).next_multiple_of(4);

        // Check if there's any non-zero data in the current word after the zero terminator.
        if str_end + 1 != real_end {
            assert!(real_end <= bytes.len());
            let must_show_padding = bytes[str_end + 1..real_end].iter().any(|x| *x != 0);

            if must_show_padding {
                for i in str_end + 1..real_end {
                    let offset = Size::new(i as u32);
                    self.display_as_byte(f, i, current_rom + offset, current_vram + offset)?;
                }
            }
        }

        if let Some(compiler) = self.metadata.compiler() {
            self.settings.common.display_alignment_directive(
                f,
                self.metadata,
                compiler,
                Some(2),
            )?;
        }

        Ok(real_end)
    }
}

impl fmt::Display for SymDataDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sym_type = self.metadata.sym_type();

        self.display_sym_warnings(f)?;

        self.settings
            .common
            .display_sym_property_comments(f, self.metadata, self.owned_segment)?;
        self.settings
            .common
            .display_sym_prev_alignment(f, self.metadata)?;
        self.settings.common.display_symbol_name(
            f,
            self.context.global_config(),
            self.metadata,
            false,
            self.metadata.section_type(),
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

            // Check if we have less bytes than a word left or if we have alignment issues.
            let advance = match (bytes_len - i, x % 4) {
                (1, _) => self.display_as_byte(f, i, current_rom, current_vram)?,
                (_, 1) => self.display_as_byte(f, i, current_rom, current_vram)?,
                (2 | 3, _) => {
                    if sym_type == Some(SymbolType::Byte) || self.is_byte(i) {
                        self.display_as_byte(f, i, current_rom, current_vram)?
                    } else {
                        self.display_as_short(f, i, current_rom, current_vram)?
                    }
                }
                (_, 2 | 3) => {
                    if sym_type == Some(SymbolType::Byte) || self.is_byte(i) {
                        self.display_as_byte(f, i, current_rom, current_vram)?
                    } else {
                        self.display_as_short(f, i, current_rom, current_vram)?
                    }
                }
                _ => {
                    // At this point we should have at least 4 bytes to display and we should have
                    // at least a 4bytes alignement.

                    // Try to display according to the given type.
                    match sym_type {
                        Some(SymbolType::Function) => {
                            // This should be cod, how did this end up here?
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        // TODO: consider adding a specialized thing for tables?
                        Some(SymbolType::Jumptable | SymbolType::GccExceptTable) => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::VirtualTable) => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        Some(SymbolType::Byte) => {
                            self.display_as_byte(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::Short) => {
                            self.display_as_short(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::Word) => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::DWord) if x % 8 == 0 && bytes_len - i >= 8 => {
                            // Maybe display DWords with `.quad` https://sourceware.org/binutils/docs/as/Quad.html?
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        Some(SymbolType::Float32) => {
                            self.display_as_float32(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::Float64) if x % 8 == 0 && bytes_len - i >= 8 => {
                            self.display_as_float64(f, i, current_rom, current_vram)?
                        }
                        Some(SymbolType::CString) => {
                            self.display_as_c_string(f, i, current_rom, current_vram)?
                        }

                        // Maybe someday add support for custom structs?
                        Some(SymbolType::UserCustom) => {
                            self.display_as_word(f, i, current_rom, current_vram)?
                        }

                        None | Some(SymbolType::DWord) | Some(SymbolType::Float64) => {
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
            .display_sym_end(f, self.context.global_config(), self.metadata)?;

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

        #[pyo3(name = "set_rom_comment_width")]
        pub fn py_set_rom_comment_width(&mut self, rom_comment_width: u8) {
            self.set_rom_comment_width(rom_comment_width);
        }
    }
}
