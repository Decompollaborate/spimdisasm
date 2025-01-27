/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum Endian {
    Big,
    Little,
}

impl Endian {
    pub fn word_from_bytes(self, bytes: &[u8]) -> u32 {
        let x = bytes.try_into().expect("Wrong input");

        match self {
            Endian::Big => u32::from_be_bytes(x),
            Endian::Little => u32::from_le_bytes(x),
        }
    }
    pub fn short_from_bytes(self, bytes: &[u8]) -> u16 {
        let x = bytes.try_into().expect("Wrong input");

        match self {
            Endian::Big => u16::from_be_bytes(x),
            Endian::Little => u16::from_le_bytes(x),
        }
    }
    pub fn dword_from_bytes(self, bytes: &[u8]) -> u64 {
        let x = bytes.try_into().expect("Wrong input");

        match self {
            Endian::Big => u64::from_be_bytes(x),
            Endian::Little => u64::from_le_bytes(x),
        }
    }

    pub fn bytes_from_word(self, word: u32) -> [u8; 4] {
        match self {
            Endian::Big => word.to_be_bytes(),
            Endian::Little => word.to_le_bytes(),
        }
    }
}
