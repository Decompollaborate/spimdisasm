/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::rom_address::RomAddress;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolMetadata {

}

impl SymbolMetadata {
    pub fn is_trustable_function(&self) -> bool {
        // TODO
        true
    }
}

impl SymbolMetadata {
    pub fn rom(&self) -> Option<RomAddress> {
        // TODO
        None
    }

    pub fn user_declared_size(&self) -> Option<u32> {
        // TODO
        None
    }
}

impl SymbolMetadata {
    pub(crate) fn set_autocreated_from_other_sized_sym(&mut self) {
        // TODO
    }
}

