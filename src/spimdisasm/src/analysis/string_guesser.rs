/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rabbitizer::access_type::AccessType;

use crate::{addresses::Vram, config::Compiler, metadata::SymbolType, str_decoding::Encoding};

use super::ReferenceWrapper;

/// Sets the level for the rodata C string guesser. Smaller values mean more conservative methods
/// to guess a string, while higher values are more agressive. The first level ([`No`]) completely
/// disables the guessing feature.
///
/// A C string must start at a 0x4-aligned region, it must be '\\0'-terminated and padded with
/// '\\0's until the next 0x4 boundary. There's no way to bypass this hard restriction.
/// Some compilers may even impose an stricter alignement than 0x4.
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub(crate) enum StringGuessError {
    UserTypeMissingTerminatorAndNoGivenSize,
    GivenUserTypeIsNotCString,
    ReachedLateRodata,
    NotProperAlignment,
    GuesserDisabled,
    ReferencedMoreThanOnce,
    EmptyString,
    HasAutodetectedType,
    HasBeenDereferenced,
    InvalidString,
}

impl fmt::Display for StringGuessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: write a proper display
        match self {
            StringGuessError::UserTypeMissingTerminatorAndNoGivenSize => {
                write!(f, "UserTypeMissingTerminatorAndNoGivenSize")
            }
            StringGuessError::GivenUserTypeIsNotCString => write!(f, "GivenUserTypeIsNotCString"),
            StringGuessError::ReachedLateRodata => write!(f, "ReachedLateRodata"),
            StringGuessError::NotProperAlignment => write!(f, "NotProperAlignment"),
            StringGuessError::GuesserDisabled => write!(f, "GuesserDisabled"),
            StringGuessError::ReferencedMoreThanOnce => write!(f, "ReferencedMoreThanOnce"),
            StringGuessError::EmptyString => write!(f, "EmptyString"),
            StringGuessError::HasAutodetectedType => write!(f, "HasAutodetectedType"),
            StringGuessError::HasBeenDereferenced => write!(f, "HasBeenDereferenced"),
            StringGuessError::InvalidString => write!(f, "InvalidString"),
        }
    }
}
impl error::Error for StringGuessError {}

// TODO: add an option to avoid guessing strings on unreferenced symbols

impl StringGuesserLevel {
    pub const fn default() -> Self {
        StringGuesserLevel::MultipleReferences
    }

    pub(crate) fn guess(
        &self,
        ref_wrapper: Option<ReferenceWrapper>,
        vram: Vram,
        bytes: &[u8],
        encoding: Encoding,
        compiler: Option<Compiler>,
        reached_late_rodata: bool,
    ) -> Result<usize, StringGuessError> {
        /*
        if contextSym._ranStringCheck:
            return contextSym.isMaybeString
        */

        if let Some(ref_wrapper) = ref_wrapper {
            // Check for user-defined info.

            if ref_wrapper.sym_type() == Some(SymbolType::CString) {
                // User says it is a C string, we gotta believe them.

                let size = if let Some(size) = ref_wrapper.user_declared_size() {
                    // Blindly believe the user about the size of the string.
                    size.inner() as usize
                } else if let Some(str_end) = bytes.iter().position(|x| *x == 0) {
                    // Zero terminator.
                    str_end + 1
                } else {
                    return Err(StringGuessError::UserTypeMissingTerminatorAndNoGivenSize);
                };

                return Ok(size);
            } else if ref_wrapper.user_declared_type().is_some() {
                // User said this symbol is a non string.
                return Err(StringGuessError::GivenUserTypeIsNotCString);
            }
        }

        if reached_late_rodata {
            return Err(StringGuessError::ReachedLateRodata);
        }

        /*
        if not self.enableStringGuessing:
            return False
        */

        let expected_alignement = {
            let alignment_shift = compiler
                .and_then(|x| x.prev_align_for_type(SymbolType::CString))
                .unwrap_or(2);

            1 << alignment_shift
        };

        if vram.inner() % expected_alignement != 0 {
            // A C string must start at a 0x4-aligned region
            return Err(StringGuessError::NotProperAlignment);
        }

        if *self <= Self::No {
            return Err(StringGuessError::GuesserDisabled);
        }

        if ref_wrapper.is_some_and(|x| x.reference_counter() > 1)
            && *self < Self::MultipleReferences
        {
            return Err(StringGuessError::ReferencedMoreThanOnce);
        }

        if bytes[0] == b'\0' {
            // Empty strings are pretty rare and unlikely.
            if *self < Self::EmptyStrings {
                return Err(StringGuessError::EmptyString);
            }
        }

        if ref_wrapper.is_some_and(|x| {
            x.autodetected_type()
                .is_some_and(|x| x != SymbolType::CString)
        }) && *self < Self::IgnoreDetectedType
        {
            return Err(StringGuessError::HasAutodetectedType);
        }
        if ref_wrapper.is_some_and(|x| {
            x.all_access_types()
                .iter()
                .filter(|(x, _)| {
                    !matches!(
                        x,
                        AccessType::WORD_LEFT
                            | AccessType::WORD_RIGHT
                            | AccessType::DOUBLEWORD_LEFT
                            | AccessType::DOUBLEWORD_RIGHT
                    )
                })
                .count()
                != 0
        }) && *self < Self::IgnoreDetectedType
        {
            // Avoid considering something as a string if it has been dereferenced.
            // But allow LEFT/RIGHT accesses, since that can be used to declare strings on the stack.
            // TODO: make this thing its own guessing level instead of hyjacking IgnoreDetectedType.
            return Err(StringGuessError::HasBeenDereferenced);
        }

        let raw_size = if let Ok(raw_size) = encoding.check_valid(bytes) {
            raw_size
        } else {
            return Err(StringGuessError::InvalidString);
        };

        Ok(raw_size)
    }
}

impl Default for StringGuesserLevel {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guess_string_ending_in_word_of_zeroes() {
        static BYTES: [u8; 56] = [
            0x54, 0x68, 0x69, 0x73, 0x20, 0x67, 0x61, 0x6D, 0x65, 0x20, 0x69, 0x73, 0x20, 0x6E,
            0x6F, 0x74, 0x20, 0x64, 0x65, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x7E, 0x6E, 0x66,
            0x6F, 0x72, 0x20, 0x75, 0x73, 0x65, 0x20, 0x6F, 0x6E, 0x20, 0x74, 0x68, 0x69, 0x73,
            0x20, 0x73, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x2E, 0x7E, 0x7A, 0x00, 0x00, 0x00, 0x00,
        ];
        let encoding = Encoding::ShiftJis;
        let vram = Vram::new(0x80000000);
        let guesser = StringGuesserLevel::MultipleReferences;

        let maybe_size = guesser.guess(None, vram, &BYTES, encoding, None, false);

        #[cfg(feature = "std")]
        println!("{:?}", maybe_size);

        //None::<u32>.unwrap();
        assert_eq!(maybe_size, Ok(53));
    }

    #[test]
    fn guess_string_ending_tight() {
        static BYTES: [u8; 4] = [0x4E, 0x41, 0x4E, 0x00];
        let encoding = Encoding::ShiftJis;
        let vram = Vram::new(0x80000000);
        let guesser = StringGuesserLevel::MultipleReferences;

        let maybe_size = guesser.guess(None, vram, &BYTES, encoding, None, false);

        #[cfg(feature = "std")]
        println!("{:?}", maybe_size);

        //None::<u32>.unwrap();
        assert_eq!(maybe_size, Ok(4));
    }
}
