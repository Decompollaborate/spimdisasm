/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod context;
pub mod context_builder;

pub use context::{Context, OwnedSegmentNotFoundError};
pub use context_builder::ContextBuilder;
