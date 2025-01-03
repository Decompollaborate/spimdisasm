/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use alloc::string::String;

use crate::{
    address_abstraction::Vram,
    config::{Compiler, GlobalConfig},
    metadata::{SegmentMetadata, SymbolMetadata, SymbolMetadataNameDisplay, SymbolType},
    rom_address::RomAddress,
};

pub(crate) enum WordComment {
    No,
    U32(u32),
    U64(u64),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymCommonDisplaySettings {
    line_end: Option<String>,

    emit_asm_comment: bool,

    asm_indentation: u8,
}

impl SymCommonDisplaySettings {
    #[must_use]
    pub fn new() -> Self {
        Self {
            line_end: None,
            emit_asm_comment: true,
            asm_indentation: 4,
        }
    }

    #[must_use]
    pub fn line_end(&self) -> &str {
        if let Some(line_end) = &self.line_end {
            line_end
        } else {
            "\n"
        }
    }
}

impl SymCommonDisplaySettings {
    pub fn display_sym_property_comments(
        &self,
        f: &mut fmt::Formatter<'_>,
        metadata: &SymbolMetadata,
        _owned_segment: &SegmentMetadata,
    ) -> fmt::Result {
        if !self.emit_asm_comment {
            return Ok(());
        }

        if false {
            write!(
                f,
                "/* access type: {:?} */{}",
                metadata.access_type(),
                self.line_end()
            )?;
            write!(
                f,
                "/* user_declared_type: {:?} */{}",
                metadata.user_declared_type(),
                self.line_end()
            )?;
            write!(
                f,
                "/* autodetected_type: {:?} */{}",
                metadata.autodetected_type(),
                self.line_end()
            )?;
        }

        // TODO
        // if self.isStatic():
        if false {
            write!(f, "/* static variable */{}", self.line_end())?;
        }
        if let Some(main_sym_vram) = metadata.autogenerated_pad_info() {
            write!(f, "/* Automatically generated and unreferenced pad ",)?;
            if false {
                // This block currently exists for debugging only.
                // Maybe convert into a proper feature in the future?
                write!(f, "(generated by {}) ", main_sym_vram)?;
            }
            write!(f, "*/{}", self.line_end())?;
        }

        Ok(())
    }

    pub fn display_symbol_name(
        &self,
        f: &mut fmt::Formatter<'_>,
        global_config: &GlobalConfig,
        sym_name: &SymbolMetadataNameDisplay,
        metadata: &SymbolMetadata,
        in_middle: bool,
    ) -> fmt::Result {
        if let Some(macro_labels) = global_config.macro_labels() {
            // Write the label, ie `glabel`, `dlabel`, etc
            if let Some(sym_type) = metadata.sym_type() {
                match sym_type {
                    SymbolType::Function => {
                        if in_middle {
                            write!(f, "{}", macro_labels.alt_func())?;
                        } else {
                            write!(f, "{}", macro_labels.func())?;
                        }
                    }
                    SymbolType::JumptableLabel => write!(f, "{}", macro_labels.jtbl_label())?,
                    SymbolType::GccExceptTableLabel => write!(f, "{}", macro_labels.ehtbl_label())?,
                    SymbolType::BranchLabel
                    | SymbolType::Jumptable
                    | SymbolType::GccExceptTable
                    | SymbolType::Byte
                    | SymbolType::Short
                    | SymbolType::Word
                    | SymbolType::DWord
                    | SymbolType::Float32
                    | SymbolType::Float64
                    | SymbolType::CString
                    | SymbolType::UserCustom => write!(f, "{}", macro_labels.data())?,
                }
            } else {
                write!(f, "{}", macro_labels.data())?
            }

            write!(f, " {}", sym_name)?;

            match metadata.visibility() {
                None | Some("global") | Some("globl") => {}
                Some(vis) => write!(f, ", {}", vis)?,
            }
        } else {
            /*
            .globl func_80045DD0
            .type func_80045DD0,@function
            .ent func_80045DD0
            func_80045DD0:
            */
            let vis = match metadata.visibility() {
                None | Some("globl") => "globl",
                Some(vis) => vis,
            };
            write!(f, ".{} {}{}", vis, sym_name, self.line_end())?;

            if let Some(sym_type) = metadata.sym_type() {
                match sym_type {
                    SymbolType::Function => {
                        write!(f, ".type {}, @function{}", sym_name, self.line_end())?;
                        if in_middle {
                            write!(f, ".aent {}{}", sym_name, self.line_end())?;
                        } else {
                            write!(f, ".ent {}{}", sym_name, self.line_end())?;
                        }
                    }
                    SymbolType::JumptableLabel
                    | SymbolType::GccExceptTableLabel
                    | SymbolType::BranchLabel => {}

                    SymbolType::Jumptable | SymbolType::GccExceptTable => {}
                    SymbolType::Byte
                    | SymbolType::Short
                    | SymbolType::Word
                    | SymbolType::DWord
                    | SymbolType::Float32
                    | SymbolType::Float64
                    | SymbolType::CString
                    | SymbolType::UserCustom => {
                        write!(f, ".type {}, @object{}", sym_name, self.line_end())?
                    }
                }
            } else {
                write!(f, ".type {}, @object{}", sym_name, self.line_end())?
            }

            write!(f, "{}:", sym_name)?;
        }

        /*
        if GlobalConfig.GLABEL_ASM_COUNT:
            if self.index is not None:
                label += f" # {self.index}"
        */

        write!(f, "{}", self.line_end())
    }

