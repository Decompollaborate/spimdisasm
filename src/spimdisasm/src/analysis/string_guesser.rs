/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use bitflags::bitflags;
use core::{error, fmt};
use rabbitizer::access_type::AccessType;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{addresses::Vram, config::Compiler, metadata::SymbolType, str_decoding::Encoding};

use super::ReferenceWrapper;

bitflags! {
    /// Sets flags for tweaking for the C string guesser.
    ///
    /// A C string must start at a 0x4-aligned region, it must be '\\0'-terminated and padded with
    /// '\\0's until the next 0x4 boundary. There's no way to bypass this hard restriction.
    /// Some compilers may even impose an stricter alignement than 0x4.
    ///
    /// Flag combinations are additive, meaning that enabling more flag allows for more wiggle room
    /// while trying to guess for strings. Flags are build on top of [`Basic`].
    ///
    /// - [`Basic`]: The most conservative setting. Imposes the following restrictions:
    ///     - Do not try to guess if the user provided a type for the symbol.
    ///     - Do not try to guess if the user provided just a size for the symbol, but that size
    ///       doesn't match the one of a correct string (i.e. missing terminator).
    ///     - Do not try to guess if type information for the symbol can be inferred by other means.
    ///     - A string symbol must be referenced only once.
    ///     - Strings must not be empty.
    /// - [`MultipleReferences`]: A string no longer needs to be referenced only once to be
    ///   considered a string candidate. This may happen because of a deduplication optimization or by
    ///   plain `data` strings.
    /// - [`EmptyStrings`]: Allow empty strings. Likely to yield false positives.
    /// - [`IgnoreDetectedType`]: Symbols with autodetected type information but no user type
    ///   information can still be guessed as strings.
    ///
    /// Additionally it is possible to completely disable guessing for strings with the [`no`]
    /// function. On the hand it is possible to enable all settings and almost always try to guess
    /// if something is a string with the [`full`] function, this only avoids guessing if the
    /// user provided a type or a size for the symbol.
    ///
    /// [`no`]: StringGuesserFlags::no
    /// [`Basic`]: StringGuesserFlags::Basic
    /// [`MultipleReferences`]: StringGuesserFlags::MultipleReferences
    /// [`EmptyStrings`]: StringGuesserFlags::EmptyStrings
    /// [`IgnoreDetectedType`]: StringGuesserFlags::IgnoreDetectedType
    /// [`full`]: StringGuesserFlags::full
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    #[non_exhaustive]
    #[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
    pub struct StringGuesserFlags: u32 {
        /// The most conservative setting. Imposes the following restrictions:
        /// - Do not try to guess if the user provided a type for the symbol.
        /// - Do not try to guess if the user provided just a size for the symbol, but that size
        ///   doesn't match the one of a correct string (i.e. missing terminator).
        /// - Do not try to guess if type information for the symbol can be inferred by other means.
        /// - A string symbol must be referenced only once.
        /// - Strings must not be empty.
        const Basic = 1 << 0;

        /// A string no longer needs to be referenced only once to be considered a string candidate.
        /// This may happen because of a deduplication optimization or by plain `data` strings.
        const MultipleReferences = 1 << 1;

        /// Allow empty strings. Likely to yield false positives.
        const EmptyStrings = 1 << 2;

        /// Symbols with autodetected type information but no user type information can still be guessed as strings.
        const IgnoreDetectedType = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub(crate) enum StringGuessError {
    UserTypeMissingTerminatorAndNoGivenSize,
    UserSizeButTerminatorMismatch,
    UserSizeMissingTerminator,
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
            StringGuessError::UserSizeButTerminatorMismatch => {
                write!(f, "UserSizeButTerminatorMismatch")
            }
            StringGuessError::UserSizeMissingTerminator => write!(f, "UserSizeMissingTerminator"),
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

impl StringGuesserFlags {
    pub const fn default() -> Self {
        Self::Basic.union(Self::MultipleReferences)
    }

    pub const fn full() -> Self {
        Self::all()
    }

    pub const fn no() -> Self {
        Self::empty()
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
            } else if let Some(user_size) = ref_wrapper.user_declared_size() {
                // No user type, but user size?

                if let Some(str_end) = bytes.iter().position(|x| *x == 0) {
                    // User may give us the precise size or a word-aligned size, accept both
                    if user_size.inner().next_multiple_of(4) as usize
                        == (str_end + 1).next_multiple_of(4)
                    {
                        // Do not return the size here, we still have to check if this is a valid string
                    } else {
                        return Err(StringGuessError::UserSizeButTerminatorMismatch);
                    }
                } else {
                    return Err(StringGuessError::UserSizeMissingTerminator);
                }
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

        if self.is_empty() {
            return Err(StringGuessError::GuesserDisabled);
        }

        if !self.contains(Self::MultipleReferences)
            && ref_wrapper.is_some_and(|x| x.reference_counter() > 1)
        {
            return Err(StringGuessError::ReferencedMoreThanOnce);
        }

        if !self.contains(Self::EmptyStrings) && bytes[0] == b'\0' {
            // Empty strings are pretty rare and unlikely.
            return Err(StringGuessError::EmptyString);
        }

        if !self.contains(Self::IgnoreDetectedType)
            && ref_wrapper.is_some_and(|x| {
                x.autodetected_type()
                    .is_some_and(|x| x != SymbolType::CString)
            })
        {
            return Err(StringGuessError::HasAutodetectedType);
        }
        if !self.contains(Self::IgnoreDetectedType)
            && ref_wrapper.is_some_and(|x| {
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
            })
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

impl Default for StringGuesserFlags {
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
        let guesser = StringGuesserFlags::default();

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
        let guesser = StringGuesserFlags::default();

        let maybe_size = guesser.guess(None, vram, &BYTES, encoding, None, false);

        #[cfg(feature = "std")]
        println!("{:?}", maybe_size);

        //None::<u32>.unwrap();
        assert_eq!(maybe_size, Ok(4));
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl StringGuesserFlags {
        #[staticmethod]
        #[pyo3(name = "full")]
        pub const fn py_full() -> Self {
            Self::full()
        }

        #[staticmethod]
        #[pyo3(name = "no")]
        pub const fn py_no() -> Self {
            Self::no()
        }

        #[staticmethod]
        #[pyo3(name = "Basic")]
        pub const fn py_basic() -> Self {
            Self::Basic
        }
        #[staticmethod]
        #[pyo3(name = "MultipleReferences")]
        pub const fn py_multiple_references() -> Self {
            Self::MultipleReferences
        }
        #[staticmethod]
        #[pyo3(name = "EmptyStrings")]
        pub const fn py_empty_strings() -> Self {
            Self::EmptyStrings
        }
        #[staticmethod]
        #[pyo3(name = "IgnoreDetectedType")]
        pub const fn py_ignore_detected_type() -> Self {
            Self::IgnoreDetectedType
        }

        pub fn __or__(&self, other: Self) -> Self {
            *self | other
        }
    }
}
