#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import struct
from typing import Generator

from .. import common


# a.k.a. Phdr (program header)
@dataclasses.dataclass
class Elf32ProgramHeaderEntry:
    type:       int  # word     # 0x00
    offset:     int  # offset   # 0x04
    vaddr:      int  # address  # 0x08
    paddr:      int  # address  # 0x0C
    filesz:     int  # word     # 0x10
    memsz:      int  # word     # 0x14
    flags:      int  # word     # 0x18
    align:      int  # word     # 0x1C
                                # 0x20

    @staticmethod
    def fromBytearray(array_of_bytes: bytes, offset: int = 0) -> Elf32ProgramHeaderEntry:
        headerFormat = common.GlobalConfig.ENDIAN.toFormatString() + "8I"
        unpacked = struct.unpack_from(headerFormat, array_of_bytes, offset)

        return Elf32ProgramHeaderEntry(*unpacked)


class Elf32ProgramHeaders:
    def __init__(self, array_of_bytes: bytes, phoff: int, phnum: int) -> None:
        self.headers: list[Elf32ProgramHeaderEntry] = list()
        self.phoff: int = phoff
        self.phnum: int = phnum

        for i in range(phnum):
            programHeaderEntry = Elf32ProgramHeaderEntry.fromBytearray(array_of_bytes, phoff + i * 0x20)
            self.headers.append(programHeaderEntry)

    def __iter__(self) -> Generator[Elf32ProgramHeaderEntry, None, None]:
        for entry in self.headers:
            yield entry

    def __len__(self) -> int:
        return len(self.headers)
