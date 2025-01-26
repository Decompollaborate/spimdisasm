/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub mod display;
pub mod preprocessed;
pub mod processed;
mod symbol_creation_error;
mod symbol_post_process_error;
mod trait_symbol;

pub use symbol_creation_error::SymbolCreationError;
pub use symbol_post_process_error::{OwnedSymbolNotFoundError, SymbolPostProcessError};
pub use trait_symbol::{
    RomSymbol, RomSymbolPreprocessed, RomSymbolProcessed, Symbol, SymbolPreprocessed,
    SymbolProcessed,
};
