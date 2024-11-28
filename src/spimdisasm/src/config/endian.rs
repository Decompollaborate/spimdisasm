/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
}
