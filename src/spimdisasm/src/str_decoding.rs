/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use alloc::{string::String, vec::Vec};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum Encoding {
    Ascii,
    ShiftJis,
    EucJp,
    EucCn,
}

// Escape characters that are unlikely to be used
static BANNED_ESCAPE_CHARACTERS: [u8; 26] = [
    0x00, // '\0'
    0x01, //
    0x02, //
    0x03, //
    0x04, //
    0x05, //
    0x06, //
    0x07, // '\a'
    0x08, // '\b'
    // 0x09, // '\t'
    // 0x0A, // '\n'
    0x0B, // '\v'
    // 0x0C, // '\f'
    // 0x0D, // '\r'
    0x0E, //
    0x0F, //
    0x10, //
    0x11, //
    0x12, //
    0x13, //
    0x14, //
    0x15, //
    0x16, //
    0x17, //
    0x18, //
    0x19, //
    // 0x1A, // Some games use this char as an escape sequence
    // 0x1B, // VT escape sequences
    0x1C, //
    0x1D, //
    0x1E, //
    0x1F, //
];

// TODO
#[allow(dead_code)]
static ESCAPE_CHARACTERS_SPECIAL_CASES: [u8; 4] = [0x1A, 0x1B, 0x8C, 0x8D];

impl Encoding {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ASCII" => Some(Encoding::Ascii),
            "SHIFT-JIS" => Some(Encoding::ShiftJis),
            "EUC-JP" => Some(Encoding::EucJp),
            "EUC-CN" | "GB2312" => Some(Encoding::EucCn),
            _ => None,
        }
    }

    pub const fn default() -> Self {
        Encoding::Ascii
    }

    const fn as_encoding_rs(&self) -> &'static encoding_rs::Encoding {
        match self {
            Encoding::Ascii => {
                // An extension of Latin1.
                // For some reason `encoding_rs` uses this encoding to decode ASCII instead of having a
                // dedicated ASCII encoding, so we just use the same.
                // Care must be taken to avoid decoding bytes outside the ASCII range (> 0x7F).
                encoding_rs::WINDOWS_1252
            }
            Encoding::ShiftJis => encoding_rs::SHIFT_JIS,
            Encoding::EucJp => encoding_rs::EUC_JP,
            Encoding::EucCn => encoding_rs::GBK,
        }
    }

    pub(crate) fn check_valid(&self, bytes: &[u8]) -> Result<usize, DecodingError> {
        let mut i = 0;

        while i < bytes.len() && bytes[i] != 0 {
            let char = bytes[i];

            if BANNED_ESCAPE_CHARACTERS.contains(&char) {
                return Err(DecodingError::InvalidEscapeCharacter);
            }

            if char > 0x7F {
                // `char` is the first character of a multibyte glyph.

                // We have to manually checks for the validity of many multibyte combinations because encoding_rs is not strict enough.
                let sequence_length = match *self {
                    Encoding::Ascii => {
                        // ASCII CHECK: prevent decoding bytes outside the ASCII range as ASCII.
                        return Err(DecodingError::AsciiOutOfRange);
                    }
                    Encoding::ShiftJis => {
                        // Invalid first byte according to https://en.wikipedia.org/wiki/Shift_JIS#Shift_JIS_byte_map
                        match char {
                            0x80 | 0xA0 | 0xFD..=0xFF => {
                                return Err(DecodingError::InvalidFirstByteOfMultibyte);
                            }
                            _ => 2,
                        }
                    }
                    Encoding::EucJp => {
                        // Invalid first byte according to https://uic.io/en/charset/show/euc-jp/
                        match char {
                            0x8F => 3,
                            0x80..=0x8D | 0x8F..=0xA0 | 0xA9..=0xAF | 0xF5..=0xFF => {
                                return Err(DecodingError::InvalidFirstByteOfMultibyte);
                            }
                            _ => 2,
                        }
                    }
                    Encoding::EucCn => {
                        // Invalid first byte according to https://uic.io/en/charset/show/euc-cn/
                        match char {
                            0x80..=0xA0 | 0xAA..=0xAF | 0xF8..=0xFF => {
                                return Err(DecodingError::InvalidFirstByteOfMultibyte);
                            }
                            _ => 0,
                        }
                    }
                };

                if i + sequence_length > bytes.len() {
                    return Err(DecodingError::MultibyteNotLongEnough);
                }

                if self
                    .decode_to_string(&bytes[i..i + sequence_length])
                    .is_none()
                {
                    return Err(DecodingError::InvalidMultibyte);
                }

                i += sequence_length;
                continue;
            } else {
                i += 1;
                continue;
            }

            // #[expect(unreachable_code)]
            // i += 1;
        }

        if i >= bytes.len() || bytes[i] != 0 {
            // Reached the end of the buffer without finding an 0
            return Err(DecodingError::TerminatorNotFound);
        }

        // To be a valid aligned string, the next word-aligned bytes needs to be zero
        let mut check_start_offset = i;
        let check_end_offset = bytes.len().min((check_start_offset & !3) + 4);
        while check_start_offset < check_end_offset {
            if bytes[check_start_offset] != 0 {
                return Err(DecodingError::InvalidPad);
            }
            check_start_offset += 1
        }

        i += 1;

        Ok(i)
    }

    // TODO: Decide how this should look like
    pub(crate) fn decode_to_string(&self, bytes: &[u8]) -> Option<String> {
        if let (x, false) = self.as_encoding_rs().decode_without_bom_handling(bytes) {
            Some(x.into_owned())
        } else {
            None
        }
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub(crate) enum DecodingError {
    InvalidEscapeCharacter,
    AsciiOutOfRange,
    MultibyteNotLongEnough,
    InvalidFirstByteOfMultibyte,
    InvalidMultibyte,
    TerminatorNotFound,
    InvalidPad,
}
impl fmt::Display for DecodingError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
impl error::Error for DecodingError {}

// TODO: improve this thingy
pub(crate) fn escape_string(val: &str) -> String {
    let mut escaped_buf = Vec::with_capacity(val.len());
    let bytes = val.as_bytes();

    for b in bytes {
        // The cases of this switch are sorted by ASCII order
        match b {
            0x07 => {
                // \a
                escaped_buf.push(b'\\');
                escaped_buf.push(b'a');
            }
            b'\t' => {
                escaped_buf.push(b'\\');
                escaped_buf.push(b't');
            }
            b'\n' => {
                escaped_buf.push(b'\\');
                escaped_buf.push(b'n');
            }
            0x0C => {
                // \f
                escaped_buf.push(b'\\');
                escaped_buf.push(b'f');
            }
            b'\r' => {
                escaped_buf.push(b'\\');
                escaped_buf.push(b'r');
            }
            b'\"' => {
                escaped_buf.push(b'\\');
                escaped_buf.push(b'"');
            }
            b'\\' => {
                // \a
                escaped_buf.push(b'\\');
                escaped_buf.push(b'\\');
            }
            x => escaped_buf.push(*x),
        }
    }

    String::from_utf8(escaped_buf).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_valid_string_ending_in_word_of_zeroes() {
        static BYTES: [u8; 56] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x67, 0x61, 0x6D, 0x65, 0x20, 0x69, 0x73, 0x20, 0x6E,
            0x6F, 0x74, 0x20, 0x64, 0x65, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x7E, 0x6E, 0x66,
            0x6F, 0x72, 0x20, 0x75, 0x73, 0x65, 0x20, 0x6F, 0x6E, 0x20, 0x74, 0x68, 0x69, 0x73,
            0x20, 0x73, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x2E, 0x7E, 0x7A, 0x00, 0x00, 0x00, 0x00,
        ];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        assert_eq!(maybe_size, Ok(53));
    }

    #[test]
    fn check_valid_string_ending_tight() {
        static BYTES: [u8; 56] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x67, 0x61, 0x6D, 0x65, 0x20, 0x69, 0x73, 0x20, 0x6E,
            0x6F, 0x74, 0x20, 0x64, 0x65, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x7E, 0x6E, 0x66,
            0x6F, 0x72, 0x20, 0x75, 0x73, 0x65, 0x20, 0x6F, 0x6E, 0x20, 0x74, 0x68, 0x69, 0x73,
            0x20, 0x73, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x2E, 0x7E, 0x7A, 0x20, 0x20, 0x20, 0x00,
        ];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        assert_eq!(maybe_size, Ok(56));
    }

    #[test]
    fn check_valid_string_ending_tight_2() {
        static BYTES: [u8; 8] = [0x4E, 0x41, 0x4E, 0x00, 0x3F, 0xF0, 0x00, 0x00];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        assert_eq!(maybe_size, Ok(4));
    }

    #[test]
    fn check_valid_string_invalid() {
        static BYTES: [u8; 56] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x67, 0x61, 0x6D, 0x65, 0x20, 0x69, 0x73, 0x20, 0x6E,
            0x6F, 0x74, 0x20, 0x64, 0x65, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x7E, 0x6E, 0x66,
            0x6F, 0x72, 0x20, 0x75, 0x73, 0x65, 0x20, 0x6F, 0x6E, 0x20, 0x74, 0x68, 0x69, 0x73,
            0x20, 0x73, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x2E, 0x7E, 0x7A, 0x20, 0x20, 0x20, 0x20,
        ];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        assert_eq!(maybe_size, Err(DecodingError::TerminatorNotFound));
    }

    // encoding_rs is silly and decodes those invalid bytes as if they were valid shift-jis strings
    #[test]
    fn check_valid_string_invalid_2() {
        static BYTES: [u8; 16] = [
            0x80, 0x03, 0xD7, 0xCC, //
            0x80, 0x03, 0xD7, 0xCC, //
            0x80, 0x03, 0xD7, 0xB0, //
            0x00, 0x00, 0x00, 0x00, //
        ];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        #[cfg(feature = "std")]
        println!("{:?}", encoding.decode_to_string(&BYTES));

        assert_eq!(maybe_size, Err(DecodingError::InvalidFirstByteOfMultibyte));
    }

    #[test]
    fn check_valid_string_invalid_3() {
        static BYTES: [u8; 4] = [0x80, 0x2C, 0x5E, 0x68];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        #[cfg(feature = "std")]
        println!("{:?}", encoding.decode_to_string(&BYTES));

        assert_eq!(maybe_size, Err(DecodingError::InvalidFirstByteOfMultibyte));
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl Encoding {
        #[staticmethod]
        #[pyo3(name = "from_name")]
        pub fn py_from_name(name: &str) -> Option<Self> {
            Self::from_name(name)
        }
    }
}
