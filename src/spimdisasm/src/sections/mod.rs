/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub mod preprocessed;
pub mod processed;
mod section_creation_error;
mod section_post_process_error;
mod trait_section;

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings;

pub use section_creation_error::SectionCreationError;
pub use section_post_process_error::SectionPostProcessError;
pub use trait_section::{
    RomSection, RomSectionPreprocessed, RomSectionProcessed, Section, SectionPreprocessed,
    SectionProcessed,
};
