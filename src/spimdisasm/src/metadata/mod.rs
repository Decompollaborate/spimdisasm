/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod ignored_address_range;
mod overlay_category;
mod overlay_category_name;
mod parent_section_metadata;
pub(crate) mod segment_metadata;
mod symbol_metadata;
mod symbol_metadata_name_display;
mod symbol_type;
mod user_segment_metadata;

pub(crate) use ignored_address_range::IgnoredAddressRange;
pub use overlay_category::OverlayCategory;
pub use overlay_category_name::OverlayCategoryName;
pub(crate) use parent_section_metadata::ParentSectionMetadata;
pub use segment_metadata::SegmentMetadata;
pub(crate) use symbol_metadata::OwnerSegmentKind;
pub use symbol_metadata::{GeneratedBy, RodataMigrationBehavior, SymbolMetadata};
pub use symbol_metadata_name_display::SymbolMetadataNameDisplay;
pub use symbol_type::SymbolType;
pub(crate) use user_segment_metadata::UserSegmentMetadata;
