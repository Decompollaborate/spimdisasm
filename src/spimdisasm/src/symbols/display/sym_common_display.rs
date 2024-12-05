/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use alloc::string::String;
use rabbitizer::Vram;

use crate::rom_address::RomAddress;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymCommonDisplaySettings {
    line_end: Option<String>,

    emit_asm_comment: bool,

    asm_indentation: u8,
}

impl SymCommonDisplaySettings {
    pub fn new() -> Self {
        Self {
            line_end: None,
            emit_asm_comment: true,
            asm_indentation: 4,
        }
    }

    pub fn line_end(&self) -> &str {
        if let Some(line_end) = &self.line_end {
            line_end
        } else {
            "\n"
        }
    }
}

impl SymCommonDisplaySettings {
    pub fn display_asm_comment(
        &self,
        f: &mut fmt::Formatter<'_>,
        rom: Option<RomAddress>,
        vram: Vram,
        word: Option<u32>,
    ) -> fmt::Result {
        if self.asm_indentation > 0 {
            write!(f, "{:width$}", " ", width = self.asm_indentation as usize)?;
        }

        if !self.emit_asm_comment {
            return Ok(());
        }

        // TODO:
        /*
        if emitRomOffset:
            offsetHex = "{0:0{1}X} ".format(localOffset + self.inFileOffset + self.commentOffset, common.GlobalConfig.ASM_COMMENT_OFFSET_WIDTH)
        else:
            offsetHex = ""

        currentVram = self.getVramOffset(localOffset)
        vramHex = f"{currentVram:08X}"

        wordValueHex = ""
        if wordValue is not None:
            if isDouble:
                wordValueHex = f"{common.Utils.qwordToCurrenEndian(wordValue):016X} "
            else:
                wordValueHex = f"{common.Utils.wordToCurrenEndian(wordValue):08X} "

        return f"{indentation}/* {offsetHex}{vramHex} {wordValueHex}*/
"
        */

        write!(f, "/* ")?;
        if let Some(rom) = rom {
            // TODO: implement display for RomAddress
            write!(f, "{:06X} ", rom.inner())?;
        }
        write!(f, "{} ", vram)?;
        if let Some(word) = word {
            // TODO: endian
            write!(f, "{:08X} ", word)?;
        }

        write!(f, "*/ ")
    }
}
