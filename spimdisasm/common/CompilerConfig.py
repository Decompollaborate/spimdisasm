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


@enum.unique
class Compiler(enum.Enum):
    UNKNOWN = CompilerProperties("UNKNOWN")

    # General GCC
    GCC = CompilerProperties("GCC", prevAlign_jumptable=3)

    # N64
    IDO = CompilerProperties("IDO", hasLateRodata=True)
    KMC = CompilerProperties("KMC", prevAlign_jumptable=3)
    SN64 = CompilerProperties("SN64", prevAlign_double=3, prevAlign_jumptable=3)

    # iQue
    EGCS = CompilerProperties("EGCS", prevAlign_jumptable=3)

    # PS1
    PSYQ = CompilerProperties("PSYQ", prevAlign_double=3, prevAlign_jumptable=3)

    # PS2
    MWCC = CompilerProperties("MWCC", prevAlign_jumptable=4)
    EEGCC = CompilerProperties("EEGCC", prevAlign_jumptable=3, prevAlign_string=3, prevAlign_function=4)

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
