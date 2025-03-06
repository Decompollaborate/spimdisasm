/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use core::hash::Hash;

use crate::{addresses::Vram, metadata::LabelType};

#[derive(Debug, Clone)]
pub struct ReferencedLabel {
    user_declared: bool,
    vram: Vram,
    autodetected_type: LabelType,
    referenced_by: Vec<Vram>,

    user_declared_type: Option<LabelType>,
}

impl ReferencedLabel {
    pub(crate) fn new(vram: Vram, label_type: LabelType) -> Self {
        Self {
            user_declared: false,
            vram,
            autodetected_type: label_type,
            referenced_by: Vec::new(),

            user_declared_type: None,
        }
    }

    pub(crate) fn new_user_declared(vram: Vram, label_type: LabelType) -> Self {
        Self {
            user_declared: true,
            user_declared_type: Some(label_type),
            ..Self::new(vram, label_type)
        }
    }

    pub const fn vram(&self) -> Vram {
        self.vram
    }
    pub const fn user_declared(&self) -> bool {
        self.user_declared
    }
    pub fn referenced_by(&self) -> &[Vram] {
        &self.referenced_by
    }

    pub fn label_type(&self) -> LabelType {
        self.user_declared_type.unwrap_or(self.autodetected_type)
    }

    pub fn reference_counter(&self) -> usize {
        self.referenced_by.len()
    }
}

impl ReferencedLabel {
    pub(crate) fn add_referenced_by(&mut self, specific_address: Vram) {
        self.referenced_by.push(specific_address);
    }

    pub(crate) fn set_autodetected_type(&mut self, new_type: LabelType) {
        if self.autodetected_type.does_new_takes_precedence(new_type) {
            self.autodetected_type = new_type;
        }
    }
}

impl PartialEq for ReferencedLabel {
    fn eq(&self, other: &Self) -> bool {
        self.vram == other.vram
    }
}
impl PartialOrd for ReferencedLabel {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.vram.partial_cmp(&other.vram)
    }
}
impl Hash for ReferencedLabel {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.vram.hash(state);
    }
}
