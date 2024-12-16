/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{string::String, vec::Vec};

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
