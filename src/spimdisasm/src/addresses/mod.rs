/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub use rabbitizer::{vram::VramOffset, Vram};

mod address_range;
mod global_offset_table;
mod gp_value;
mod rom;
mod rom_vram_range;
mod size;

pub use address_range::AddressRange;
pub(crate) use global_offset_table::GotRequestedAddress;
pub use global_offset_table::{GlobalOffsetTable, GotGlobalEntry, GotLocalEntry};
pub use gp_value::GpValue;
pub use rom::Rom;
pub use rom_vram_range::RomVramRange;
pub use size::Size;
