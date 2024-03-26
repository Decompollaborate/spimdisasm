#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import enum


@dataclasses.dataclass
class CompilerProperties:
    name: str

    hasLateRodata: bool = False

    prevAlign_double: int|None = None # TODO: Specifying 3 as the default should be harmless. Need to investigate.
    prevAlign_jumptable: int|None = None
    prevAlign_string: int|None = 2
    prevAlign_function: int|None = None

    pairMultipleHiToSameLow: bool = True

    allowRdataMigration: bool = False

    bigAddendWorkaroundForMigratedFunctions: bool = True
    """
    Modern GAS can handle big addends (outside the 16-bits range) for the `%lo`
    directive just fine, but old assemblers choke on them, so we truncate them
    to said range when building with those assemblers.

    Decomp projects usually use two assemblers:
    - One to assemble unmigrated files, usually with modern GAS.
    - Another one to assemble individual functions that get inserted into C
      files, either with asm directives from the compiler (using the built-in
      old assembler shipped with the old compiler) or with an external tools
      (like asm-proc for IDO).

    Modern GAS requires no addend truncation to produce matching output, so we
    don't use the workaround for unmigrated asm files.

    For migrated functions we need to know if the compiler uses modern GAS or
    old GAS. If it uses modern GAS (like IDO projects), then this flag should
    be turned off, but if the project uses its own old assembler (like most GCC
    based projects) then this flag needs to be turned on.
    """


@enum.unique
class Compiler(enum.Enum):
    UNKNOWN = CompilerProperties("UNKNOWN")

    # General GCC
    GCC = CompilerProperties("GCC", prevAlign_jumptable=3)

    # N64
    IDO = CompilerProperties("IDO", hasLateRodata=True, pairMultipleHiToSameLow=False, bigAddendWorkaroundForMigratedFunctions=False)
    KMC = CompilerProperties("KMC", prevAlign_jumptable=3)
    SN64 = CompilerProperties("SN64", prevAlign_double=3, prevAlign_jumptable=3, allowRdataMigration=True)

    # iQue
    EGCS = CompilerProperties("EGCS", prevAlign_jumptable=3)

    # PS1
    PSYQ = CompilerProperties("PSYQ", prevAlign_double=3, prevAlign_jumptable=3, allowRdataMigration=True)

    # PS2
    MWCC = CompilerProperties("MWCC", prevAlign_jumptable=4)
    EEGCC = CompilerProperties("EEGCC", prevAlign_jumptable=3, prevAlign_string=3, prevAlign_function=3)

    @staticmethod
    def fromStr(value: str) -> Compiler:
        return compilerOptions.get(value, Compiler.UNKNOWN)


compilerOptions: dict[str, Compiler] = {
    x.name: x
    for x in [
        Compiler.GCC,
        Compiler.IDO,
        Compiler.KMC,
        Compiler.SN64,
        Compiler.EGCS,
        Compiler.PSYQ,
        Compiler.MWCC,
        Compiler.EEGCC,
    ]
}
