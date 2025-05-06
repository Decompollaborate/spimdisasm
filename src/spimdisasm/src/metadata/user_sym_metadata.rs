/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use super::{GotAccessKind, RodataMigrationBehavior, SymbolMetadata, SymbolNameGenerationSettings};

pub struct UserSymMetadata<'seg> {
    sym: &'seg mut SymbolMetadata,
}

impl<'seg> UserSymMetadata<'seg> {
    pub(crate) fn new(sym: &'seg mut SymbolMetadata) -> Self {
        Self { sym }
    }
}

impl UserSymMetadata<'_> {
    #[must_use]
    pub fn symbol_name_generation_settings_mut(&mut self) -> &mut SymbolNameGenerationSettings {
        self.sym.symbol_name_generation_settings_mut()
    }

    pub fn set_user_declared_name_end<T>(&mut self, name: T)
    where
        T: Into<Arc<str>>,
    {
        self.sym.set_user_declared_name_end(name.into())
    }

    pub fn set_got_access_kind(&mut self, kind: GotAccessKind) {
        self.sym.set_got_access_kind(kind)
    }

    #[must_use]
    pub fn rodata_migration_behavior_mut(&mut self) -> &mut RodataMigrationBehavior {
        self.sym.rodata_migration_behavior_mut()
    }

    pub fn set_allow_ref_with_addend(&mut self, val: bool) {
        self.sym.set_allow_ref_with_addend(val);
    }

    pub fn set_visibility<T>(&mut self, visibility: T)
    where
        T: Into<Arc<str>>,
    {
        self.sym.set_visibility(visibility.into());
    }
}
