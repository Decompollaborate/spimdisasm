/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Vram;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    metadata::{segment_metadata::FindSettings, SegmentMetadata, SymbolMetadata, SymbolType},
    size::Size,
    str_decoding::Encoding,
};

/// Sets the level for the rodata C string guesser. Smaller values mean more conservative methods
/// to guess a string, while higher values are more agressive. The first level ([`No`]) completely
/// disables the guessing feature.
///
/// A C string must start at a 0x4-aligned region, it must be '\\0'-terminated and padded with
/// '\\0's until the next 0x4 boundary. There's no way to bypass this hard restriction.
///
/// - Level [`No`]: Completely disable the guessing feature.
/// - Level [`Conservative`]: The most conservative guessing level. Imposes the following restrictions:
///     - Do not try to guess if the user provided a type for the symbol.
///     - Do not try to guess if type information for the symbol can be inferred by other means.
///     - A string symbol must be referenced only once.
///     - Strings must not be empty.
/// - Level [`MultipleReferences`]: A string no longer needs to be referenced only once to be
///   considered a string candidate. This may happen because of a deduplication optimization or by
///   plain `data` strings.
/// - Level [`EmptyStrings`]: Allow empty strings. Likely to yield false positives.
/// - Level [`IgnoreDetectedType`]: Symbols with autodetected type information but no user type
///   information can still be guessed as strings.
/// - Level [`Full`]: (Almost) always try to guess if something is a string. Only avoids checking
///   if something is a string if it has a user-declared type.
///
/// [`No`]: StringGuesserLevel::No
/// [`Conservative`]: StringGuesserLevel::Conservative
/// [`MultipleReferences`]: StringGuesserLevel::MultipleReferences
/// [`EmptyStrings`]: StringGuesserLevel::EmptyStrings
/// [`IgnoreDetectedType`]: StringGuesserLevel::IgnoreDetectedType
/// [`Full`]: StringGuesserLevel::Full
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum StringGuesserLevel {
    /// Completely disable the guessing feature.
    No,

    /// The most conservative guessing level. Imposes the following restrictions:
    /// - Do not try to guess if the user provided a type for the symbol.
    /// - Do not try to guess if type information for the symbol can be inferred by other means.
    /// - A string symbol must be referenced only once.
    /// - Strings must not be empty.
    Conservative,

    /// A string no longer needs to be referenced only once to be considered a string candidate.
    /// This may happen because of a deduplication optimization or by plain `data` strings.
    MultipleReferences,

    /// Allow empty strings. Likely to yield false positives.
    EmptyStrings,

    /// Symbols with autodetected type information but no user type information can still be
    /// guessed as strings.
    IgnoreDetectedType,

    /// (Almost) always try to guess if something is a string.
    ///
    /// Only avoids checking if something is a string if it has a user-declared type.
    Full,
}

impl StringGuesserLevel {
    pub const fn default() -> Self {
        StringGuesserLevel::MultipleReferences
    }

