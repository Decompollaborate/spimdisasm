/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::metadata::SymbolNameGenerationSettings;

use super::{Endian, GpConfig, MacroLabels};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GlobalConfig {
    endian: Endian,
    gp_config: Option<GpConfig>,

    macro_labels: Option<MacroLabels>,
    // TODO: Maybe move to each section's disassembly settings
    emit_size_directive: bool,
    // ASM_USE_PRELUDE: bool = True
    // ASM_PRELUDE_USE_INCLUDES: bool = True
    // ASM_PRELUDE_USE_INSTRUCTION_DIRECTIVES: bool = True
    // ASM_PRELUDE_USE_SECTION_START: bool = True
    // ASM_GENERATED_BY: bool = True
    symbol_name_generation_settings: SymbolNameGenerationSettings,
}

impl GlobalConfig {
    pub const fn endian(&self) -> Endian {
        self.endian
    }

    pub const fn gp_config(&self) -> Option<&GpConfig> {
        self.gp_config.as_ref()
    }

    pub const fn macro_labels(&self) -> Option<&MacroLabels> {
        self.macro_labels.as_ref()
    }

    pub const fn emit_size_directive(&self) -> bool {
        self.emit_size_directive
    }

    pub const fn symbol_name_generation_settings(&self) -> &SymbolNameGenerationSettings {
        &self.symbol_name_generation_settings
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GlobalConfigBuilder {
    endian: Endian,
    gp_config: Option<GpConfig>,

    macro_labels: Option<MacroLabels>,
    // TODO: Maybe move to each section's disassembly settings
    emit_size_directive: bool,
    // ASM_USE_PRELUDE: bool = True
    // ASM_PRELUDE_USE_INCLUDES: bool = True
    // ASM_PRELUDE_USE_INSTRUCTION_DIRECTIVES: bool = True
    // ASM_PRELUDE_USE_SECTION_START: bool = True
    // ASM_GENERATED_BY: bool = True
    symbol_name_generation_settings: SymbolNameGenerationSettings,
}

impl GlobalConfigBuilder {
    pub fn new(endian: Endian) -> Self {
        Self {
            endian,
            gp_config: None,

            macro_labels: Some(MacroLabels::new()),
            emit_size_directive: true,

            symbol_name_generation_settings: SymbolNameGenerationSettings::new(),
        }
    }

    pub fn build(self) -> GlobalConfig {
        let Self {
            endian,
            gp_config,
            macro_labels,
            emit_size_directive,
            symbol_name_generation_settings,
        } = self;

        GlobalConfig {
            endian,
            gp_config,
            macro_labels,
            emit_size_directive,
            symbol_name_generation_settings,
        }
    }
}

impl GlobalConfigBuilder {
    pub const fn endian(&self) -> Endian {
        self.endian
    }

    pub const fn gp_config(&self) -> Option<&GpConfig> {
        self.gp_config.as_ref()
    }
    pub fn gp_config_mut(&mut self) -> &mut Option<GpConfig> {
        &mut self.gp_config
    }
    pub fn with_gp_config(self, gp_config: Option<GpConfig>) -> Self {
        Self { gp_config, ..self }
    }

    pub const fn macro_labels(&self) -> Option<&MacroLabels> {
        self.macro_labels.as_ref()
    }
    pub fn macro_labels_mut(&mut self) -> &mut Option<MacroLabels> {
        &mut self.macro_labels
    }
    pub fn with_macro_labels(self, macro_labels: Option<MacroLabels>) -> Self {
        Self {
            macro_labels,
            ..self
        }
    }

    pub const fn emit_size_directive(&self) -> bool {
        self.emit_size_directive
    }
    pub fn emit_size_directive_mut(&mut self) -> &mut bool {
        &mut self.emit_size_directive
    }
    pub fn with_emit_size_directive(self, emit_size_directive: bool) -> Self {
        Self {
            emit_size_directive,
            ..self
        }
    }

    pub const fn symbol_name_generation_settings(&self) -> &SymbolNameGenerationSettings {
        &self.symbol_name_generation_settings
    }
    pub fn symbol_name_generation_settings_mut(&mut self) -> &mut SymbolNameGenerationSettings {
        &mut self.symbol_name_generation_settings
    }
    pub fn with_symbol_name_generation_settings(
        self,
        symbol_name_generation_settings: SymbolNameGenerationSettings,
    ) -> Self {
        Self {
            symbol_name_generation_settings,
            ..self
        }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl GlobalConfigBuilder {
        #[new]
        pub fn py_new(endian: Endian) -> Self {
            let mut myself = Self::new(endian);

            myself
                .symbol_name_generation_settings_mut()
                .set_use_type_prefix(false);

            myself
        }

        #[pyo3(name = "build")]
        pub fn py_build(&self) -> GlobalConfig {
            self.clone().build()
        }

        #[pyo3(name = "set_gp_config")]
        pub fn py_set_gp_config(&mut self, gp_config: GpConfig) {
            self.gp_config = Some(gp_config);
        }

        #[pyo3(name = "set_macro_labels", signature=(macro_labels))]
        pub fn py_set_macro_labels(&mut self, macro_labels: Option<MacroLabels>) {
            self.macro_labels = macro_labels;
        }

        #[pyo3(name = "set_emit_size_directive")]
        pub fn py_set_emit_size_directive(&mut self, emit_size_directive: bool) {
            self.emit_size_directive = emit_size_directive;
        }
    }
}
