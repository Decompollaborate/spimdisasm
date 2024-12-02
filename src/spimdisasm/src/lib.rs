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

pub mod config;
pub mod context;
pub mod metadata;
pub mod relocation;
pub mod sections;
pub mod symbols;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn spimdisasm(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<rom_vram_range::RomVramRange>()?;

    m.add_class::<config::Endian>()?;
    m.add_class::<config::GlobalConfig>()?;

    m.add_class::<context::ContextBuilder>()?;
    // m.add_class::<context::ContextBuilderOverlay>()?;
    // m.add_class::<context::ContextBuilderFinderHeater>()?;
    // m.add_class::<context::ContextBuilderFinderHeaterOverlays>()?;
    m.add_class::<context::Context>()?;
    Ok(())
}
