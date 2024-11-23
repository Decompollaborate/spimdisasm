/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt, ops};

use rabbitizer::{vram::VramOffset, Vram};

use crate::rom_address::RomAddress;

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size {
    inner: u32,
}

impl Size {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self {inner: value}
    }

    pub const fn inner(&self) -> u32 {
        self.inner
    }

    pub const fn add_vram(&self, rhs: &Vram) -> Vram {
        Vram::new(self.inner() + rhs.inner())
    }

    pub const fn add_rom(&self, rhs: &RomAddress) -> RomAddress {
        RomAddress::new(self.inner() + rhs.inner())
    }
}

impl ops::Add<Vram> for Size {
    type Output = Vram;

    fn add(self, rhs: Vram) -> Self::Output {
        self.add_vram(&rhs)
    }
}

impl ops::Add<Size> for Vram {
    type Output = Vram;

    fn add(self, rhs: Size) -> Self::Output {
        rhs.add_vram(&self)
    }
}

impl ops::Add<RomAddress> for Size {
    type Output = RomAddress;

    fn add(self, rhs: RomAddress) -> Self::Output {
        self.add_rom(&rhs)
    }
}

impl ops::Add<Size> for RomAddress {
    type Output = RomAddress;

    fn add(self, rhs: Size) -> Self::Output {
        rhs.add_rom(&self)
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Size {{ inner: 0x{:02X} }}", self.inner)
    }
}
impl fmt::UpperHex for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.inner)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ConvertToSizeError {
    inner: i32,
}
impl fmt::Display for ConvertToSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't convert negative value {} (-0x{:X}) to `Size`.", self.inner, -self.inner)
    }
}
impl error::Error for ConvertToSizeError {}

impl TryFrom<VramOffset> for Size {
    type Error = ConvertToSizeError;

    fn try_from(value: VramOffset) -> Result<Self, Self::Error> {
        if value.inner() < 0 {
            Err(ConvertToSizeError {inner: value.inner()})
        } else {
            Ok(Self::new(value.inner() as u32))
        }
    }
}

#[cfg(test)]
mod tests {
    use rabbitizer::Vram;

    use super::Size;

    #[test]
    #[should_panic]
    fn conversion_error_from_vram_offset() {
        let a = Vram::new(0x80000010);
        let b = Vram::new(0x80000200);
        let diff = a - b;

        Size::try_from(diff).unwrap();
    }
}
