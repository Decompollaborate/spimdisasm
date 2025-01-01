/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use super::{Endian, GpConfig, MacroLabels};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct GlobalConfig {
    endian: Endian,
    gp_config: Option<GpConfig>,

    macro_labels: Option<MacroLabels>,
    emit_size_directive: bool,
    // ASM_USE_PRELUDE: bool = True
    // ASM_PRELUDE_USE_INCLUDES: bool = True
    // ASM_PRELUDE_USE_INSTRUCTION_DIRECTIVES: bool = True
    // ASM_PRELUDE_USE_SECTION_START: bool = True
    // ASM_GENERATED_BY: bool = True
}

impl GlobalConfig {
    pub fn new(endian: Endian) -> Self {
        Self {
            endian,
            gp_config: None,

            macro_labels: Some(MacroLabels::new()),
            emit_size_directive: true,
        }
    }
}

impl GlobalConfig {
    pub const fn endian(&self) -> Endian {
        self.endian
    }
    pub fn endian_mut(&mut self) -> &mut Endian {
        &mut self.endian
    }
    pub fn with_endian(self, endian: Endian) -> Self {
        Self { endian, ..self }
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
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl GlobalConfig {
        #[new]
        pub fn py_new(endian: Endian) -> Self {
            Self::new(endian)
        }
    }
}
