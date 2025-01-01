/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GpConfig {
    gp_value: Vram,
    pic: bool,
}

impl GpConfig {
    #[must_use]
    pub const fn new(gp_value: Vram, pic: bool) -> Self {
        Self { gp_value, pic }
    }

    #[must_use]
    pub const fn gp_value(&self) -> Vram {
        self.gp_value
    }

    #[must_use]
    pub const fn pic(&self) -> bool {
        self.pic
    }
}