    #[must_use]
    pub fn guess(
        &self,
        metadata: Option<&SymbolMetadata>,
        vram: Vram,
        bytes: &[u8],
        encoding: Encoding,
        owned_segment: &SegmentMetadata,
    ) -> Option<usize> {
        /*
        if contextSym._ranStringCheck:
            return contextSym.isMaybeString
        */

        if let Some(metadata) = metadata {
            if metadata.sym_type() == Some(&SymbolType::CString) {
                let size = if let Some(size) = metadata.user_declared_size() {
                    size.inner().next_multiple_of(4) as usize
                } else if let Some(str_end) = bytes.iter().position(|x| *x == 0) {
                    (str_end + 1).next_multiple_of(4)
                } else {
                    return None;
                };

                // TODO: check padding
                return Some(size);
            } else if metadata.user_declared_type().is_some() {
                return None;
            }
        }

        /*
        if not self.enableStringGuessing:
            return False
        */

        if vram.inner() % 4 != 0 {
            // A C string must start at a 0x4-aligned region
            return None;
        }

        if *self <= Self::No {
            return None;
        }

        if metadata.is_some_and(|x| x.reference_counter() > 1) && *self < Self::MultipleReferences {
            return None;
        }

        if bytes[0] == b'\0' {
            // Empty strings are pretty rare and unlikely.
            if *self < Self::EmptyStrings {
                return None;
            }
        }

        if metadata.is_some_and(|x| x.autodetected_type().is_some())
            && *self < Self::IgnoreDetectedType
        {
            return None;
        }

        let raw_size = if let Ok(raw_size) = encoding.check_valid(bytes) {
            raw_size
        } else {
            return None;
        };

        // TODO: why `.with_check_upper_limit(false)`?
        if owned_segment
            .find_symbol(
                vram + Size::new(raw_size as u32 - 1),
                FindSettings::default()
                    .with_allow_addend(true)
                    .with_check_upper_limit(false),
            )
            .is_some_and(|x| x.vram() != vram)
        {
            // Check if there is already another symbol after the current one and before the end of the string,
            // in which case we say this symbol should not be a string
            return None;
        }

        Some(raw_size.next_multiple_of(4))
    }
}

impl Default for StringGuesserLevel {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        address_range::AddressRange, metadata::GeneratedBy, rom_address::RomAddress,
        rom_vram_range::RomVramRange,
    };

    use super::*;

    #[test]
    fn guess_string_ending_in_word_of_zeroes() {
        static BYTES: [u8; 56] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x67, 0x61, 0x6D, 0x65, 0x20, 0x69, 0x73, 0x20, 0x6E,
            0x6F, 0x74, 0x20, 0x64, 0x65, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x7E, 0x6E, 0x66,
            0x6F, 0x72, 0x20, 0x75, 0x73, 0x65, 0x20, 0x6F, 0x6E, 0x20, 0x74, 0x68, 0x69, 0x73,
            0x20, 0x73, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x2E, 0x7E, 0x7A, 0x00, 0x00, 0x00, 0x00,
        ];
        let size = Size::new(BYTES.len() as u32 * 2);
        let encoding = Encoding::ShiftJis;
        let vram = Vram::new(0x80000000);
        let mut owned_segment = SegmentMetadata::new(
            RomVramRange::new(
                AddressRange::new(RomAddress::new(0), RomAddress::new(0) + size),
                AddressRange::new(vram, vram + size),
            ),
            None,
        );
        owned_segment.add_symbol(
            vram + Size::new(BYTES.len() as u32),
            None,
            GeneratedBy::Autogenerated,
            None,
            false,
        );
        let guesser = StringGuesserLevel::MultipleReferences;

        let maybe_size = guesser.guess(None, vram, &BYTES, encoding, &owned_segment);
        // println!("{:?}", maybe_size);

        //None::<u32>.unwrap();
        assert_eq!(maybe_size, Some(56));
    }

    #[test]
    fn guess_string_ending_tight() {
        static BYTES: [u8; 4] = [0x4E, 0x41, 0x4E, 0x00];
        let size = Size::new(BYTES.len() as u32 * 2);
        let encoding = Encoding::ShiftJis;
        let vram = Vram::new(0x80000000);
        let mut owned_segment = SegmentMetadata::new(
            RomVramRange::new(
                AddressRange::new(RomAddress::new(0), RomAddress::new(0) + size),
                AddressRange::new(vram, vram + size),
            ),
            None,
        );
        owned_segment.add_symbol(
            vram + Size::new(BYTES.len() as u32),
            None,
            GeneratedBy::Autogenerated,
            None,
            false,
        );
        let guesser = StringGuesserLevel::MultipleReferences;

        let maybe_size = guesser.guess(None, vram, &BYTES, encoding, &owned_segment);
        // println!("{:?}", maybe_size);

        //None::<u32>.unwrap();
        assert_eq!(maybe_size, Some(4));
    }
}
