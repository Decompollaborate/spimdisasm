/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, num::NonZeroU32, ops};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::collections::addended_ordered_map::SizedValue;

use super::{Rom, Size, Vram};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UserSize {
    inner: NonZeroU32,
}

impl UserSize {
    #[must_use]
    pub const fn new(value: NonZeroU32) -> Self {
        Self { inner: value }
    }

    #[must_use]
    pub const fn new_checked(value: u32) -> Option<Self> {
        Self::new_option(NonZeroU32::new(value))
    }

    #[must_use]
    pub const fn new_option(value: Option<NonZeroU32>) -> Option<Self> {
        match value {
            Some(x) => Some(Self::new(x)),
            None => None,
        }
    }

    #[must_use]
    pub const fn inner(&self) -> NonZeroU32 {
        self.inner
    }

    #[must_use]
    pub const fn add_user_size(&self, rhs: &Self) -> Self {
        let slf = self.inner().get();
        let temp = slf.checked_add(rhs.inner().get()).unwrap();

        Self::new(NonZeroU32::new(temp).unwrap())
    }

    #[must_use]
    pub const fn add_size(&self, rhs: &Size) -> Self {
        let slf = self.inner().get();
        let temp = slf.checked_add(rhs.inner()).unwrap();

        Self::new(NonZeroU32::new(temp).unwrap())
    }

    #[must_use]
    pub const fn add_vram(&self, rhs: &Vram) -> Vram {
        let slf = self.inner().get();
        let temp = slf.checked_add(rhs.inner()).unwrap();

        Vram::new(temp)
    }

    #[must_use]
    pub const fn add_rom(&self, rhs: &Rom) -> Rom {
        let slf = self.inner().get();
        let temp = slf.checked_add(rhs.inner()).unwrap();

        Rom::new(temp)
    }
}

impl ops::Add<UserSize> for UserSize {
    type Output = UserSize;

    fn add(self, rhs: UserSize) -> Self::Output {
        self.add_user_size(&rhs)
    }
}
impl ops::AddAssign for UserSize {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl ops::Add<Size> for UserSize {
    type Output = UserSize;

    fn add(self, rhs: Size) -> Self::Output {
        self.add_size(&rhs)
    }
}
impl ops::Add<UserSize> for Size {
    type Output = UserSize;

    fn add(self, rhs: UserSize) -> Self::Output {
        rhs.add_size(&self)
    }
}
impl ops::AddAssign<Size> for UserSize {
    fn add_assign(&mut self, rhs: Size) {
        *self = *self + rhs
    }
}

impl ops::Add<Vram> for UserSize {
    type Output = Vram;

    fn add(self, rhs: Vram) -> Self::Output {
        self.add_vram(&rhs)
    }
}
impl ops::Add<UserSize> for Vram {
    type Output = Vram;

    fn add(self, rhs: UserSize) -> Self::Output {
        rhs.add_vram(&self)
    }
}
impl ops::AddAssign<UserSize> for Vram {
    fn add_assign(&mut self, rhs: UserSize) {
        *self = *self + rhs
    }
}

impl ops::Add<Rom> for UserSize {
    type Output = Rom;

    fn add(self, rhs: Rom) -> Self::Output {
        self.add_rom(&rhs)
    }
}

impl ops::Add<UserSize> for Rom {
    type Output = Rom;

    fn add(self, rhs: UserSize) -> Self::Output {
        rhs.add_rom(&self)
    }
}
impl ops::AddAssign<UserSize> for Rom {
    fn add_assign(&mut self, rhs: UserSize) {
        *self = *self + rhs
    }
}

impl fmt::Debug for UserSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserSize {{ 0x{:02X} }}", self.inner)
    }
}
impl fmt::Display for UserSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02X}", self.inner)
    }
}

impl From<UserSize> for Size {
    fn from(value: UserSize) -> Self {
        Self::new(value.inner.get())
    }
}

impl From<Option<UserSize>> for Size {
    fn from(value: Option<UserSize>) -> Self {
        let val = match value {
            Some(x) => x.inner().get(),
            None => 0,
        };
        Self::new(val)
    }
}

impl SizedValue<Size> for UserSize {
    fn size(&self) -> Size {
        Size::from(*self)
    }
}

impl SizedValue<UserSize> for Option<UserSize> {
    fn size(&self) -> UserSize {
        self.unwrap_or(const { UserSize::new_checked(1).unwrap() })
    }
}
impl SizedValue<Size> for Option<UserSize> {
    fn size(&self) -> Size {
        self.unwrap_or(const { UserSize::new_checked(1).unwrap() })
            .into()
    }
}

impl SizedValue<UserSize> for (Vram, UserSize) {
    fn size(&self) -> UserSize {
        self.1
    }
}
impl SizedValue<Size> for (Vram, UserSize) {
    fn size(&self) -> Size {
        self.1.into()
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl UserSize {
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
