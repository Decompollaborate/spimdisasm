/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use rabbitizer::{Instruction, InstructionDisplayFlags, Vram};

#[cfg(feature = "pyo3")]
use alloc::string::ToString;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    context::Context,
    metadata::{segment_metadata::FindSettings, SegmentMetadata, SymbolMetadata, SymbolType},
    relocation::RelocationInfo,
    size::Size,
    symbols::{
        display::sym_common_display::WordComment, trait_symbol::RomSymbol, Symbol, SymbolFunction,
    },
};

use super::{sym_display_error::SymDisplayError, SymCommonDisplaySettings};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct FunctionDisplaySettings {
    common: SymCommonDisplaySettings,

    display_flags: InstructionDisplayFlags,

    asm_label_indentation: u8,

    _gp_rel_hack: bool,

    migrate: bool,
}

impl FunctionDisplaySettings {
    pub fn new(display_flags: InstructionDisplayFlags) -> Self {
        Self {
            common: SymCommonDisplaySettings::new(),
            display_flags,
            asm_label_indentation: 2,
            _gp_rel_hack: false,
            migrate: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct FunctionDisplay<'ctx, 'sym, 'flg> {
    context: &'ctx Context,
    sym: &'sym SymbolFunction,
    settings: &'flg FunctionDisplaySettings,

    owned_segment: &'ctx SegmentMetadata,
    metadata: &'ctx SymbolMetadata,
}

impl<'ctx, 'sym, 'flg> FunctionDisplay<'ctx, 'sym, 'flg> {
    pub(crate) fn new(
        context: &'ctx Context,
        sym: &'sym SymbolFunction,
        settings: &'flg FunctionDisplaySettings,
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

impl FunctionDisplay<'_, '_, '_> {
    fn display_label(&self, f: &mut fmt::Formatter<'_>, current_vram: Vram) -> fmt::Result {
        if current_vram == self.sym.vram_range().start() {
            // Avoid duplicating first symbol
            return Ok(());
        }

        if let Some(sym_label) = self
            .owned_segment
            .find_symbol(current_vram, FindSettings::new().with_allow_addend(false))
        {
            if self.settings.asm_label_indentation > 0 {
                write!(
                    f,
                    "{:width$}",
                    " ",
                    width = self.settings.asm_label_indentation as usize
                )?;
            }

            let use_macro = sym_label.sym_type().is_none_or(|x| match x {
                SymbolType::Function => true,
                SymbolType::BranchLabel => false,
                SymbolType::JumptableLabel => !self.settings.migrate,
                SymbolType::GccExceptTableLabel => true,
                _ => false,
            });

            // TODO:
            /*
            if not use_macro:
                if common.GlobalConfig.ASM_GLOBALIZE_TEXT_LABELS_REFERENCED_BY_NON_JUMPTABLE:
                    # Check if any non-jumptable symbol references this label
                    for otherSym in labelSym.referenceSymbols:
                        if otherSym.getTypeSpecial() != common.SymbolSpecialType.jumptable:
                            use_macro = True
                            break
            */

            let name = sym_label.display_name();
            if use_macro {
                // label = labelSym.getReferenceeSymbols()

                self.settings.common.display_symbol_name(
                    f,
                    self.context.global_config(),
                    &name,
                    sym_label,
                    true,
                )?;
            } else {
                write!(f, "{}:{}", name, self.settings.common.line_end(),)?;
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
            .display_asm_comment(f, rom, vram, WordComment::U32(instr.word()))?;

        // TODO: why an extra space?
        write!(f, " ")?;

        let extra_ljust = if prev_instr_had_delay_slot {
            write!(f, " ")?;
            -1
        } else {
            0
        };

        let find_settings =
            FindSettings::default().with_allow_addend(self.metadata.allow_ref_with_addend());
        let imm_override = self
            .get_reloc(instr)
            .and_then(|x| x.display(self.context, self.sym.parent_segment_info(), find_settings));

        let instr_display = instr.display(&self.settings.display_flags, imm_override, extra_ljust);

        #[cfg(feature = "pyo3")]
        let instr_display = instr_display.to_string().replace("$s8", "$fp");

        write!(f, "{}", instr_display)
    }

    fn get_reloc(&self, instr: &Instruction) -> Option<&RelocationInfo> {
        let index = (instr.vram() - self.sym.vram_range().start()).inner() / 4;
        self.sym.relocs()[index as usize]
            .as_ref()
            .filter(|x| !x.reloc_type().is_none())
    }

    fn display_end_of_line_comment(
        &self,
        f: &mut fmt::Formatter<'_>,
        instr: &Instruction,
    ) -> fmt::Result {
        let vram = instr.vram();
        let rom = self.sym.rom_vram_range().rom_from_vram(vram).unwrap();

        if self.sym.handwritten_instrs().contains(&rom) {
            write!(f, " /* handwritten instruction */")?;
        }

        Ok(())
    }
}

impl fmt::Display for FunctionDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.sym.handwritten_instrs().is_empty() {
            write!(
                f,
                "/* Handwritten function */{}",
                self.settings.common.line_end()
            )?;
        }

        let name = self.metadata.display_name();

        self.settings
            .common
            .display_sym_property_comments(f, self.metadata, self.owned_segment)?;
        self.settings
            .common
            .display_sym_prev_alignment(f, self.metadata)?;
        self.settings.common.display_symbol_name(
            f,
            self.context.global_config(),
            &name,
            self.metadata,
            false,
        )?;

        let mut size = Size::new(0);

        let mut prev_instr_had_delay_slot = false;
        for instr in self.sym.instructions() {
            let current_vram = instr.vram();
            self.display_label(f, current_vram)?;
            self.display_instruction(f, instr, prev_instr_had_delay_slot)?;

            self.display_end_of_line_comment(f, instr)?;
            write!(f, "{}", self.settings.common.line_end())?;

            prev_instr_had_delay_slot = instr.opcode().has_delay_slot();

            size += Size::new(4);
            if Some(size) == self.metadata.size() {
                self.settings.common.display_sym_end(
                    f,
                    self.context.global_config(),
                    &name,
                    self.metadata,
                )?;
            }
        }

        self.settings
            .common
            .display_sym_post_alignment(f, self.metadata)?;

        Ok(())
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl FunctionDisplaySettings {
        #[new]
        pub fn py_new(/*display_flags: InstructionDisplayFlags*/) -> Self {
            Self::new(InstructionDisplayFlags::default().with_named_fpr(true))
        }
    }
}
