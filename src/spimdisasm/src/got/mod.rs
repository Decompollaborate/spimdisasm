/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod global_offset_table;
mod global_offset_table_iter;
mod got_global_entry;
mod got_local_entry;
mod got_requested_address;

pub use global_offset_table::GlobalOffsetTable;
pub(crate) use global_offset_table_iter::GlobalOffsetTableIter;
pub use got_global_entry::GotGlobalEntry;
pub use got_local_entry::GotLocalEntry;
pub(crate) use got_requested_address::GotRequestedAddress;
