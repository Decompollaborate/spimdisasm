#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ... import common

from . import SymbolBase


class SymbolData(SymbolBase):
    def __init__(self, context: common.Context, vrom: int, inFileOffset: int, vram: int, words: list[int]):
        super().__init__(context, vrom, inFileOffset, vram, words, common.FileSectionType.Data)
