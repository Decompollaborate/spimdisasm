#!/usr/bin/env python3

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ..common.FileSectionType import FileSectionType


class ElementBase:
    def __init__(self, context: Context, inFileOffset: int, vram: int|None, name: str, words: list[int]=[]):
        self.context: Context = context
        self.inFileOffset: int = inFileOffset
        self.vram: int|None = vram
        self.name: str = name
        self.words: list[int] = words

        self.commentOffset: int = 0
        self.index: int|None = None

        self.parent: ElementBase|None = None

        self.sectionType: FileSectionType = FileSectionType.Unknown

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

    def getSymbolLabelAtVram(self, vram: int, fallback="") -> str:
        # if we have vram available, try to get the symbol name from the Context
        if self.vram is not None:
            sym = self.context.getAnySymbol(vram)
            if sym is not None:
                label = ""
                if sym.isStatic:
                    label += "\n/* static variable */"
                label += "\nglabel " + sym.getSymbolPlusOffset(vram) + "\n"
                return label
        return fallback

    def getSymbolLabelAtOffset(self, inFileOffset: int, fallback="") -> str:
        # try to get the symbol name from the offset of the file (possibly from a .o elf file)
        possibleSymbolName = self.context.getOffsetSymbol(inFileOffset, self.sectionType)
        if possibleSymbolName is not None:
            label = ""
            if possibleSymbolName.isStatic:
                label = "\n/* static variable */"
            label += f"\nglabel {possibleSymbolName.name}\n"
            return label
        return fallback


    def analyze(self):
        pass


    def disassemble(self) -> str:
        return ""
