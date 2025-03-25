/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum AddGlobalOffsetTableErrorVariant {
    AlreadyAdded,
    NotPic,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct AddGlobalOffsetTableError {
    variant: AddGlobalOffsetTableErrorVariant,
}
impl AddGlobalOffsetTableError {
    fn new(variant: AddGlobalOffsetTableErrorVariant) -> Self {
        Self { variant }
    }

    pub(crate) fn new_already_added() -> Self {
        Self::new(AddGlobalOffsetTableErrorVariant::AlreadyAdded)
    }

    pub(crate) fn new_not_pic() -> Self {
        Self::new(AddGlobalOffsetTableErrorVariant::NotPic)
    }
}

impl fmt::Display for AddGlobalOffsetTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while trying to add the Global Offset Table (GOT): ",
        )?;

        match self.variant {
            AddGlobalOffsetTableErrorVariant::AlreadyAdded => {
                write!(f, "A GOT has been added already.")
            }
            AddGlobalOffsetTableErrorVariant::NotPic => {
                write!(f, "Global configuration has not been configured for Position Independent Code (PIC).")
            }
        }
    }
}
impl error::Error for AddGlobalOffsetTableError {}
