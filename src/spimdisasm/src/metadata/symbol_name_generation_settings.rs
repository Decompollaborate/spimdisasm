/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

// TODO: Ideally we wouldn't need to clone this struct all over the place

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolNameGenerationSettings {
    use_type_prefix: bool,
    custom_suffix: Option<Arc<str>>,
}

impl SymbolNameGenerationSettings {
    #[expect(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            use_type_prefix: true,
            custom_suffix: None,
        }
    }

    pub const fn use_type_prefix(&self) -> bool {
        self.use_type_prefix
    }
    pub const fn set_use_type_prefix(&mut self, use_type_prefix: bool) {
        self.use_type_prefix = use_type_prefix;
    }
    pub fn with_use_type_prefix(self, use_type_prefix: bool) -> Self {
        Self {
            use_type_prefix,
            ..self
        }
    }

    pub fn custom_suffix(&self) -> Option<Arc<str>> {
        self.custom_suffix.clone()
    }
    // pub fn set_custom_suffix(&mut self, custom_suffix: Option<Arc<str>>) {
    //     self.custom_suffix = custom_suffix;
    // }
    pub fn set_custom_suffix<T>(&mut self, custom_suffix: Option<T>)
    where
        T: Into<Arc<str>>,
    {
        self.custom_suffix = custom_suffix.map(|x| x.into());
    }
    pub fn with_custom_suffix(self, custom_suffix: Option<Arc<str>>) -> Self {
        Self {
            custom_suffix,
            ..self
        }
    }
}
