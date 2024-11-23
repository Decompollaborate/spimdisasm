/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#![cfg_attr(feature = "nightly", feature(btree_cursors))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
pub extern crate rabbitizer;

pub mod rom_address;
pub mod size;
pub mod address_range;
pub mod section_type;
pub mod parent_segment_info;

pub mod metadata;
pub mod context;
pub mod sections;
pub mod symbols;
