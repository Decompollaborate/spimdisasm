/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod section_creation_error;
mod section_data;
mod section_executable;
mod section_noload;
mod section_post_process_error;
mod trait_section;

pub use section_creation_error::SectionCreationError;
pub use section_data::{SectionData, SectionDataSettings};
pub use section_executable::{SectionExecutable, SectionExecutableSettings};
pub use section_noload::{SectionNoload, SectionNoloadSettings};
pub use section_post_process_error::SectionPostProcessError;
pub use trait_section::{RomSection, Section};
