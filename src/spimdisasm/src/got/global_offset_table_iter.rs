/* SPDX-FileCopyrightText: © 2025-2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use super::{GlobalOffsetTable, GotRequestedAddress};

use crate::addresses::{Size, Vram};

#[must_use]
pub struct GlobalOffsetTableIter<'got> {
    current_vram: Vram,
    got: &'got GlobalOffsetTable,
}

impl<'got> GlobalOffsetTableIter<'got> {
    pub(super) fn new(got: &'got GlobalOffsetTable) -> Self {
        Self {
            current_vram: got.vram().start(),
            got,
        }
    }
}

impl<'got> Iterator for GlobalOffsetTableIter<'got> {
    type Item = (Vram, GotRequestedAddress<'got>);

    fn next(&mut self) -> Option<Self::Item> {
        let current_vram = self.current_vram;

        if let Some(requested_address) = self.got.request_address(current_vram) {
            self.current_vram += Size::new(4);
            Some((current_vram, requested_address))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = (self.got.vram().end() - self.current_vram).inner() / 4;
        let left = left.max(0) as usize;

        (left, Some(left))
    }
}
