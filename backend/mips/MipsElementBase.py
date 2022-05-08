#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from .. import common


class ElementBase:
    def __init__(self, context: common.Context, inFileOffset: int, vram: int|None, name: str, words: list[int], sectionType: common.FileSectionType):
        self.context: common.Context = context
        self.inFileOffset: int = inFileOffset
        self.vram: int|None = vram
        self.name: str = name
        self.words: list[int] = words
        self.sectionType: common.FileSectionType = sectionType

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

    def getLabelFromSymbol(self, sym: common.ContextSymbolBase|None) -> str:
        if sym is not None:
            label = sym.getSymbolLabel()
            if common.GlobalConfig.GLABEL_ASM_COUNT:
                if self.index is not None:
                    label += f" # {self.index}"
            label += "\n"
            return label
        return ""


    def analyze(self):
        pass


    def disassemble(self) -> str:
        return ""
