/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrackedRegisterState {}

impl TrackedRegisterState {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl TrackedRegisterState {
    pub fn clear(&mut self) {}
}
