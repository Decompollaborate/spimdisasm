/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub(crate) mod func_rodata_migration;
mod pairing_display;
mod pairing_error;
mod rodata_iterator;

pub use func_rodata_migration::FuncRodataPairing;
pub use pairing_display::{FuncRodataPairingDisplay, FuncRodataPairingDisplaySettings};
pub use pairing_error::{
    FunctionOutOfBoundsError, MissingRodataSectionError, MissingTextSectionError, PairingError,
    RodataOutOfBoundsError,
};
pub use rodata_iterator::RodataIterator;
