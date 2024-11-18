/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

pub trait Symbol {
    fn bytes(&self) -> &[u8];
}
