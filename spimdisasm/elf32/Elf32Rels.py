#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import struct

from .. import common


@dataclasses.dataclass
class Elf32RelEntry:
    offset: int  # address  # 0x00
    info:   int  # word     # 0x04
                            # 0x08

    @property
    def rSym(self):
        return self.info >> 8

    @property
    def rType(self):
        return self.info & 0xFF

    @staticmethod
    def fromBytearray(array_of_bytes: bytes, offset: int = 0) -> Elf32RelEntry:
        entryFormat = common.GlobalConfig.ENDIAN.toFormatString() + "II"
        unpacked = struct.unpack_from(entryFormat, array_of_bytes, offset)

        return Elf32RelEntry(*unpacked)


class Elf32Rels:
    def __init__(self, sectionName: str, array_of_bytes: bytes, offset: int, rawSize: int):
        self.sectionName = sectionName
        self.relocations: list[Elf32RelEntry] = list()
        self.offset: int = offset
        self.rawSize: int = rawSize

        for i in range(rawSize // 0x08):
            entry = Elf32RelEntry.fromBytearray(array_of_bytes, offset + i*0x08)
            self.relocations.append(entry)

    def __iter__(self):
        for entry in self.relocations:
            yield entry
