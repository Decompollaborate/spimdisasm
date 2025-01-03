/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::addresses::Rom;

pub struct JrRegData {
    lo_rom: Rom,
    address: u32,
    branch_info: Option<Rom>,
}

impl JrRegData {
    #[must_use]
    pub(crate) const fn new(lo_rom: Rom, address: u32, branch_info: Option<Rom>) -> Self {
        Self {
            lo_rom,
            address,
            branch_info,
        }
    }

    #[must_use]
    pub(crate) fn lo_rom(&self) -> Rom {
        self.lo_rom
    }
    #[must_use]
    pub(crate) fn address(&self) -> u32 {
        self.address
    }
    #[must_use]
    pub(crate) fn branch_info(&self) -> Option<&Rom> {
        self.branch_info.as_ref()
    }
}
