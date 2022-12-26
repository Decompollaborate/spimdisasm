#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import enum


class RelocTypes(enum.Enum):
    MIPS_NONE       = 0 # No reloc
    MIPS_16         = 1 # Direct 16 bit
    MIPS_32         = 2 # Direct 32 bit
    MIPS_REL32      = 3 # PC relative 32 bit
    MIPS_26         = 4 # Direct 26 bit shifted
    MIPS_HI16       = 5 # High 16 bit
    MIPS_LO16       = 6 # Low 16 bit
    MIPS_GPREL16    = 7 # GP relative 16 bit
    MIPS_LITERAL    = 8 # 16 bit literal entry
    MIPS_GOT16      = 9 # 16 bit GOT entry
    MIPS_PC16       = 10 # PC relative 16 bit
    MIPS_CALL16     = 11 # 16 bit GOT entry for function
    MIPS_GPREL32    = 12 # GP relative 32 bit

    MIPS_GOT_HI16   = 22
    MIPS_GOT_LO16   = 23
    MIPS_CALL_HI16  = 30
    MIPS_CALL_LO16  = 31

    @staticmethod
    def fromValue(value: int) -> RelocTypes|None:
        try:
            return RelocTypes(value)
        except ValueError:
            return None
