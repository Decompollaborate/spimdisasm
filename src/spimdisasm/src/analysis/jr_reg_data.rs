/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::addresses::Rom;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct JrRegData {
    lo_rom: Rom,
    address: u32,
    branch_info: Option<Rom>,
    added_with_gp: Option<Rom>,
}

impl JrRegData {
    #[must_use]
    pub(crate) const fn new(
        lo_rom: Rom,
        address: u32,
        branch_info: Option<Rom>,
        added_with_gp: Option<Rom>,
    ) -> Self {
        Self {
            lo_rom,
            address,
            branch_info,
            added_with_gp,
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
    #[must_use]
    pub(crate) fn added_with_gp(&self) -> Option<&Rom> {
        self.added_with_gp.as_ref()
    }
}
