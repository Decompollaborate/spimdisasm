/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

use crate::{parent_segment_info::ParentSegmentInfo, size::Size};

pub trait Symbol {
    fn vram(&self) -> Vram;
    fn size(&self) -> Size;
    fn parent_segment_info(&self) -> &ParentSegmentInfo;
}
