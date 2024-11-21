/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, ops};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddressRange<T>
{
    start: T,
    end: T,
}

impl<T> AddressRange<T>
where T: Copy + PartialOrd
{
    #[must_use]
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "An address range can't contain an `end` value that's smaller than the `start` one");
        Self {start, end}
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

impl<T, U> AddressRange<T>
where T: Copy + ops::Sub<Output = U>
{
    #[must_use]
    pub fn size(&self) -> U {
        self.end - self.start
    }
}

impl<T> AddressRange<T>
where T: Copy + PartialOrd
{
    #[must_use]
    pub fn in_range(&self, value: T) -> bool {
        self.start <= value && value < self.end
    }

    pub fn decrease_start(&mut self, value: T) {
        if value < self.start {
            self.start = value;
        }
    }

    pub fn increase_end(&mut self, value: T) {
        if value >= self.end {
            self.end = value;
        }
    }
}


impl<T> fmt::Display for AddressRange<T>
where T: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}, {}}}", self.start, self.end)
    }
}
