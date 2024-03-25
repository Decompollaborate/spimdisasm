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


@enum.unique
class Compiler(enum.Enum):
    UNKNOWN = CompilerProperties("UNKNOWN")

    # General GCC
    GCC = CompilerProperties("GCC")

    # N64
    IDO = CompilerProperties("IDO", hasLateRodata=True)
    KMC = CompilerProperties("KMC")
    SN64 = CompilerProperties("SN64")

    # iQue
    EGCS = CompilerProperties("EGCS")

    # PS1
    PSYQ = CompilerProperties("PSYQ")

    # PS2
    MWCC = CompilerProperties("MWCC")
    EEGCC = CompilerProperties("EEGCC")

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
