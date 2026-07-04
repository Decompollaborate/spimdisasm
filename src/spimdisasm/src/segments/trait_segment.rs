/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};

use address_space::{AddressRange, RomVramRange, Vram};

use super::OverlayCategoryName;

pub trait Segment {
    type FindSettings: ?Sized;
    type Symbol: ?Sized;
    type Label: ?Sized;

    fn name(&self) -> Arc<str>;
    fn rom_vram_range(&self) -> &RomVramRange;

    fn prioritised_overlays(&self) -> &[Arc<str>];

    fn find_symbol(&self, vram: Vram, settings: &Self::FindSettings) -> Option<&Self::Symbol>;

    fn find_label(&self, vram: Vram, settings: &Self::FindSettings) -> Option<&Self::Label>;
}

pub trait AbsoluteSegment {
    type FindSettings: ?Sized;
    type Symbol: ?Sized;

    fn find_symbol(&self, vram: Vram, settings: &Self::FindSettings) -> Option<&Self::Symbol>;
}

pub trait SegBuilder {
    fn name(&self) -> Arc<str>;
    fn rom_vram_range(&self) -> &RomVramRange;

    fn prioritised_overlays(&self) -> &[Arc<str>];
}

pub trait GlobalSegBuilder: SegBuilder {
    type Finished: Segment;

    fn finish(self, visible_ranges: Vec<AddressRange<Vram>>) -> Self::Finished;
}

pub trait GlobalOvlBuilder: SegBuilder {
    type Finished: Segment;

    fn category_name(&self) -> OverlayCategoryName;

    fn finish(self, visible_ranges: Vec<AddressRange<Vram>>) -> Self::Finished;
}
