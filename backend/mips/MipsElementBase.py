#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context, ContextSymbolBase
from ..common.FileSectionType import FileSectionType


class ElementBase:
    def __init__(self, context: Context, inFileOffset: int, vram: int|None, name: str, words: list[int], sectionType: FileSectionType):
        self.context: Context = context
        self.inFileOffset: int = inFileOffset
        self.vram: int|None = vram
        self.name: str = name
        self.words: list[int] = words
        self.sectionType: FileSectionType = sectionType

        self.commentOffset: int = 0
        self.index: int|None = None

        self.parent: ElementBase|None = None


    @property
    def sizew(self) -> int:
        return len(self.words)

    def setVram(self, vram: int):
        self.vram = vram

    def setCommentOffset(self, commentOffset: int):
        self.commentOffset = commentOffset

    def getVramOffset(self, localOffset: int) -> int:
        if self.vram is None:
            return self.inFileOffset + localOffset
        return self.vram + localOffset
        # return self.vram + self.inFileOffset + localOffset

    def getLabelFromSymbol(self, sym: ContextSymbolBase|None) -> str:
        if sym is not None:
            label = sym.getSymbolLabel()
            if GlobalConfig.GLABEL_ASM_COUNT:
                if self.index is not None:
                    label += f" # {self.index}"
            label += "\n"
            return label
        return ""


    def analyze(self):
        pass


    def disassemble(self) -> str:
        return ""
