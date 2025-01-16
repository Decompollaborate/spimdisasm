/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![warn(clippy::ref_option)]
#![cfg_attr(feature = "nightly", feature(btree_cursors))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;
pub extern crate rabbitizer;

pub mod addresses;
pub mod analysis;
pub mod parent_segment_info;
pub mod section_type;
pub mod str_decoding;

pub mod config;
pub mod context;
pub mod metadata;
pub mod migration;
pub mod relocation;
pub mod sections;
pub mod symbols;

pub mod collections;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn spimdisasm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<addresses::Size>()?;
    m.add_class::<addresses::Rom>()?;
    m.add_class::<addresses::RomVramRange>()?;
    m.add_class::<addresses::Vram>()?;

    m.add_class::<metadata::SymbolType>()?;
    m.add_class::<metadata::RodataMigrationBehavior>()?;

    m.add_class::<config::Compiler>()?;
    m.add_class::<config::Endian>()?;
    m.add_class::<config::MacroLabels>()?;
    m.add_class::<config::GlobalConfig>()?;

    m.add_class::<analysis::StringGuesserLevel>()?;
    m.add_class::<str_decoding::Encoding>()?;

    m.add_class::<context::builder::AddUserSymbolError>()?;
    m.add_class::<context::builder::segment_builder::python_bindings::SymAttributes>()?;
    m.add_class::<context::builder::GlobalSegmentBuilder>()?;
    m.add_class::<context::builder::OverlaySegmentBuilder>()?;
    m.add_class::<context::ContextBuilder>()?;
    m.add_class::<context::Context>()?;

    m.add_class::<metadata::OverlayCategoryName>()?;
    m.add_class::<parent_segment_info::ParentSegmentInfo>()?;

    m.add_class::<sections::SectionExecutableSettings>()?;
    m.add_class::<sections::SectionDataSettings>()?;
    m.add_class::<sections::SectionNoloadSettings>()?;
    m.add_class::<sections::SectionExecutable>()?;
    m.add_class::<sections::SectionData>()?;
    m.add_class::<sections::SectionNoload>()?;

    m.add_class::<symbols::display::FunctionDisplaySettings>()?;
    m.add_class::<symbols::display::SymDataDisplaySettings>()?;
    m.add_class::<symbols::display::SymNoloadDisplaySettings>()?;

    m.add_class::<migration::FuncRodataPairing>()?;
    m.add_class::<migration::PairingError>()?;

    // rabbitizer types
    m.add_class::<rabbitizer::display_flags::InstructionDisplayFlags>()?;

    m.add_class::<rabbitizer::instr::InstructionFlags>()?;

    m.add_class::<rabbitizer::isa::IsaVersion>()?;
    m.add_class::<rabbitizer::isa::IsaExtension>()?;

    Ok(())
}
