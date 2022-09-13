#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import struct


@dataclasses.dataclass
class Elf32RegInfo:
    gprmask:    int        # uint32_t    # 0x00 /* General registers used.  */
    cprmask:    list[int]  # uint32_t[4] # 0x04 /* Coprocessor registers used.  */
    gpValue:    int        # int32_t     # 0x14 /* $gp register value.  */
                                         # 0x18

    @staticmethod
    def fromBytearray(array_of_bytes: bytearray, offset: int = 0) -> Elf32RegInfo:
        gprFormat = ">I"
        gpr = struct.unpack_from(gprFormat, array_of_bytes, 0 + offset)[0]
        # print(gpr)

        cprFormat = ">4I"
        cpr = list(struct.unpack_from(cprFormat, array_of_bytes, 4 + offset))
        # print(cpr)

        gpFormat = ">i"
        gp = struct.unpack_from(gpFormat, array_of_bytes, 0x14 + offset)[0]
        # print(gp)

        return Elf32RegInfo(gpr, cpr, gp)
