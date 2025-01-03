/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use alloc::vec::Vec;

use crate::{
    addresses::Size,
    context::Context,
    sections::{Section, SectionData, SectionExecutable},
    symbols::{
        display::{
            FunctionDisplay, FunctionDisplaySettings, InternalSymDisplSettings, SymDataDisplay,
            SymDataDisplaySettings,
        },
        Symbol,
    },
};

use super::{FuncRodataPairing, PairingError};

#[derive(Debug, Clone, PartialEq)]
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
    func_display: Option<FunctionDisplay<'ctx, 'text, 'text_settings>>,
    ro_syms_display: Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>,
    late_ro_syms_display: Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>,

    settings: FuncRodataPairingDisplaySettings<'text_label, 'ro_label, 'late_ro_label>,
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
        pairing: &FuncRodataPairing,
        context: &'ctx Context,
        text_section: Option<&'text SectionExecutable>,
        function_display_settings: &'text_settings FunctionDisplaySettings,
        rodata_section: Option<&'rodata SectionData>,
        rodata_display_settings: &'rodata_settings SymDataDisplaySettings,
        settings: FuncRodataPairingDisplaySettings<'text_label, 'ro_label, 'late_ro_label>,
    ) -> Result<Self, PairingError> {
        let (func_display, ro_syms_display, late_ro_syms_display) = match pairing {
            FuncRodataPairing::SingleRodata { rodata_index } => {
                let (ro_syms_display, late_ro_syms_display) = Self::do_rodata_section(
                    context,
                    rodata_section,
                    rodata_display_settings,
                    &[*rodata_index],
                    &[],
                )?;

                let func_display = None;

                (func_display, ro_syms_display, late_ro_syms_display)
            }
            FuncRodataPairing::Pairing {
                function_index,
                rodata_indices,
                late_rodata_indices,
            } => {
                let text_section = if let Some(text_section) = text_section {
                    text_section
                } else {
                    return Err(PairingError::MissingTextSection {});
                };
                let functions = text_section.functions();
                let func = if let Some(func) = functions.get(*function_index) {
                    func
                } else {
                    return Err(PairingError::FunctionOutOfBounds {
                        index: *function_index,
                        len: functions.len(),
                        section_name: text_section.name().into(),
                    });
                };

                let (ro_syms_display, late_ro_syms_display) = Self::do_rodata_section(
                    context,
                    rodata_section,
                    rodata_display_settings,
                    rodata_indices,
                    late_rodata_indices,
                )?;

                // We do this late to ensure all the section-existing checks are nearby and exist fast.
                let func_display = Some(func.display_internal(
                    context,
                    function_display_settings,
                    InternalSymDisplSettings::new(true),
                )?);

                (func_display, ro_syms_display, late_ro_syms_display)
            }
        };

        Ok(Self {
            func_display,
            ro_syms_display,
            late_ro_syms_display,
            settings,
        })
    }

    fn do_rodata_section(
        context: &'ctx Context,
        rodata_section: Option<&'rodata SectionData>,
        rodata_display_settings: &'rodata_settings SymDataDisplaySettings,
        rodata_indices: &[usize],
        late_rodata_indices: &[usize],
    ) -> Result<
        (
            Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>,
            Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>,
        ),
        PairingError,
    > {
        if rodata_indices.is_empty() && late_rodata_indices.is_empty() {
            // We only care if the rodata section exists if we do actually reference rodata symbols.
            return Ok((Vec::new(), Vec::new()));
        }

        let rodata_section = if let Some(rodata_section) = rodata_section {
            rodata_section
        } else {
            return Err(PairingError::MissingRodataSection {});
        };

        let ro_syms_display = Self::do_rodata_displays(
            context,
            rodata_section,
            rodata_display_settings,
            rodata_indices,
        )?;
        let late_ro_syms_display = Self::do_rodata_displays(
            context,
            rodata_section,
            rodata_display_settings,
            late_rodata_indices,
        )?;

        Ok((ro_syms_display, late_ro_syms_display))
    }

    fn do_rodata_displays(
        context: &'ctx Context,
        rodata_section: &'rodata SectionData,
        rodata_display_settings: &'rodata_settings SymDataDisplaySettings,
        indices: &[usize],
    ) -> Result<Vec<SymDataDisplay<'ctx, 'rodata, 'rodata_settings>>, PairingError> {
        let rodata_syms = rodata_section.data_symbols();

        indices
            .iter()
            .map(|x| {
                let rodata = if let Some(rodata) = rodata_syms.get(*x) {
                    rodata
                } else {
                    return Err(PairingError::RodataOutOfBounds {
                        index: *x,
                        len: rodata_syms.len(),
                        section_name: rodata_section.name().into(),
                    });
                };
                Ok(rodata.display_internal(
                    context,
                    rodata_display_settings,
                    InternalSymDisplSettings::new(true),
                )?)
            })
            .collect()
    }
}

impl fmt::Display for FuncRodataPairingDisplay<'_, '_, '_, '_, '_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.ro_syms_display.is_empty() {
            let line_end = self.ro_syms_display[0].settings_common().line_end();

            write!(f, ".section {}", self.settings.section_label_rodata())?;

            // TODO: pyo3 hack for splat stuff
            #[cfg(feature = "pyo3")]
            {
                if self.func_display.is_none() {
                    write!(f, "{}", line_end)?;
                }
            }

            for sym_display in &self.ro_syms_display {
                write!(f, "{}", line_end)?;
                // TODO:
                // f.write(sym.disassemble(migrate=True, useGlobalLabel=True, isSplittedSymbol=True))
                write!(f, "{}", sym_display)?;
            }

            // TODO: pyo3 hack for splat stuff
            #[cfg(feature = "pyo3")]
            {
                if self.func_display.is_some() {
                    write!(f, "{}", line_end)?;
                }
            }
        }

        if !self.late_ro_syms_display.is_empty() {
            if let Some(func_display) = self.func_display {
                let line_end = self.late_ro_syms_display[0].settings_common().line_end();

                write!(
                    f,
                    ".section {}{}",
                    self.settings.section_label_late_rodata(),
                    line_end,
                )?;

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
                    line_end,
                    self.settings.section_label_text(),
                    line_end,
                )?;
            }

            // TODO:
            // f.write(self.function.disassemble(migrate=self.hasRodataSyms(), isSplittedSymbol=True))
            write!(f, "{}", func_display)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct FuncRodataPairingDisplaySettings<'text_label, 'ro_label, 'late_ro_label> {
    section_label_text: Option<&'text_label str>,
    section_label_rodata: Option<&'ro_label str>,
    section_label_late_rodata: Option<&'late_ro_label str>,
}

impl<'text_label, 'ro_label, 'late_ro_label>
    FuncRodataPairingDisplaySettings<'text_label, 'ro_label, 'late_ro_label>
{
    pub fn new(
        section_label_text: Option<&'text_label str>,
        section_label_rodata: Option<&'ro_label str>,
        section_label_late_rodata: Option<&'late_ro_label str>,
    ) -> Self {
        Self {
            section_label_text,
            section_label_rodata,
            section_label_late_rodata,
        }
    }

    pub fn section_label_text(&self) -> &'text_label str {
        self.section_label_text.unwrap_or(".text")
    }
    pub fn section_label_rodata(&self) -> &'ro_label str {
        self.section_label_rodata.unwrap_or(".rodata")
    }
    pub fn section_label_late_rodata(&self) -> &'late_ro_label str {
        self.section_label_late_rodata.unwrap_or(
            ".late_rodata
        ",
        )
    }
}
