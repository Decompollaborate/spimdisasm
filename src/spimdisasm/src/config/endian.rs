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
    #[must_use]
    pub const fn word_from_bytes(self, bytes: &[u8]) -> u32 {
        assert!(bytes.len() >= 4, "Not big enough");
        let arr = [bytes[0], bytes[1], bytes[2], bytes[3]];

        match self {
            Endian::Big => u32::from_be_bytes(arr),
            Endian::Little => u32::from_le_bytes(arr),
        }
    }
    #[must_use]
    pub const fn short_from_bytes(self, bytes: &[u8]) -> u16 {
        assert!(bytes.len() >= 2, "Not big enough");
        let arr = [bytes[0], bytes[1]];

        match self {
            Endian::Big => u16::from_be_bytes(arr),
            Endian::Little => u16::from_le_bytes(arr),
        }
    }
    #[must_use]
    pub const fn dword_from_bytes(self, bytes: &[u8]) -> u64 {
        assert!(bytes.len() >= 8, "Not big enough");
        let arr = [
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ];

        match self {
            Endian::Big => u64::from_be_bytes(arr),
            Endian::Little => u64::from_le_bytes(arr),
        }
    }

    #[must_use]
    pub const fn bytes_from_word(self, word: u32) -> [u8; 4] {
        match self {
            Endian::Big => word.to_be_bytes(),
            Endian::Little => word.to_le_bytes(),
        }
    }
}
