#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import struct


# a.k.a. Dyn ()
@dataclasses.dataclass
class Elf32DynEntry:
    d_tag:  int  # int32_t  # 0x00
    "Dynamic entry type"
    d_val:  int  # uint32_t # 0x04
    "Integer value"
                            # 0x08

    @property
    def d_ptr(self) -> int:
        "Address value"
        # Elf32_Addr
        return self.d_val

    @staticmethod
    def fromBytearray(array_of_bytes: bytearray, offset: int = 0) -> Elf32DynEntry:
        entryFormat = ">II"
        unpacked = struct.unpack_from(entryFormat, array_of_bytes, offset)

        return Elf32DynEntry(*unpacked)

    @staticmethod
    def structSize() -> int:
        return 0x08


class Elf32Dyns:
    def __init__(self, array_of_bytes: bytearray, offset: int, rawSize: int):
        self.dyns: list[Elf32DynEntry] = list()
        self.offset: int = offset
        self.rawSize: int = rawSize

        for i in range(rawSize // Elf32DynEntry.structSize()):
            entry = Elf32DynEntry.fromBytearray(array_of_bytes, offset + i*Elf32DynEntry.structSize())
            self.dyns.append(entry)

    def __getitem__(self, key: int) -> Elf32DynEntry:
        return self.dyns[key]
