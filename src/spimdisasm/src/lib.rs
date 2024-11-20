/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

extern crate alloc;

pub mod rom_address;
pub mod metadata;
pub mod context;
pub mod sections;
pub mod symbols;
