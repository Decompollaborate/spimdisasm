/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub mod context_builder;
mod the_context;

pub use context_builder::ContextBuilder;
pub use the_context::{Context, OwnedSegmentNotFoundError};
