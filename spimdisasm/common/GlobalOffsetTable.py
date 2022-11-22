#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses

from .. import common


@dataclasses.dataclass
class GotSymEntry:
    address: int
    contextSym: common.ContextSymbol|None = None

class GlobalOffsetTable:
    def __init__(self):
        self.localsTable: list[GotSymEntry] = list()
        self.globalsTable: list[GotSymEntry] = list()

        self.tableStart: int|None = None


    def initTables(self, pltGot: int, localsTable: list[int], globalsTable: list[int]):
        self.tableStart = pltGot
        self.localsTable = [GotSymEntry(address) for address in localsTable]
        self.globalsTable = [GotSymEntry(address) for address in globalsTable]


    def getGotSymEntry(self, address: int) -> tuple[GotSymEntry|None, bool|None]:
        if self.tableStart is None:
            return None, None

        index = (address - self.tableStart) // 4

        if index < 0:
            common.Utils.eprint(f"Warning: %got address 0x{address:X} not found on local or global table (negative index)")
            common.Utils.eprint(f"\tindex: {index}, len(localsTable):{len(self.localsTable)}, len(globalsTable): {len(self.globalsTable)}")
            return None, None

        if index < len(self.localsTable):
            return self.localsTable[index], False

        index -= len(self.localsTable)
        if index >= len(self.globalsTable):
            common.Utils.eprint(f"Warning: %got address 0x{address:X} not found on local or global table (out of range)")
            common.Utils.eprint(f"\tindex: {index}, len(globalsTable): {len(self.globalsTable)}")
            return None, None
        return self.globalsTable[index], True
