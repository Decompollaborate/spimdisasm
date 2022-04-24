#!/usr/bin/env python3

from __future__ import annotations

from ...common.Utils import *
from ...common.GlobalConfig import GlobalConfig
from ...common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ...common.FileSectionType import FileSectionType

from .MipsSymbolBase import SymbolBase


class SymbolData(SymbolBase):
    def __init__(self, context: Context, name: str, inFileOffset: int, vram: int|None, words: list[int]=[]):
        super().__init__(context, name, inFileOffset, vram, words)

        self.sectionType = FileSectionType.Data
