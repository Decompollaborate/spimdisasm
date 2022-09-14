#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations


class GlobalOffsetTable:
    def __init__(self):
        self.localsTable: list[int] = list()
        self.globalsTable: list[int] = list()

        self.tableStart: int|None = None


    def getAddress(self, address: int) -> int|None:
        if self.tableStart is None:
            return None

        index = (address - self.tableStart) // 4
        if index < len(self.localsTable):
            return self.localsTable[index]

        index -= len(self.localsTable)
        return self.globalsTable[index]
