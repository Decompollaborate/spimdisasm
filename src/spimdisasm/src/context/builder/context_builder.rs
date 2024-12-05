/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{config::GlobalConfig, metadata::SegmentMetadata, rom_vram_range::RomVramRange};

use super::{ContextBuilderOverlay, SegmentModifier};

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct ContextBuilder {
    global_config: GlobalConfig,

    global_segment: SegmentMetadata,
}

impl ContextBuilder {
    #[must_use]
    pub fn new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
        let global_segment = SegmentMetadata::new(global_ranges, None);

        Self {
            global_config,
            global_segment,
        }
    }

    #[must_use]
    pub fn global_segment(&mut self) -> SegmentModifier {
        SegmentModifier::new(&mut self.global_segment)
    }

    #[must_use]
    pub fn process(self) -> ContextBuilderOverlay {
        ContextBuilderOverlay::new(self.global_config, self.global_segment)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use rabbitizer::Vram;

    use crate::rom_address::RomAddress;

    use super::*;

    #[pymethods]
    impl ContextBuilder {
        #[new]
        fn py_new(global_config: GlobalConfig, global_ranges: RomVramRange) -> Self {
            Self::new(global_config, global_ranges)
        }

        // TODO: add a way to add symbols
        // #[pyo3(name = "global_segment")]
        // pub fn py_global_segment(&mut self) -> SegmentModifier {
        //     self.global_segment()
        // }
        #[pyo3(signature = (name, vram, rom))]
        pub fn add_symbol(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment().add_symbol(name, Vram::new(vram), rom);
        }

        #[pyo3(signature = (name, vram, rom))]
        pub fn add_function(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment()
                .add_function(name, Vram::new(vram), rom);
        }

        #[pyo3(signature = (name, vram, rom))]
        pub fn add_branch_label(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment()
                .add_branch_label(name, Vram::new(vram), rom);
        }

        #[pyo3(signature = (name, vram, rom))]
        pub fn add_jumptable(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment()
                .add_jumptable(name, Vram::new(vram), rom);
        }

        #[pyo3(signature = (name, vram, rom))]
        pub fn add_jumptable_label(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment()
                .add_jumptable_label(name, Vram::new(vram), rom);
        }

        #[pyo3(signature = (name, vram, rom))]
        pub fn add_gcc_except_table(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment()
                .add_gcc_except_table(name, Vram::new(vram), rom);
        }

        #[pyo3(signature = (name, vram, rom))]
        pub fn add_gcc_except_table_label(
            &mut self,
            name: String,
            vram: u32, // Vram // TODO
            rom: Option<RomAddress>,
        ) {
            self.global_segment()
                .add_gcc_except_table_label(name, Vram::new(vram), rom);
        }

        #[pyo3(name = "process")]
        pub fn py_process(&self) -> ContextBuilderOverlay {
            // Silly clone because we can't move from a Python instance
            self.clone().process()
        }
    }
}
