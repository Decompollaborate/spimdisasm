/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod context;
pub mod context_builder;
mod global_config;

pub use context::{Context, OwnedSegmentNotFoundError};
pub use context_builder::ContextBuilder;
pub use global_config::{GlobalConfig, InputEndian};
