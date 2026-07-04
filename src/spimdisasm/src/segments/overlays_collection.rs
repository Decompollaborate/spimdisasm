/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use address_space::Rom;

use super::{OverlayCategory, OverlayCategoryName};
use crate::collections::unordered_map::UnorderedMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlaysCollection<SEG> {
    inner: UnorderedMap<OverlayCategoryName, OverlayCategory<SEG>>,
}

impl<SEG> OverlaysCollection<SEG> {
    pub(crate) fn new(inner: UnorderedMap<OverlayCategoryName, OverlayCategory<SEG>>) -> Self {
        Self { inner }
    }

    pub(crate) fn inner(&self) -> &UnorderedMap<OverlayCategoryName, OverlayCategory<SEG>> {
        &self.inner
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get_segment(
        &self,
        category_name: &OverlayCategoryName,
        segment_rom: Rom,
    ) -> Option<&SEG> {
        self.inner
            .get(category_name)
            .and_then(|segments_per_rom| segments_per_rom.get(segment_rom))
    }

    pub fn get_segment_mut(
        &mut self,
        category_name: &OverlayCategoryName,
        segment_rom: Rom,
    ) -> Option<&mut SEG> {
        self.inner
            .get_mut(category_name)
            .and_then(|segments_per_rom| segments_per_rom.get_mut(segment_rom))
    }

    pub fn iter_segments(&self) -> impl Iterator<Item = (&OverlayCategoryName, &Rom, &SEG)> {
        self.inner
            .iter()
            .flat_map(|(category_name, segments_per_rom)| {
                segments_per_rom
                    .segments()
                    .iter()
                    .map(move |(rom, seg)| (category_name, rom, seg))
            })
    }

    pub fn iter_segments_mut(
        &mut self,
    ) -> impl Iterator<Item = (&OverlayCategoryName, &Rom, &mut SEG)> {
        self.inner
            .iter_mut()
            .flat_map(|(category_name, segments_per_rom)| {
                segments_per_rom
                    .segments_mut()
                    .iter_mut()
                    .map(move |(rom, seg)| (category_name, rom, seg))
            })
    }
}
