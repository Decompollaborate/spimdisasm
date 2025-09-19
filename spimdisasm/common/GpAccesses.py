#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
from typing import Generator

from .. import common

from .SortedDict import SortedDict


class GotEntry:
    def __init__(self, value: int, isGlobal: bool) -> None:
        self._value = value
        self._isGlobal = isGlobal

    def value(self) -> int:
        return self._value
    def isGlobal(self) -> bool:
        return self._isGlobal


class GlobalOffsetTable:
    def __init__(self) -> None:
        self._table: list[GotEntry] = list()

        self._tableAddress: int|None = None

    def setupTable(self, pltGot: int, table: list[GotEntry]) -> None:
        self._tableAddress = pltGot
        self._table = [entry for entry in table]

    #! @deprecated
    def initTables(self, pltGot: int, localsTable: list[int], globalsTable: list[int]) -> None:
        """
        DEPRECATED: Prefer `setupTable()` instead.
        """
        table = []
        for x in localsTable:
            table.append(GotEntry(x, False))
        for x in globalsTable:
            table.append(GotEntry(x, True))
        self.setupTable(pltGot, table)


    def tableAddress(self) -> int|None:
        return self._tableAddress

    def iterGlobals(self) -> Generator[GotEntry]:
        for entry in self._table:
            if entry._isGlobal:
                yield entry

    def iterLocals(self) -> Generator[GotEntry]:
        for entry in self._table:
            if not entry._isGlobal:
                yield entry


    def _getGotEntry(self, address: int) -> GotEntry|None:
        if self._tableAddress is None:
            return None

        index = (address - self._tableAddress) // 4

        if index < 0:
            common.Utils.eprint(f"Warning: %got address 0x{address:X} not found on local or global table (negative index)")
            common.Utils.eprint(f"    index: {index}, len(table):{len(self._table)}")
            return None

        if index >= len(self._table):
            common.Utils.eprint(f"Warning: %got address 0x{address:X} not found on local or global table (out of range)")
            common.Utils.eprint(f"    index: {index}, len(table): {len(self._table)}")
            return None
        return self._table[index]


@dataclasses.dataclass
class SmallSection:
    address: int
    size: int

    def isInRange(self, address: int) -> bool:
        return self.address <= address < self.address + self.size


@dataclasses.dataclass
class GpAccess:
    address: int

    isGotLocal: bool = False
    isGotGlobal: bool = False
    isSmallSection: bool = False

    @property
    def isGot(self) -> bool:
        return self.isGotLocal or self.isGotGlobal


class GpAccessContainer:
    def __init__(self) -> None:
        self.got = GlobalOffsetTable()
        self.smallSections: SortedDict[SmallSection] = SortedDict()

    def addSmallSection(self, address: int, size: int) -> None:
        self.smallSections[address] = SmallSection(address, size)

    def setupGotTable(self, tableAddress: int, table: list[GotEntry]) -> None:
        self.got.setupTable(tableAddress, table)
        self.addSmallSection(tableAddress, len(table) * 4)

    #! @deprecated
    def initGotTable(self, tableAddress: int, localsTable: list[int], globalsTable: list[int]) -> None:
        """
        DEPRECATED: Prefer `setupGotTable()` instead
        """
        table = []
        for x in localsTable:
            table.append(GotEntry(x, False))
        for x in globalsTable:
            table.append(GotEntry(x, True))
        self.setupGotTable(tableAddress, table)

    def requestAddress(self, address: int) -> GpAccess|None:
        small = self.smallSections.getKeyRight(address)
        if small is None:
            common.Utils.eprint(f"Warning: No section found for $gp access at address 0x{address:08X}")
            return None

        sectionAddr, sectionData = small

        if sectionAddr != self.got._tableAddress:
            if not sectionData.isInRange(address):
                common.Utils.eprint(f"Warning: No section found for $gp access at address 0x{address:08X}")
                return None

            # small section
            access = GpAccess(address)
            access.isSmallSection = True
            return access

        # got
        gotEntry = self.got._getGotEntry(address)
        if gotEntry is None:
            return None

        access = GpAccess(gotEntry._value)
        access.isGotGlobal = gotEntry._isGlobal
        access.isGotLocal = not gotEntry._isGlobal

        return access
