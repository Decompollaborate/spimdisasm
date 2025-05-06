/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod compiler;
mod endian;
mod global_config;
mod gp_config;
mod macro_labels;

pub use compiler::Compiler;
pub use endian::Endian;
pub use global_config::{GlobalConfig, GlobalConfigBuilder};
pub use gp_config::GpConfig;
pub use macro_labels::MacroLabels;
