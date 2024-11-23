/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::String;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OverlayCategoryName {
    inner: String,
}

impl OverlayCategoryName {
    pub const fn new(name: String) -> Self {
        Self { inner: name }
    }
}
