/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod rom_address;
pub mod address_range;
pub mod section_type;

pub mod metadata;
pub mod context;
pub mod sections;
pub mod symbols;
