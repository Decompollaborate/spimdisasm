/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::addresses::Rom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LoPairingInfo {
    pub(crate) instr_rom: Rom,
    pub(crate) value: i64, // TODO: This is fishy
    pub(crate) is_gp_rel: bool,
    pub(crate) is_gp_got: bool,
    pub(crate) upper_imm: Option<i64>,
}
