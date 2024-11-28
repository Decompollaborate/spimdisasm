/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![cfg_attr(feature = "nightly", feature(btree_cursors))]
#![cfg_attr(not(feature = "std"), no_std)]

// #[macro_use]
extern crate alloc;
pub extern crate rabbitizer;

pub mod address_range;
pub mod analysis;
pub mod parent_segment_info;
pub mod rom_address;
pub mod section_type;
pub mod size;

pub mod config;
pub mod context;
pub mod metadata;
pub mod sections;
pub mod symbols;
