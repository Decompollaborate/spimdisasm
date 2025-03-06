/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{Rom, Vram},
    parent_segment_info::ParentSegmentInfo,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReferencedInfo {
    Function {
        sym_vram: Vram,
        parent: ParentSegmentInfo,
        specific_rom: Rom,
    },
    Data {
        sym_vram: Vram,
        parent: ParentSegmentInfo,
        specific_rom: Rom,
    },
}
