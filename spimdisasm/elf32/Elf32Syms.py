#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import struct

from .. import common


# a.k.a. Sym (symbol)
@dataclasses.dataclass
class Elf32SymEntry:
    name:   int  # word     # 0x00
    value:  int  # address  # 0x04
    size:   int  # word     # 0x08
    info:   int  # uchar    # 0x0C
    other:  int  # uchar    # 0x0D
    shndx:  int  # section  # 0x0E
                            # 0x10

    @property
    def stBind(self):
        return self.info >> 4

    @property
    def stType(self):
        return self.info & 0xF

    @staticmethod
    def fromBytearray(array_of_bytes: bytes, offset: int = 0) -> Elf32SymEntry:
        entryFormat = common.GlobalConfig.ENDIAN.toFormatString() + "IIIBBH"
        unpacked = struct.unpack_from(entryFormat, array_of_bytes, offset)

        return Elf32SymEntry(*unpacked)

    @staticmethod
    def structSize() -> int:
        return 0x10


class Elf32Syms:
    def __init__(self, array_of_bytes: bytes, offset: int, rawSize: int):
        self.symbols: list[Elf32SymEntry] = list()
        self.offset: int = offset
        self.rawSize: int = rawSize

        for i in range(rawSize // Elf32SymEntry.structSize()):
            entry = Elf32SymEntry.fromBytearray(array_of_bytes, offset + i*Elf32SymEntry.structSize())
            self.symbols.append(entry)

    def __getitem__(self, key: int) -> Elf32SymEntry:
        return self.symbols[key]

    def __iter__(self):
        for entry in self.symbols:
            yield entry

    def __len__(self) -> int:
        return len(self.symbols)
