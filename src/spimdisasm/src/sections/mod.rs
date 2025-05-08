/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub mod before_proc;
pub mod processed;
mod section_creation_error;
mod section_post_process_error;
mod trait_section;

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings;

pub use section_creation_error::{
    BadBytesSizeError, BadUserSymbolSizeError, EmptySectionError, RomVramAlignmentMismatchError,
    SectionAlreadyCreatedError, SectionCreationError, SectionNotPreheatedError, UnalignedRomError,
    UnalignedVramError,
};
pub use section_post_process_error::SectionPostProcessError;
pub use trait_section::{
    RomSection, RomSectionPreprocessed, RomSectionProcessed, Section, SectionPreprocessed,
    SectionProcessed,
};
