/* SPDX-FileCopyrightText: © 2025-2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GotGlobalEntry {
    initial: u32,
    sym_val: u32,
    undef_com_or_abs: bool,
    sym_name: Arc<str>,
}

impl GotGlobalEntry {
    #[must_use]
    pub fn new<T>(initial: u32, sym_val: u32, undef_com_or_abs: bool, sym_name: T) -> Self
    where
        T: Into<Arc<str>>,
    {
        Self::new_impl(initial, sym_val, undef_com_or_abs, sym_name.into())
    }

    #[must_use]
    const fn new_impl(
        initial: u32,
        sym_val: u32,
        undef_com_or_abs: bool,
        sym_name: Arc<str>,
    ) -> Self {
        Self {
            initial,
            sym_val,
            undef_com_or_abs,
            sym_name,
        }
    }

    #[must_use]
    pub const fn initial(&self) -> u32 {
        self.initial
    }

    #[must_use]
    pub const fn undef_com_or_abs(&self) -> bool {
        self.undef_com_or_abs
    }

    #[must_use]
    pub fn sym_name(&self) -> Arc<str> {
        self.sym_name.clone()
    }

    #[must_use]
    pub const fn address(&self) -> u32 {
        if self.sym_val != 0 && !self.undef_com_or_abs {
            self.sym_val
        } else {
            self.initial
        }
    }
}
