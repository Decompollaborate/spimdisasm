/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod overlay_category;
mod overlay_category_name;
pub(crate) mod segment_metadata;
mod symbol_metadata;
mod symbol_metadata_name_display;
mod symbol_type;

pub use overlay_category::OverlayCategory;
pub use overlay_category_name::OverlayCategoryName;
pub use segment_metadata::SegmentMetadata;
pub use symbol_metadata::{GeneratedBy, RodataMigrationBehavior, SymbolMetadata};
pub use symbol_metadata_name_display::SymbolMetadataNameDisplay;
pub use symbol_type::SymbolType;
