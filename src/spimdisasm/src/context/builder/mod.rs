/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod add_ignored_address_range_error;
mod add_user_segment_symbol_error;
mod add_user_symbol_error;
mod context_builder;
pub(crate) mod segment_builder;
mod segment_heater;
mod user_segment_builder;

pub use add_ignored_address_range_error::AddIgnoredAddressRangeError;
pub use add_user_segment_symbol_error::AddUserSegmentSymbolError;
pub use add_user_symbol_error::AddUserSymbolError;
pub use context_builder::ContextBuilder;
pub use segment_builder::{GlobalSegmentBuilder, OverlaySegmentBuilder};
pub(crate) use segment_heater::SegmentHeater;
pub use segment_heater::{GlobalSegmentHeater, OverlaySegmentHeater};
pub use user_segment_builder::UserSegmentBuilder;
