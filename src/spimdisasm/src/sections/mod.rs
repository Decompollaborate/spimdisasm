/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod section_data;
mod section_rodata;
mod section_text;
mod trait_section;

pub use section_data::{SectionData, SectionDataSettings};
pub use section_rodata::SectionRodata;
pub use section_text::{SectionText, SectionTextSettings};
pub use trait_section::{RomSection, Section};
