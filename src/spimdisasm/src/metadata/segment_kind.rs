/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::fmt;

use super::OwnerSegmentKind;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SegmentKind {
    Global(Arc<str>),
    Overlay(Arc<str>),
    Unknown,
}

impl SegmentKind {
    pub fn name(&self) -> Option<Arc<str>> {
        match self {
            Self::Global(name) => Some(Arc::clone(name)),
            Self::Overlay(name) => Some(Arc::clone(name)),
            Self::Unknown => None,
        }
    }

    pub fn write_verbose(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentKind::Global(name) => write!(f, "global segment '{name}'"),
            SegmentKind::Overlay(name) => write!(f, "overlay segment '{name}'"),
            SegmentKind::Unknown => write!(f, "the unknown segment"),
        }
    }
}

impl From<SegmentKind> for OwnerSegmentKind {
    fn from(value: SegmentKind) -> Self {
        match value {
            SegmentKind::Global(name) => Self::Global(name),
            SegmentKind::Overlay(name) => Self::Overlay(name),
            SegmentKind::Unknown => Self::Unknown,
        }
    }
}
