/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod overlay_category;
mod overlay_category_name;
mod parent_section_metadata;
mod platform_segment_metadata;
pub(crate) mod segment_metadata;
mod symbol_metadata;
mod symbol_metadata_name_display;
mod symbol_type;

pub use overlay_category::OverlayCategory;
pub use overlay_category_name::OverlayCategoryName;
pub(crate) use parent_section_metadata::ParentSectionMetadata;
pub(crate) use platform_segment_metadata::PlatformSegmentMetadata;
pub use segment_metadata::SegmentMetadata;
pub use symbol_metadata::{GeneratedBy, RodataMigrationBehavior, SymbolMetadata};
pub use symbol_metadata_name_display::SymbolMetadataNameDisplay;
pub use symbol_type::SymbolType;
