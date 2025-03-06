/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod add_ignored_address_range_error;
mod add_user_label_error;
mod add_user_segment_symbol_error;
mod add_user_symbol_error;
mod context_builder;
mod context_builder_error;
pub(crate) mod segment_builder;
mod segment_builder_error;
mod segment_heater;
mod user_segment_builder;

pub use add_ignored_address_range_error::AddIgnoredAddressRangeError;
pub use add_user_label_error::AddUserLabelError;
pub use add_user_segment_symbol_error::AddUserSegmentSymbolError;
pub use add_user_symbol_error::AddUserSymbolError;
pub use context_builder::ContextBuilder;
pub use context_builder_error::{AddOverlayToBuilderError, BuildContextError};
pub use segment_builder::{GlobalSegmentBuilder, OverlaySegmentBuilder};
pub use segment_builder_error::AddPrioritisedOverlayError;
pub(crate) use segment_heater::SegmentHeater;
pub use segment_heater::{GlobalSegmentHeater, OverlaySegmentHeater};
pub use user_segment_builder::UserSegmentBuilder;
