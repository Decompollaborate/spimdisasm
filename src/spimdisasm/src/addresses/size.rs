/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt, ops};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::collections::addended_ordered_map::SizedValue;

use super::{Rom, Vram, VramOffset};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct Size {
    inner: u32,
}

impl Size {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { inner: value }
    }

    #[must_use]
    pub const fn inner(&self) -> u32 {
        self.inner
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner == 0
    }

    #[must_use]
    pub const fn add_size(&self, rhs: &Self) -> Self {
        Self::new(self.inner() + rhs.inner())
    }

    #[must_use]
    pub const fn add_vram(&self, rhs: &Vram) -> Vram {
        Vram::new(self.inner() + rhs.inner())
    }

    #[must_use]
    pub const fn add_rom(&self, rhs: &Rom) -> Rom {
        Rom::new(self.inner() + rhs.inner())
    }
}

impl ops::Add<Size> for Size {
    type Output = Size;

    fn add(self, rhs: Size) -> Self::Output {
        self.add_size(&rhs)
    }
}
impl ops::AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
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
impl ops::AddAssign<Size> for Vram {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs
    }
}

impl ops::Add<Rom> for Size {
    type Output = Rom;

    fn add(self, rhs: Rom) -> Self::Output {
        self.add_rom(&rhs)
    }
}

impl ops::Add<Size> for Rom {
    type Output = Rom;

    fn add(self, rhs: Size) -> Self::Output {
        rhs.add_rom(&self)
    }
}
impl ops::AddAssign<Size> for Rom {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Size {{ 0x{:02X} }}", self.inner)
    }
}
impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.inner)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ConvertToSizeError {
    inner: i32,
}
impl fmt::Display for ConvertToSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Can't convert negative value {} (-0x{:X}) to `Size`.",
            self.inner, -self.inner
        )
    }
}
impl error::Error for ConvertToSizeError {}

impl TryFrom<VramOffset> for Size {
    type Error = ConvertToSizeError;

    fn try_from(value: VramOffset) -> Result<Self, Self::Error> {
        if value.inner() < 0 {
            Err(ConvertToSizeError {
                inner: value.inner(),
            })
        } else {
            Ok(Self::new(value.inner() as u32))
        }
    }
}

impl SizedValue for Size {
    fn size(&self) -> Size {
        *self
    }
}

impl SizedValue for Option<Size> {
    fn size(&self) -> Size {
        self.unwrap_or(Size::new(1))
    }
}

impl SizedValue for (Vram, Size) {
    fn size(&self) -> Size {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn conversion_error_from_vram_offset() {
        let a = Vram::new(0x80000010);
        let b = Vram::new(0x80000200);
        let diff = a - b;

        Size::try_from(diff).unwrap();
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl Size {
        #[new]
        pub fn py_new(value: u32) -> Self {
            Self::new(value)
        }

        #[pyo3(name = "inner")]
        pub fn py_inner(&self) -> u32 {
            self.inner()
        }
    }
}
