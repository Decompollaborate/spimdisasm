/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub use rabbitizer::{vram::VramOffset, Vram};

mod address_range;
mod rom;
mod rom_vram_range;
mod size;

pub use address_range::AddressRange;
pub use rom::Rom;
pub use rom_vram_range::RomVramRange;
pub use size::Size;
