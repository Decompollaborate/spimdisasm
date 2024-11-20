/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::metadata::SymbolMetadata;

use super::GlobalConfig;

pub struct Context {
    global_config: GlobalConfig,
}

impl Context {
    pub fn new(global_config: GlobalConfig) -> Self {
        Self {
            global_config,
        }
    }
}

impl Context {
    pub const fn global_config(&self) -> &GlobalConfig {
        &self.global_config
    }
}

impl Context {
    pub fn find_symbol(&self) -> Option<&SymbolMetadata> {
        // TODO
        None
    }

    //pub fn find_symbol_mut(&self) -> Option<&mut SymbolMetadata> {
    //    // TODO
    //    None
    //}
}
