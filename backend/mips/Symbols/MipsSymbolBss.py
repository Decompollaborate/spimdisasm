#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ...common.Utils import *
from ...common.GlobalConfig import GlobalConfig
from ...common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ...common.FileSectionType import FileSectionType

from .MipsSymbolBase import SymbolBase


class SymbolBss(SymbolBase):
    def __init__(self, context: Context, inFileOffset: int, vram: int|None, name: str, spaceSize: int):
        super().__init__(context, inFileOffset, vram, name)

        self.spaceSize: int = spaceSize

        self.sectionType = FileSectionType.Bss


    def disassembleAsBss(self) -> str:
        output = self.getLabel()
        output += self.generateAsmLineComment(0)
        output += f" .space 0x{self.spaceSize:02X}\n"
        return output

    def disassemble(self) -> str:
        return self.disassembleAsBss()