    /*
    pub fn display_symbol_start(&self, f: &mut fmt::Formatter<'_>, sym_name: &SymbolMetadataNameDisplay) -> fmt::Result {
        /*
        output = self.contextSym.getReferenceeSymbols()
        output += self.getPrevAlignDirective(0)

        symName = self.getName()
        output += self.getSymbolAsmDeclaration(symName, useGlobalLabel)
        */

        Ok(())
    }
    */

    fn display_alignment_directive(
        &self,
        f: &mut fmt::Formatter<'_>,
        metadata: &SymbolMetadata,
        compiler: Compiler,
        shift_value: Option<u8>,
    ) -> fmt::Result {
        let shift_value = if let Some(shift_value) = shift_value {
            shift_value
        } else {
            return Ok(());
        };

        let parent_metadata = if let Some(parent_metadata) = metadata.parent_metadata() {
            parent_metadata
        } else {
            // Can't emit alignment directives if we don't have info about the parent.
            return Ok(());
        };

        let shifted_val = 1u32 << shift_value;

        let sub_ram = if compiler.symbol_alignment_requires_aligned_section() {
            if parent_metadata.vram().inner() % shifted_val != 0 {
                // Can't emit alignment directives if the parent file isn't properly aligned.
                return Ok(());
            }

            metadata.vram().inner()
        } else {
            // Check if alignment is relative to the file or to the full binary.
            (metadata.vram() - parent_metadata.vram()).inner() as u32
        };

        if sub_ram % shifted_val != 0 {
            // Emitting an alignment directive when the symbol is not already aligned to the desired
            // alignment would break matching.
            return Ok(());
        }

        write!(f, ".align {}{}", shift_value, self.line_end())
    }

    pub fn display_sym_prev_alignment(
        &self,
        f: &mut fmt::Formatter<'_>,
        metadata: &SymbolMetadata,
    ) -> fmt::Result {
        if let Some(compiler) = metadata.compiler() {
            if let Some(sym_type) = metadata.sym_type() {
                self.display_alignment_directive(
                    f,
                    metadata,
                    compiler,
                    compiler.prev_align_for_type(sym_type),
                )?;
            }
        }

        Ok(())
    }

    pub fn display_sym_post_alignment(
        &self,
        f: &mut fmt::Formatter<'_>,
        metadata: &SymbolMetadata,
    ) -> fmt::Result {
        if let Some(compiler) = metadata.compiler() {
            if let Some(sym_type) = metadata.sym_type() {
                self.display_alignment_directive(
                    f,
                    metadata,
                    compiler,
                    compiler.post_align_for_type(sym_type),
                )?;
            }
        }

        Ok(())
    }

    pub fn display_sym_end(
        &self,
        f: &mut fmt::Formatter<'_>,
        global_config: &GlobalConfig,
        sym_name: &SymbolMetadataNameDisplay,
        metadata: &SymbolMetadata,
    ) -> fmt::Result {
        if let Some(macro_labels) = global_config.macro_labels() {
            if let Some(sym_type) = metadata.sym_type() {
                match sym_type {
                    SymbolType::Function => {
                        if let Some(func_end) = macro_labels.func_end() {
                            write!(f, "{} {}{}", func_end, sym_name, self.line_end())?;
                        }
                    }
                    SymbolType::JumptableLabel
                    | SymbolType::GccExceptTableLabel
                    | SymbolType::BranchLabel => {}

                    SymbolType::Jumptable
                    | SymbolType::GccExceptTable
                    | SymbolType::Byte
                    | SymbolType::Short
                    | SymbolType::Word
                    | SymbolType::DWord
                    | SymbolType::Float32
                    | SymbolType::Float64
                    | SymbolType::CString
                    | SymbolType::UserCustom => {
                        if let Some(data_end) = macro_labels.data_end() {
                            write!(f, "{} {}{}", data_end, sym_name, self.line_end())?;
                        }
                    }
                }
            } else if let Some(data_end) = macro_labels.data_end() {
                write!(f, "{} {}{}", data_end, sym_name, self.line_end())?;
            }
        } else if let Some(SymbolType::Function) = metadata.sym_type() {
            write!(f, ".end {}{}", sym_name, self.line_end())?;
        }

        if global_config.emit_size_directive() {
            write!(f, ".size {}, . - {}{}", sym_name, sym_name, self.line_end())?;
        }

        Ok(())
    }

    pub fn display_asm_comment(
        &self,
        f: &mut fmt::Formatter<'_>,
        rom: Option<RomAddress>,
        vram: Vram,
        word_comment: WordComment,
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

        // TODO: endian
        match word_comment {
            WordComment::No => {}
            WordComment::U32(word) => write!(f, "{:08X} ", word)?,
            WordComment::U64(dword) => write!(f, "{:016X} ", dword)?,
        }

        write!(f, "*/ ")
    }
}
