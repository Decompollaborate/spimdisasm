#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import enum

from .ContextSymbols import ContextSymbol
from .FileSectionType import FileSectionType


class RelocTypes(enum.Enum):
    MIPS_NONE           = 0 # No reloc
    MIPS_16             = 1 # Direct 16 bit
    MIPS_32             = 2 # Direct 32 bit
    MIPS_REL32          = 3 # PC relative 32 bit
    MIPS_26             = 4 # Direct 26 bit shifted
    MIPS_HI16           = 5 # High 16 bit
    MIPS_LO16           = 6 # Low 16 bit
    MIPS_GPREL16        = 7 # GP relative 16 bit
    MIPS_LITERAL        = 8 # 16 bit literal entry
    MIPS_GOT16          = 9 # 16 bit GOT entry
    MIPS_PC16           = 10 # PC relative 16 bit
    MIPS_CALL16         = 11 # 16 bit GOT entry for function
    MIPS_GPREL32        = 12 # GP relative 32 bit

    MIPS_GOT_HI16       = 22
    MIPS_GOT_LO16       = 23
    MIPS_CALL_HI16      = 30
    MIPS_CALL_LO16      = 31

    # Custom values
    CUSTOM_CONSTANT_HI  = -1
    CUSTOM_CONSTANT_LO  = -2

    @staticmethod
    def fromValue(value: int) -> RelocTypes|None:
        try:
            return RelocTypes(value)
        except ValueError:
            return None

_percentRel = {
    # RelocTypes.MIPS_NONE:       f"",
    # RelocTypes.MIPS_16:         f"",
    # RelocTypes.MIPS_32:         f"",
    # RelocTypes.MIPS_REL32:      f"",
    # RelocTypes.MIPS_26:         f"",
    RelocTypes.MIPS_HI16:       f"%hi",
    RelocTypes.MIPS_LO16:       f"%lo",
    RelocTypes.MIPS_GPREL16:    f"%gp_rel",
    # RelocTypes.MIPS_LITERAL:    f"",
    RelocTypes.MIPS_GOT16:      f"%got",
    # RelocTypes.MIPS_PC16:       f"",
    RelocTypes.MIPS_CALL16:     f"%call16",
    # RelocTypes.MIPS_GPREL32:    f"",

    RelocTypes.MIPS_GOT_HI16:   f"%got_hi",
    RelocTypes.MIPS_GOT_LO16:   f"%got_lo",
    RelocTypes.MIPS_CALL_HI16:  f"%call_hi",
    RelocTypes.MIPS_CALL_LO16:  f"%call_lo",
}

_wordRel = {
    # RelocTypes.MIPS_NONE:       f"",
    # RelocTypes.MIPS_16:         f"",
    RelocTypes.MIPS_32:         f".word",
    # RelocTypes.MIPS_REL32:      f"",
    # RelocTypes.MIPS_26:         f"",
    # RelocTypes.MIPS_HI16:       f"",
    # RelocTypes.MIPS_LO16:       f"",
    # RelocTypes.MIPS_GPREL16:    f"",
    # RelocTypes.MIPS_LITERAL:    f"",
    # RelocTypes.MIPS_GOT16:      f"",
    # RelocTypes.MIPS_PC16:       f"",
    # RelocTypes.MIPS_CALL16:     f"",
    RelocTypes.MIPS_GPREL32:    f".gpword",
    # RelocTypes.MIPS_GOT_HI16:   f"",
    # RelocTypes.MIPS_GOT_LO16:   f"",
    # RelocTypes.MIPS_CALL_HI16:  f"",
    # RelocTypes.MIPS_CALL_LO16:  f"",
}

_operationRel = {
    RelocTypes.CUSTOM_CONSTANT_HI:  f">> 16",
    RelocTypes.CUSTOM_CONSTANT_LO:  f"& 0xFFFF",
}

@dataclasses.dataclass
class RelocationStaticReference:
    # For elfs with relocations to static symbols
    sectionType: FileSectionType
    sectionVram: int

@dataclasses.dataclass
class RelocationInfo:
    relocType: RelocTypes
    symbol: ContextSymbol|str

    addend: int = 0

    staticReference: RelocationStaticReference|None = None

    def getName(self) -> str:
        if isinstance(self.symbol, ContextSymbol):
            name = self.symbol.getName()
        else:
            name = self.symbol

        if self.addend == 0:
            return name
        if self.addend < 0:
            return f"{name} - 0x{-self.addend:X}"
        return f"{name} + 0x{self.addend:X}"

    def getNameWithReloc(self) -> str:
        name = self.getName()

        percentRel = _percentRel.get(self.relocType)
        if percentRel is not None:
            return f"{percentRel}({name})"

        wordRel = _wordRel.get(self.relocType)
        if wordRel is not None:
            return f"{wordRel} {name}"

        operationRel = _operationRel.get(self.relocType)
        if operationRel is not None:
            return f"({name} {operationRel})"

        return name
