/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::fmt;

use crate::metadata::{OwnerSegmentKind, SegmentKind};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SegmentBuilderKind {
    Global(Arc<str>),
    Overlay(Arc<str>),
}

impl SegmentBuilderKind {
    pub fn name(&self) -> Arc<str> {
        match self {
            Self::Global(name) => Arc::clone(name),
            Self::Overlay(name) => Arc::clone(name),
        }
    }
    pub fn into_name(self) -> Arc<str> {
        match self {
            Self::Global(name) => name,
            Self::Overlay(name) => name,
        }
    }

    pub fn write_verbose(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SegmentBuilderKind::Global(name) => write!(f, "global segment '{name}'"),
            SegmentBuilderKind::Overlay(name) => write!(f, "overlay segment '{name}'"),
        }
    }
}

impl From<SegmentBuilderKind> for OwnerSegmentKind {
    fn from(value: SegmentBuilderKind) -> Self {
        match value {
            SegmentBuilderKind::Global(name) => Self::Global(name),
            SegmentBuilderKind::Overlay(name) => Self::Overlay(name),
        }
    }
}

impl From<SegmentBuilderKind> for SegmentKind {
    fn from(value: SegmentBuilderKind) -> Self {
        match value {
            SegmentBuilderKind::Global(name) => Self::Global(name),
            SegmentBuilderKind::Overlay(name) => Self::Overlay(name),
        }
    }
}
