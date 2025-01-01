/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::rom_address::RomAddress;

pub struct LoPairingInfo {
    pub(crate) instr_rom: RomAddress,
    pub(crate) value: i64, // TODO: This is fishy
    pub(crate) is_gp_rel: bool,
    pub(crate) is_gp_got: bool,
}
