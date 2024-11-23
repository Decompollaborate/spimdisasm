/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

#[derive(Clone, Copy)]
pub enum InputEndian {
    Big,
    Little,
}

impl InputEndian {
    pub fn word_from_bytes(self, bytes: &[u8]) -> u32 {
        let x = bytes.try_into().expect("Wrong input");

        match self {
            InputEndian::Big => u32::from_be_bytes(x),
            InputEndian::Little => u32::from_le_bytes(x),
        }
    }
}

pub struct GlobalConfig {
    endian: InputEndian,
}

impl GlobalConfig {
    pub const fn default() -> Self {
        Self {
            endian: InputEndian::Big,
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl GlobalConfig {
    pub const fn endian(&self) -> InputEndian {
        self.endian
    }
    pub const fn with_endian(self, endian: InputEndian) -> Self {
        Self { endian, ..self }
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::default()
    }
}
