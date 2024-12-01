/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::rom_address::RomAddress;

pub struct JrRegData {
    lo_rom: RomAddress,
    address: u32,
    branch_info: Option<RomAddress>,
}

impl JrRegData {
    #[must_use]
    pub(crate) const fn new(
        lo_rom: RomAddress,
        address: u32,
        branch_info: Option<RomAddress>,
    ) -> Self {
        Self {
            lo_rom,
            address,
            branch_info,
        }
    }

    #[must_use]
    pub(crate) fn lo_rom(&self) -> RomAddress {
        self.lo_rom
    }
    #[must_use]
    pub(crate) fn address(&self) -> u32 {
        self.address
    }
    #[must_use]
    pub(crate) fn branch_info(&self) -> Option<&RomAddress> {
        self.branch_info.as_ref()
    }
}
