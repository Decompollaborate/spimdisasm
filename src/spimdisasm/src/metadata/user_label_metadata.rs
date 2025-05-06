/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;

use super::LabelMetadata;

pub struct UserLabelMetadata<'seg> {
    label: &'seg mut LabelMetadata,
}

impl<'seg> UserLabelMetadata<'seg> {
    pub(crate) fn new(label: &'seg mut LabelMetadata) -> Self {
        Self { label }
    }
}

impl UserLabelMetadata<'_> {
    pub fn set_visibility<T>(&mut self, visibility: T)
    where
        T: Into<Arc<str>>,
    {
        self.label.set_visibility(visibility.into());
    }
}
