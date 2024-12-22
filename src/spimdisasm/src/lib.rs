/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![warn(clippy::ref_option)]
#![cfg_attr(feature = "nightly", feature(btree_cursors))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;
pub extern crate rabbitizer;

pub mod address_range;
pub mod analysis;
pub mod parent_segment_info;
pub mod rom_address;
pub mod rom_vram_range;
pub mod section_type;
pub mod size;
pub mod str_decoding;

pub mod config;
pub mod context;
pub mod metadata;
pub mod migration;
pub mod relocation;
pub mod sections;
pub mod symbols;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn spimdisasm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<size::Size>()?;
    m.add_class::<rom_address::RomAddress>()?;
    m.add_class::<rom_vram_range::RomVramRange>()?;

    m.add_class::<metadata::SymbolType>()?;

    m.add_class::<metadata::RodataMigrationBehavior>()?;

    m.add_class::<config::Compiler>()?;
    m.add_class::<config::Endian>()?;
    m.add_class::<config::GlobalConfig>()?;

    m.add_class::<analysis::StringGuesserLevel>()?;
    m.add_class::<str_decoding::Encoding>()?;

    m.add_class::<context::builder::UserSymbolOverlapError>()?;
    m.add_class::<context::builder::context_builder::python_bindings::SymAttributes>()?;
    m.add_class::<context::ContextBuilder>()?;
    // m.add_class::<context::ContextBuilderOverlay>()?;
    // m.add_class::<context::ContextBuilderFinderHeater>()?;
    // m.add_class::<context::ContextBuilderFinderHeaterOverlays>()?;
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

    Ok(())
}
