/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod add_global_to_builder_error;
mod add_overlay_to_builder_error;
mod build_segments_collection_error;
mod overlay_category;
mod overlay_category_name;
mod overlays_collection;
mod owned_segment_not_found_error;
mod parent_segment_info;
mod segments_collection;
mod segments_collection_builder;
mod trait_segment;

pub use add_global_to_builder_error::AddGlobalToBuilderError;
pub use add_overlay_to_builder_error::AddOverlayToBuilderError;
pub use build_segments_collection_error::BuildSegmentsCollectionError;
pub use overlay_category::OverlayCategory;
pub use overlay_category_name::OverlayCategoryName;
pub use overlays_collection::OverlaysCollection;
pub use owned_segment_not_found_error::OwnedSegmentNotFoundError;
pub use parent_segment_info::ParentSegmentInfo;
pub use segments_collection::SegmentsCollection;
pub use segments_collection_builder::SegmentsCollectionBuilder;
pub use trait_segment::{AbsoluteSegment, GlobalOvlBuilder, GlobalSegBuilder, SegBuilder, Segment};
