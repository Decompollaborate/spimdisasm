/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, ops};

use super::{Rom, Size, Vram};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressRange<T> {
    start: T,
    end: T,
}

impl<T> AddressRange<T>
where
    T: Copy + PartialOrd + fmt::Debug,
{
    #[must_use]
    pub fn new(start: T, end: T) -> Self {
        assert!(
            start <= end,
            "An address range can't contain an `end` value that's smaller than the `start` one. start: {:?}, end: {:?}", start, end
        );
        Self { start, end }
    }

    #[must_use]
    pub const fn start(&self) -> T {
        self.start
    }

    #[must_use]
    pub const fn end(&self) -> T {
        self.end
    }
}

impl AddressRange<Vram> {
    pub const fn size(&self) -> Size {
        // Casting to unsigned should be fine because we now `self.end` is always greater or equal than `self.start`.
        Size::new(self.end.sub_vram(&self.start).inner() as u32)
    }
}

impl AddressRange<Rom> {
    pub const fn size(&self) -> Size {
        // TODO: Add a substraction method on Rom
        Size::new(self.end.inner() - self.start.inner())
    }
}

impl<T> AddressRange<T>
where
    T: Copy + PartialOrd,
{
    #[must_use]
    pub fn in_range(&self, value: T) -> bool {
        self.start <= value && value < self.end
    }

    fn decrease_start(&mut self, value: T) {
        if value < self.start {
            self.start = value;
        }
    }
    fn increase_end(&mut self, value: T) {
        if value >= self.end {
            self.end = value;
        }
    }
    pub fn expand_range(&mut self, other: &AddressRange<T>) {
        self.decrease_start(other.start);
        self.increase_end(other.end);
    }

    #[must_use]
    pub fn overlaps(&self, other: &AddressRange<T>) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl<T> fmt::Display for AddressRange<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}, {}}}", self.start, self.end)
    }
}

impl ops::Index<AddressRange<Rom>> for [u8] {
    type Output = [u8];

    #[inline]
    fn index(&self, index: AddressRange<Rom>) -> &Self::Output {
        &self[index.start.inner() as usize..index.end.inner() as usize]
    }
}

impl<T> ops::RangeBounds<T> for AddressRange<T> {
    fn start_bound(&self) -> ops::Bound<&T> {
        ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> ops::Bound<&T> {
        ops::Bound::Excluded(&self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_range_overlaps_no() {
        let x = AddressRange::new(0, 0x10);
        let y = AddressRange::new(0x10, 0x20);

        assert_eq!(x.overlaps(&y), false);
        assert_eq!(y.overlaps(&x), false);
    }

    #[test]
    fn test_address_range_overlaps_embedded() {
        let x = AddressRange::new(0, 0x10);
        let y = AddressRange::new(0x4, 0x8);

        assert_eq!(x.overlaps(&y), true);
        assert_eq!(y.overlaps(&x), true);
    }

    #[test]
    fn test_address_range_overlaps_half() {
        let x = AddressRange::new(0x4, 0x10);
        let y = AddressRange::new(0x8, 0x18);

        assert_eq!(x.overlaps(&y), true);
        assert_eq!(y.overlaps(&x), true);

        let x = AddressRange::new(0x4, 0x10);
        let y = AddressRange::new(0x2, 0x8);

        assert_eq!(x.overlaps(&y), true);
        assert_eq!(y.overlaps(&x), true);
    }
}
