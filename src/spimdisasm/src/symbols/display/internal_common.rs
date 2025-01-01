/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InternalSymDisplSettings {
    migrate: bool,
}

impl InternalSymDisplSettings {
    pub fn new(migrate: bool) -> Self {
        Self { migrate }
    }

    pub fn migrate(&self) -> bool {
        self.migrate
    }
}
