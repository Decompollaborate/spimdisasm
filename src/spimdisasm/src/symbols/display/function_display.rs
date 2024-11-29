/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use alloc::string::{String, ToString};
use rabbitizer::{DisplayFlags, Instruction, Vram};

use crate::{
    context::Context,
    metadata::segment_metadata::FindSettings,
    relocation::RelocationInfo,
    symbols::{trait_symbol::RomSymbol, Symbol, SymbolFunction},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionDisplaySettings {
    display_flags: DisplayFlags,
    line_end: Option<String>,
    _gp_rel_hack: bool,
}

impl FunctionDisplaySettings {
    pub fn new(display_flags: DisplayFlags) -> Self {
        Self {
            display_flags,
            line_end: None,
            _gp_rel_hack: false,
        }
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
pub struct FunctionDisplay<'ctx, 'sym, 'flg> {
    context: &'ctx Context,
    sym: &'sym SymbolFunction,
    settings: &'flg FunctionDisplaySettings,
}

impl<'ctx, 'sym, 'flg> FunctionDisplay<'ctx, 'sym, 'flg> {
    pub(crate) fn new(
        context: &'ctx Context,
        sym: &'sym SymbolFunction,
        settings: &'flg FunctionDisplaySettings,
    ) -> Self {
        Self {
            context,
            sym,
            settings,
        }
    }
}

impl FunctionDisplay<'_, '_, '_> {
    fn display_label(&self, f: &mut fmt::Formatter<'_>, current_vram: Vram) -> fmt::Result {
        if current_vram == self.sym.vram_range().start() {
            // Avoid duplicating first symbol
            return Ok(());
        }

        if let Some(sym_label) = self
            .context
            .find_owned_segment(self.sym.parent_segment_info())?
            .find_symbol(current_vram, FindSettings::new().with_allow_addend(false))
        {
            if let Some(_typ) = sym_label.sym_type() {
                // TODO:
                /*
                useLabelMacro = labelSymType is None or labelSymType == common.SymbolSpecialType.function or (labelSymType == common.SymbolSpecialType.jumptablelabel and not migrate) or labelSymType == common.SymbolSpecialType.gccexcepttablelabel
                if not useLabelMacro:
                    if common.GlobalConfig.ASM_GLOBALIZE_TEXT_LABELS_REFERENCED_BY_NON_JUMPTABLE:
                        # Check if any non-jumptable symbol references this label
                        for otherSym in labelSym.referenceSymbols:
                            if otherSym.getTypeSpecial() != common.SymbolSpecialType.jumptable:
                                useLabelMacro = True
                                break

                if useLabelMacro:
                    label = labelSym.getReferenceeSymbols()
                    labelMacro = labelSym.getLabelMacro(isInMiddleLabel=True)
                    if labelMacro is not None:
                        label += f"{labelMacro} {labelSym.getName()}{common.GlobalConfig.LINE_ENDS}"
                    if common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL:
                        label += f"{labelSym.getName()}:{common.GlobalConfig.LINE_ENDS}"
                else:
                    label = labelSym.getName() + ":" + common.GlobalConfig.LINE_ENDS
                label = (" " * common.GlobalConfig.ASM_INDENTATION_LABELS) + label
                return label
                */

                // PLACEHOLDER:
                write!(
                    f,
                    "{}:{}",
                    sym_label.display_name(),
                    self.settings.line_end()
                )?;
            }
        }

        Ok(())
    }

    fn display_asm_comment(&self, f: &mut fmt::Formatter<'_>, instr: &Instruction) -> fmt::Result {
        // TODO:
        /*
        indentation = " " * common.GlobalConfig.ASM_INDENTATION

        if not common.GlobalConfig.ASM_COMMENT:
            return indentation

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
        let current_vram = instr.vram();
        if let Some(rom) = self.sym.rom_from_vram(current_vram) {
            // TODO: implement display for RomAddress
            write!(f, "{:06X} ", rom.inner())?;
        }
        write!(f, "{} ", current_vram)?;
        // TODO: endian
        write!(f, "{:08X} ", instr.word())?;

        write!(f, "*/")
    }

    fn display_instruction(
        &self,
        f: &mut fmt::Formatter<'_>,
        instr: &Instruction,
        prev_instr_had_delay_slot: bool,
    ) -> fmt::Result {
        self.display_asm_comment(f, instr)?;
        // TODO: why two spaces instead of one?
        write!(f, "  ")?;

        if prev_instr_had_delay_slot {
            write!(f, " ")?;
        }

        let temp = self
            .get_reloc(instr)
            .map(|x| {
                x.display(self.context, self.sym.parent_segment_info())
                    .map(|x| x.to_string())
            })
            .flatten();
        let imm_override = temp.as_ref().map(|x| x.as_str());

        write!(
            f,
            "{}{}",
            instr.display(imm_override, &self.settings.display_flags),
            self.settings.line_end()
        )
    }

    fn get_reloc(&self, instr: &Instruction) -> Option<&RelocationInfo> {
        let index = (instr.vram() - self.sym.vram_range().start()).inner() / 4;
        self.sym.relocs()[index as usize]
            .as_ref()
            .filter(|x| !x.reloc_type().is_none())
    }
}

impl<'ctx, 'sym, 'flg> fmt::Display for FunctionDisplay<'ctx, 'sym, 'flg> {
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

        let mut prev_instr_had_delay_slot = false;
        for instr in self.sym.instructions() {
            let current_vram = instr.vram();
            self.display_label(f, current_vram)?;
            self.display_instruction(f, instr, prev_instr_had_delay_slot)?;

            prev_instr_had_delay_slot = instr.opcode().has_delay_slot();
        }

        write!(f, ".end {}{}", name, self.settings.line_end())
    }
}
