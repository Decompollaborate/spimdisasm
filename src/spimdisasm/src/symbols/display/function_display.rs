/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use rabbitizer::{Instruction, InstructionDisplayFlags, Vram};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    context::Context,
    metadata::segment_metadata::FindSettings,
    relocation::RelocationInfo,
    symbols::{trait_symbol::RomSymbol, Symbol, SymbolFunction},
};

use super::SymCommonDisplaySettings;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct FunctionDisplaySettings {
    common: SymCommonDisplaySettings,

    display_flags: InstructionDisplayFlags,

    asm_label_indentation: u8,

    _gp_rel_hack: bool,
}

impl FunctionDisplaySettings {
    pub fn new(display_flags: InstructionDisplayFlags) -> Self {
        Self {
            common: SymCommonDisplaySettings::new(),
            display_flags,
            asm_label_indentation: 2,
            _gp_rel_hack: false,
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

                if self.settings.asm_label_indentation > 0 {
                    write!(
                        f,
                        "{:width$}",
                        " ",
                        width = self.settings.asm_label_indentation as usize
                    )?;
                }

                // PLACEHOLDER:
                write!(
                    f,
                    "{}:{}",
                    sym_label.display_name(),
                    self.settings.common.line_end()
                )?;
            }
        }

        Ok(())
    }

    fn display_instruction(
        &self,
        f: &mut fmt::Formatter<'_>,
        instr: &Instruction,
        prev_instr_had_delay_slot: bool,
    ) -> fmt::Result {
        let vram = instr.vram();
        let rom = self.sym.rom_vram_range().rom_from_vram(vram);
        self.settings
            .common
            .display_asm_comment(f, rom, vram, Some(instr.word()))?;

        // TODO: why two spaces instead of one?
        write!(f, "  ")?;

        let extra_ljust = if prev_instr_had_delay_slot {
            write!(f, " ")?;
            -1
        } else {
            0
        };

        let imm_override = self
            .get_reloc(instr)
            .and_then(|x| x.display(self.context, self.sym.parent_segment_info()));

        write!(
            f,
            "{}{}",
            instr.display(&self.settings.display_flags, imm_override, extra_ljust),
            self.settings.common.line_end()
        )
    }

    fn get_reloc(&self, instr: &Instruction) -> Option<&RelocationInfo> {
        let index = (instr.vram() - self.sym.vram_range().start()).inner() / 4;
        self.sym.relocs()[index as usize]
            .as_ref()
            .filter(|x| !x.reloc_type().is_none())
    }
}

impl fmt::Display for FunctionDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owned_segment = self
            .context
            .find_owned_segment(self.sym.parent_segment_info())?;
        let find_settings = FindSettings::default().with_allow_addend(false);
        let metadata = owned_segment
            .find_symbol(self.sym.vram_range().start(), find_settings)
            .ok_or(fmt::Error)?;

        let name = metadata.display_name();
        write!(f, ".globl {}{}", name, self.settings.common.line_end())?;

        write!(f, "{}:{}", name, self.settings.common.line_end())?;

        let mut prev_instr_had_delay_slot = false;
        for instr in self.sym.instructions() {
            let current_vram = instr.vram();
            self.display_label(f, current_vram)?;
            self.display_instruction(f, instr, prev_instr_had_delay_slot)?;

            prev_instr_had_delay_slot = instr.opcode().has_delay_slot();
        }

        write!(f, ".end {}{}", name, self.settings.common.line_end())
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl FunctionDisplaySettings {
        #[new]
        pub fn py_new(/*display_flags: InstructionDisplayFlags*/) -> Self {
            Self::new(InstructionDisplayFlags::default())
        }
    }
}
