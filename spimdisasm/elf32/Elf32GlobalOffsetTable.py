#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import struct


class Elf32GlobalOffsetTable:
    def __init__(self, array_of_bytes: bytearray, offset: int, rawSize: int):
        self.entries: list[int] = list()
        self.offset: int = offset
        self.rawSize: int = rawSize

        entryFormat = f">{rawSize//4}I"
        self.entries = list(struct.unpack_from(entryFormat, array_of_bytes, offset))

    def __getitem__(self, key: int) -> int:
        return self.entries[key]

    def __len__(self) -> int:
        return len(self.entries)
