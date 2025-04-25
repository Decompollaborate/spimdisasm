/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::{fmt::Display, time};

use object::read::elf::ElfFile32;
use spimdisasm::config::Endian;

#[track_caller]
#[inline]
pub fn pretty_unwrap<T, E>(value: Result<T, E>) -> T
where
    E: Display,
{
    match value {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
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
    pretty_unwrap(time::SystemTime::now().duration_since(time::UNIX_EPOCH))
}

#[track_caller]
#[inline]
#[must_use]
pub fn read_elf(binary_data: &[u8]) -> ElfFile32 {
    let f = pretty_unwrap(object::File::parse(binary_data));

    if let object::File::Elf32(elf_file) = f {
        elf_file
    } else {
        panic!("Not an elf32 file")
    }
}
