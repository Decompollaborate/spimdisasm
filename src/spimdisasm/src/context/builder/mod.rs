/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod absolute_segment_builder;
mod add_absolute_segment_symbol_error;
mod add_global_offset_table_error;
mod add_ignored_address_range_error;
mod add_user_label_error;
mod add_user_symbol_error;
mod context_builder;
mod context_builder_error;
pub(crate) mod segment_builder;
mod segment_builder_error;
mod segment_builder_kind;
mod segment_heater;

pub use absolute_segment_builder::AbsoluteSegmentBuilder;
pub use add_absolute_segment_symbol_error::AddAbsoluteSegmentSymbolError;
pub use add_global_offset_table_error::AddGlobalOffsetTableError;
pub use add_ignored_address_range_error::AddIgnoredAddressRangeError;
pub use add_user_label_error::AddUserLabelError;
pub use add_user_symbol_error::AddUserSymbolError;
pub use context_builder::ContextBuilder;
pub use context_builder_error::BuildContextError;
pub use segment_builder::{GlobalSegmentBuilder, OverlaySegmentBuilder};
pub use segment_builder_error::AddPrioritisedOverlayError;
pub(crate) use segment_builder_kind::SegmentBuilderKind;
pub use segment_heater::{GlobalSegmentHeater, OverlaySegmentHeater};
