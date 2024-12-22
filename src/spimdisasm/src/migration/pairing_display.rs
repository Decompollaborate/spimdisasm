/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use alloc::{borrow::Cow, vec::Vec};

use crate::{
    context::Context,
    size::Size,
    symbols::{
        display::{
            FunctionDisplay, FunctionDisplaySettings, SymDataDisplay, SymDataDisplaySettings,
            SymDisplayError,
        },
        Symbol,
    },
};

use super::FuncRodataPairing;

pub struct FuncRodataPairingDisplay<
    'ctx,
    'text,
    'text_settings,
    'rodata,
    'rodata_settings,
    'text_label,
    'ro_label,
    'late_ro_label,
> {
    //pairing: &'pairing FuncRodataPairing<'text, 'rodata>,
    func_display: Option<FunctionDisplay<'ctx, 'text, 'text_settings>>,
    ro_syms_display: Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>,
    late_ro_syms_display: Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>,

    section_label_text: Cow<'text_label, str>,
    section_label_rodata: Cow<'ro_label, str>,
    section_label_late_rodata: Cow<'late_ro_label, str>,
}

impl<
        'ctx,
        'text,
        'text_settings,
        'rodata,
        'rodata_settings,
        'text_label,
        'ro_label,
        'late_ro_label,
    >
    FuncRodataPairingDisplay<
        'ctx,
        'text,
        'text_settings,
        'rodata,
        'rodata_settings,
        'text_label,
        'ro_label,
        'late_ro_label,
    >
{
    pub(super) fn new(
        pairing: &FuncRodataPairing<'text, 'rodata>,
        context: &'ctx Context,
        function_display_settings: &'text_settings FunctionDisplaySettings,
        rodata_display_settings: &'rodata_settings SymDataDisplaySettings,
        section_label_text: Option<Cow<'text_label, str>>,
        section_label_rodata: Option<Cow<'ro_label, str>>,
        section_label_late_rodata: Option<Cow<'late_ro_label, str>>,
    ) -> Result<Self, SymDisplayError> {
        let (func_display, ro_syms_display, late_ro_syms_display) = match pairing {
            FuncRodataPairing::SingleFunction { function } => {
                let func_display = Some(function.display(context, function_display_settings)?);
                let ro_syms_display = Vec::new();
                let late_ro_syms_display = Vec::new();

                (func_display, ro_syms_display, late_ro_syms_display)
            }
            FuncRodataPairing::SingleRodata { rodata } => {
                let func_display = None;
                let ro_syms_display = vec![rodata.display(context, rodata_display_settings)?];
                let late_ro_syms_display = Vec::new();

                (func_display, ro_syms_display, late_ro_syms_display)
            }
            FuncRodataPairing::Pairing {
                function,
                rodata_syms,
                late_rodata_syms,
            } => {
                let func_display = Some(function.display(context, function_display_settings)?);
                let ro_syms_display = rodata_syms
                    .iter()
                    .map(|x| x.display(context, rodata_display_settings))
                    .collect::<Result<Vec<_>, SymDisplayError>>()?;
                let late_ro_syms_display = late_rodata_syms
                    .iter()
                    .map(|x| x.display(context, rodata_display_settings))
                    .collect::<Result<Vec<_>, SymDisplayError>>()?;

                (func_display, ro_syms_display, late_ro_syms_display)
            }
        };

        let section_label_text = if let Some(section_label_text) = section_label_text {
            section_label_text
        } else {
            Cow::from(".text")
        };
        let section_label_rodata = if let Some(section_label_rodata) = section_label_rodata {
            section_label_rodata
        } else {
            Cow::from(".rodata")
        };
        let section_label_late_rodata =
            if let Some(section_label_late_rodata) = section_label_late_rodata {
                section_label_late_rodata
            } else {
                Cow::from(".late_rodata")
            };

        Ok(Self {
            func_display,
            ro_syms_display,
            late_ro_syms_display,
            section_label_text,
            section_label_rodata,
            section_label_late_rodata,
        })
    }
}

impl fmt::Display for FuncRodataPairingDisplay<'_, '_, '_, '_, '_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.ro_syms_display.is_empty() {
            let line_end = self.ro_syms_display[0].settings_common().line_end();

            write!(f, ".section {}{}", self.section_label_rodata, line_end)?;

            for sym_display in &self.ro_syms_display {
                // TODO:
                // f.write(sym.disassemble(migrate=True, useGlobalLabel=True, isSplittedSymbol=True))
                write!(f, "{}{}", sym_display, line_end)?;
            }
        }

        if !self.late_ro_syms_display.is_empty() {
            if let Some(func_display) = self.func_display {
                let line_end = self.late_ro_syms_display[0].settings_common().line_end();

                write!(f, ".section {}{}", self.section_label_late_rodata, line_end)?;

                let func_sym = func_display.sym();

                let late_rodata_size: Size = self
                    .late_ro_syms_display
                    .iter()
                    .map(|x| x.sym().size())
                    .fold(Size::new(0), |acc, x| acc + x);
                let function_size = func_sym.size();

                if late_rodata_size.inner() as f32 / function_size.inner() as f32 > 1.0 / 3.0 {
                    let first_late_rodata_vram =
                        self.late_ro_syms_display[0].sym().vram_range().start();
                    let align = if first_late_rodata_vram.inner() % 8 == 0 {
                        8
                    } else {
                        4
                    };

                    write!(f, ".late_rodata_alignment {}{}", align, line_end)?;
                }

                for sym_display in &self.late_ro_syms_display {
                    // TODO:
                    // f.write(sym.disassemble(migrate=True, useGlobalLabel=True, isSplittedSymbol=True))
                    write!(f, "{}{}", sym_display, line_end)?;
                }
            }
        }

        if let Some(func_display) = self.func_display {
            if !self.ro_syms_display.is_empty() || !self.late_ro_syms_display.is_empty() {
                let line_end = func_display.settings_common().line_end();

                write!(
                    f,
                    "{}.section {}{}",
                    line_end, self.section_label_text, line_end
                )?;
            }

            // TODO:
            // f.write(self.function.disassemble(migrate=self.hasRodataSyms(), isSplittedSymbol=True))
            write!(f, "{}", func_display)?;
        }

        Ok(())
    }
}
