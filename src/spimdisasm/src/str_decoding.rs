/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{string::String, vec::Vec};
use core::{error, fmt};

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
static BANNED_ESCAPE_CHARACTERS: [u8; 25] = [
    0x00, // '\0'
    0x01, //
    0x02, //
    0x03, //
    0x04, //
    0x05, //
    0x06, //
    // 0x07, // '\a'
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

static ESCAPE_CHARACTERS_SPECIAL_CASES: [u8; 3] = [
    0x1A, // Arbitrary escape character
    0x1B, // VT escape sequences
    0x7F, //
];

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

            if char > 0x7F {
                // `char` is the first character of a multibyte glyph.

                // We have to manually checks for the validity of many multibyte combinations because encoding_rs is not strict enough.
                let sequence_length = match *self {
                    Encoding::Ascii => {
                        // ASCII CHECK: prevent decoding bytes outside the ASCII range as ASCII.
                        return Err(DecodingError::AsciiOutOfRange {
                            index: i,
                            character: char,
                        });
                    }
                    Encoding::ShiftJis => {
                        // Invalid first byte according to https://en.wikipedia.org/wiki/Shift_JIS#Shift_JIS_byte_map
                        match char {
                            0x80 | 0xA0 | 0xFD..=0xFF => {
                                return Err(DecodingError::InvalidFirstByteOfMultibyte {
                                    index: i,
                                    character: char,
                                });
                            }
                            _ => 2,
                        }
                    }
                    Encoding::EucJp => {
                        // Invalid first byte according to https://uic.io/en/charset/show/euc-jp/
                        match char {
                            0x8F => 3,
                            0x8C | 0x8D => {
                                // Escape character in zelda games
                                i += 1;
                                continue;
                            }
                            0x80..=0x8D | 0x8F..=0xA0 | 0xA9..=0xAF | 0xF5..=0xFF => {
                                return Err(DecodingError::InvalidFirstByteOfMultibyte {
                                    index: i,
                                    character: char,
                                });
                            }
                            _ => 2,
                        }
                    }
                    Encoding::EucCn => {
                        // Invalid first byte according to https://uic.io/en/charset/show/euc-cn/
                        match char {
                            0x80..=0xA0 | 0xAA..=0xAF | 0xF8..=0xFF => {
                                return Err(DecodingError::InvalidFirstByteOfMultibyte {
                                    index: i,
                                    character: char,
                                });
                            }
                            _ => 2,
                        }
                    }
                };

                if i + sequence_length > bytes.len() {
                    return Err(DecodingError::MultibyteNotLongEnough {
                        index: i,
                        character: char,
                        expected_sequence_length: sequence_length,
                    });
                }

                if self
                    .decode_to_string(&bytes[i..i + sequence_length])
                    .is_none()
                {
                    // TODO: there must be a better way to handle this, right?
                    let chars = match sequence_length {
                        1 => [bytes[i], 0, 0],
                        2 => [bytes[i], bytes[i + 1], 0],
                        3 => [bytes[i], bytes[i + 1], bytes[i + 2]],
                        _ => unreachable!(
                            "Oh no! We have a silly bug on the string decoding code! :c"
                        ),
                    };
                    return Err(DecodingError::InvalidMultibyte { index: i, chars });
                }

                i += sequence_length;
                continue;
            } else {
                if BANNED_ESCAPE_CHARACTERS.contains(&char) {
                    return Err(DecodingError::InvalidEscapeCharacter {
                        index: i,
                        character: char,
                    });
                }

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
            let char = bytes[check_start_offset];
            if char != 0 {
                return Err(DecodingError::InvalidPad {
                    index: i,
                    character: char,
                });
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

    pub(crate) fn decode_to_strings_vec(self, bytes: &[u8]) -> Option<Vec<(String, bool)>> {
        let mut ret = Vec::new();
        for (s, finished) in DecoderIterator::new(self, bytes) {
            ret.push((s, finished));
            if finished {
                return Some(
                    ret.into_iter()
                        .map(|(x, finished)| (x.into_string(), finished))
                        .collect(),
                );
            }
        }
        None
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct DecoderIterator<'a> {
    encoding: Encoding,
    bytes: &'a [u8],
    index: usize,
    trailing_backslash: bool,
}

impl<'a> DecoderIterator<'a> {
    fn new(encoding: Encoding, bytes: &'a [u8]) -> Self {
        Self {
            encoding,
            bytes,
            index: 0,
            trailing_backslash: false,
        }
    }

    pub(crate) fn next(&mut self) -> Option<(DecodingResult, bool)> {
        let len = self.bytes.len();

        #[cfg(feature = "pyo3")]
        if len == 0 && self.index == 0 {
            // handle empty strings
            self.index = 1;
            return Some((format!(""), true));
        }

        if self.index >= len {
            return None;
        }

        let c = self.bytes[self.index];
        if self.trailing_backslash {
            self.index += 1;
            self.trailing_backslash = false;
            return Some((DecodingResult::RawChar(c), self.index >= len));
        }
        if c > 0x7F {
            // Non ASCII
            if self.index + 1 < len {
                // If the second byte of a Japanese character is the 0x5C value ('\\') then we need to
                // handle it specially. Otherwise, when it gets iconv'd, the compiler gets confused
                // and thinks it should try to escape the next character instead.
                // So we break down the string here, add these two characters as individual characters and
                // skip them
                let next_char = self.bytes[self.index + 1];
                if next_char == 0x5C {
                    self.index += 1;
                    self.trailing_backslash = true;
                    return Some((DecodingResult::RawChar(c), self.index >= len));
                }
            }
        }

        if ESCAPE_CHARACTERS_SPECIAL_CASES.contains(&c)
            || (self.encoding == Encoding::EucJp && (c == 0x8C || c == 0x8D))
        {
            self.index += 1;
            return Some((DecodingResult::RawChar(c), self.index >= len));
        }

        for i in self.index..len {
            let c = self.bytes[i];

            if c > 0x7F {
                // Non ASCII
                if i + 1 < len {
                    // If the second byte of a Japanese character is the 0x5C value ('\\') then we need to
                    // handle it specially. Otherwise, when it gets iconv'd, the compiler gets confused
                    // and thinks it should try to escape the next character instead.
                    // So we break down the string here, add these two characters as individual characters and
                    // skip them
                    let next_char = self.bytes[i + 1];
                    if next_char == 0x5C {
                        let start = self.index;
                        self.index = i;
                        return self
                            .encoding
                            .decode_to_string(&self.bytes[start..i])
                            .map(|x| (DecodingResult::DecodedString(x), self.index >= len));
                    }
                }
            }

            if ESCAPE_CHARACTERS_SPECIAL_CASES.contains(&c)
                || (self.encoding == Encoding::EucJp && (c == 0x8C || c == 0x8D))
            {
                let start = self.index;
                self.index = i;
                return self
                    .encoding
                    .decode_to_string(&self.bytes[start..i])
                    .map(|x| (DecodingResult::DecodedString(x), self.index >= len));
            }
        }

        let start = self.index;
        self.index = len;
        self.encoding
            .decode_to_string(&self.bytes[start..len])
            .map(|x| (DecodingResult::DecodedString(x), true))
    }
}

impl Iterator for DecoderIterator<'_> {
    type Item = (DecodingResult, bool);

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum DecodingResult {
    DecodedString(String),
    RawChar(u8),
}

impl DecodingResult {
    fn into_string(self) -> String {
        match self {
            DecodingResult::DecodedString(x) => escape_string(&x),
            DecodingResult::RawChar(c) => format!("\\x{:02X}", c),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub(crate) enum DecodingError {
    InvalidEscapeCharacter {
        index: usize,
        character: u8,
    },
    AsciiOutOfRange {
        index: usize,
        character: u8,
    },
    MultibyteNotLongEnough {
        index: usize,
        character: u8,
        expected_sequence_length: usize,
    },
    InvalidFirstByteOfMultibyte {
        index: usize,
        character: u8,
    },
    InvalidMultibyte {
        index: usize,
        chars: [u8; 3],
    },
    TerminatorNotFound,
    InvalidPad {
        index: usize,
        character: u8,
    },
}
impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Consider writing an actual impl here
        fmt::Debug::fmt(&self, f)
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

        assert_eq!(
            maybe_size,
            Err(DecodingError::InvalidFirstByteOfMultibyte {
                index: 0,
                character: 0x80
            })
        );
    }

    #[test]
    fn check_valid_string_invalid_3() {
        static BYTES: [u8; 5] = [0x80, 0x2C, 0x5E, 0x68, 0x00];
        let encoding = Encoding::ShiftJis;

        let maybe_size = encoding.check_valid(&BYTES);

        #[cfg(feature = "std")]
        println!("{:?}", encoding.decode_to_string(&BYTES));

        assert_eq!(
            maybe_size,
            Err(DecodingError::InvalidFirstByteOfMultibyte {
                index: 0,
                character: 0x80
            })
        );
    }

    #[test]
    fn asdf() {
        static BYTES: [u8; 276] = [
            0x82, 0x60, 0x82, 0x61, 0x82, 0x62, 0x82, 0x63, 0x82, 0x64, 0x5F, 0x5F, 0x82, 0x81,
            0x82, 0x82, 0x82, 0x83, 0x82, 0x84, 0x82, 0x85, 0x5F, 0x5F, 0x81, 0x9B, 0x81, 0x7E,
            0x81, 0x99, 0x82, 0x65, 0x82, 0x66, 0x82, 0x67, 0x82, 0x68, 0x82, 0x69, 0x5F, 0x5F,
            0x82, 0x86, 0x82, 0x87, 0x82, 0x88, 0x82, 0x89, 0x82, 0x8A, 0x5F, 0x5F, 0x81, 0x45,
            0x81, 0x44, 0x81, 0x46, 0x82, 0x6A, 0x82, 0x6B, 0x82, 0x6C, 0x82, 0x6D, 0x82, 0x6E,
            0x5F, 0x5F, 0x82, 0x8B, 0x82, 0x8C, 0x82, 0x8D, 0x82, 0x8E, 0x82, 0x8F, 0x5F, 0x5F,
            0x81, 0x49, 0x81, 0x48, 0x81, 0x95, 0x82, 0x6F, 0x82, 0x70, 0x82, 0x71, 0x82, 0x72,
            0x82, 0x73, 0x5F, 0x5F, 0x82, 0x90, 0x82, 0x91, 0x82, 0x92, 0x82, 0x93, 0x82, 0x94,
            0x5F, 0x5F, 0x81, 0x96, 0x81, 0x5E, 0x81, 0x93, 0x82, 0x74, 0x82, 0x75, 0x82, 0x76,
            0x82, 0x77, 0x82, 0x78, 0x5F, 0x5F, 0x82, 0x95, 0x82, 0x96, 0x82, 0x97, 0x82, 0x98,
            0x82, 0x99, 0x5F, 0x5F, 0x81, 0x43, 0x81, 0x66, 0x81, 0x60, 0x82, 0x79, 0x5F, 0x5F,
            0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x82, 0x9A, 0x5F, 0x5F, 0x5F, 0x5F,
            0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x82, 0x4F,
            0x82, 0x50, 0x82, 0x51, 0x82, 0x52, 0x82, 0x53, 0x5F, 0x5F, 0x82, 0x54, 0x82, 0x55,
            0x82, 0x56, 0x82, 0x57, 0x82, 0x58, 0x5F, 0x5F, 0x33, 0x30, 0x33, 0x31, 0x33, 0x32,
            0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F,
            0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x34, 0x30, 0x34, 0x31,
            0x34, 0x32, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F,
            0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x5F, 0x35, 0x30,
            0x35, 0x31, 0x35, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let encoding = Encoding::ShiftJis;

        let decoded = encoding.decode_to_strings_vec(&BYTES);
        assert!(decoded.is_some());
        let decoded = decoded.unwrap();

        #[cfg(feature = "std")]
        for (x, f) in &decoded {
            println!("{} {}", f, x);
        }

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].0, "ＡＢＣＤＥ__ａｂｃｄｅ__○×☆ＦＧＨＩＪ__ｆｇｈｉｊ__・．：ＫＬＭＮＯ__ｋｌｍｎｏ__！？＆ＰＱＲＳＴ__ｐｑｒｓｔ__＊／％ＵＶＷＸＹ__ｕｖｗｘｙ__，’～Ｚ__________ｚ________________０１２３４__５６７８９__303132________________________404142________________________505152\0\0\0\0\0\0");
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
