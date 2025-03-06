/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod instruction_analysis_result;
mod instruction_analyzer;
mod jr_reg_data;
mod lo_pairing_info;
pub(crate) mod preheat;
mod preheat_error;
pub(crate) mod reference_wrapper;
mod referenced_address;
mod referenced_label;
mod register_tracker;
mod string_guesser;
mod tracked_register_state;

pub(crate) use instruction_analysis_result::GpSetInfo;
pub use instruction_analysis_result::InstructionAnalysisResult;
pub(crate) use instruction_analyzer::InstructionAnalyzer;
pub(crate) use jr_reg_data::JrRegData;
pub(crate) use lo_pairing_info::LoPairingInfo;
pub(crate) use preheat::Preheater;
pub use preheat_error::PreheatError;
pub use reference_wrapper::ReferenceWrapper;
pub use referenced_address::ReferencedAddress;
pub use referenced_label::ReferencedLabel;
pub(crate) use register_tracker::RegisterTracker;
pub use string_guesser::StringGuesserFlags;
pub(crate) use tracked_register_state::TrackedRegisterState;
