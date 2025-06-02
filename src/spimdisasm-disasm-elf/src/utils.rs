/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{fmt::Display, time};

use object::read::elf::ElfFile32;
use spimdisasm::config::Endian;

pub trait PrettyUnwrap {
    type Output;

    fn pretty_unwrap(self) -> Self::Output;
}

impl<T, E> PrettyUnwrap for Result<T, E>
where
    E: Display,
{
    type Output = T;

    #[track_caller]
    #[inline]
    fn pretty_unwrap(self) -> Self::Output {
        match self {
            Ok(v) => v,
            Err(e) => panic!("{}", e),
        }
    }
}

#[track_caller]
#[inline]
pub fn set_valid_option_on_none<T>(dst: &mut Option<T>, val: Option<T>) {
    // Missing a previous value is bad.
    assert!(dst.is_none());
    // Unwrap to make sure it is "valid".
    *dst = Some(val.unwrap());
}

#[inline]
#[must_use]
pub fn endian_to_endian(endian: object::Endianness) -> Endian {
    match endian {
        object::Endianness::Big => Endian::Big,
        object::Endianness::Little => Endian::Little,
    }
}

#[track_caller]
#[inline]
#[must_use]
pub fn get_time_now() -> time::Duration {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .pretty_unwrap()
}

#[track_caller]
#[inline]
#[must_use]
pub fn read_elf(binary_data: &[u8]) -> ElfFile32<'_> {
    let f = object::File::parse(binary_data).pretty_unwrap();

    if let object::File::Elf32(elf_file) = f {
        elf_file
    } else {
        panic!("Not an elf32 file")
    }
}
