/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

// TODO: maybe rename to FunctionDisplay?

use core::fmt;

use alloc::string::String;
use rabbitizer::{DisplayFlags, Instruction};

use crate::{
    context::Context,
    metadata::segment_metadata::FindSettings,
    symbols::{Symbol, SymbolFunction},
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
    fn display_label(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }

    fn display_instruction(
        &self,
        f: &mut fmt::Formatter<'_>,
        instr: &Instruction,
        prev_instr_had_delay_slot: bool,
    ) -> fmt::Result {
        // TODO: imm_override
        let imm_override = None;

        if prev_instr_had_delay_slot {
            write!(f, " ")?;
        }

        write!(
            f,
            "{}{}",
            instr.display(imm_override, &self.settings.display_flags),
            self.settings.line_end()
        )
    }
}

impl<'ctx, 'sym, 'flg> fmt::Display for FunctionDisplay<'ctx, 'sym, 'flg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let owned_segment = self
            .context
            .find_owned_segment(self.sym.parent_segment_info())?;
        let find_settings = FindSettings::default().with_allow_addend(false);
        let metadata = owned_segment
            .find_symbol(self.sym.vram(), find_settings)
            .ok_or(fmt::Error)?;

        let name = metadata.display_name();
        write!(f, ".globl {}{}", name, self.settings.line_end())?;

        write!(f, "{}:{}", name, self.settings.line_end())?;

        let mut prev_instr_had_delay_slot = false;
        for instr in self.sym.instructions() {
            self.display_label(f)?;
            self.display_instruction(f, instr, prev_instr_had_delay_slot)?;

            prev_instr_had_delay_slot = instr.opcode().has_delay_slot();
        }

        write!(f, ".end {}{}", name, self.settings.line_end())
    }
}
