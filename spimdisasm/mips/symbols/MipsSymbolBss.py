#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ... import common

from . import SymbolBase


class SymbolBss(SymbolBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, inFileOffset: int, vram: int, spaceSize: int, segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, inFileOffset, vram, list(), common.FileSectionType.Bss, segmentVromStart, overlayCategory)

        self.spaceSize: int = spaceSize


    @property
    def sizew(self) -> int:
        return self.spaceSize // 4

    def disassembleAsBss(self, useGlobalLabel: bool=True) -> str:
        output = self.getReferenceeSymbols()
        output += self.getPrevAlignDirective(0)

        output += self.getSymbolAsmDeclaration(self.getName(), useGlobalLabel)
        output += self.generateAsmLineComment(0)
        output += f" .space 0x{self.spaceSize:02X}{common.GlobalConfig.LINE_ENDS}"

        nameEnd = self.getNameEnd()
        if nameEnd is not None:
            output += self.getSymbolAsmDeclaration(nameEnd, useGlobalLabel)

        return output

    def disassemble(self, migrate: bool=False, useGlobalLabel: bool=True) -> str:
        output = ""

        if migrate:
            output += self.getSpimdisasmVersionString()

        output = self.disassembleAsBss()
        return output
