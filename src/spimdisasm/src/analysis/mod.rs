/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

mod gpr_register_value;
mod instruction_analysis_result;
mod instruction_analyzer;
pub(crate) mod preheat;
mod preheat_error;
pub(crate) mod reference_wrapper;
mod referenced_address;
mod referenced_label;
mod register_tracker;
mod string_guesser;

pub(crate) use gpr_register_value::GprRegisterValue;
pub(crate) use instruction_analysis_result::{InstrAnalysisInfo, InstructionAnalysisResult};
pub(crate) use instruction_analyzer::InstructionAnalyzer;
pub(crate) use preheat::Preheater;
pub use preheat_error::PreheatError;
pub use reference_wrapper::ReferenceWrapper;
pub use referenced_address::ReferencedAddress;
pub use referenced_label::ReferencedLabel;
pub(crate) use register_tracker::{
    InstrOpJumptable, InstrOpLink, InstrOpPairedAddress, InstrOpRegisterOperation, InstrOpTailCall,
    InstructionOperation, RegisterTracker,
};
pub use string_guesser::StringGuesserFlags;
