#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from .GlobalConfig import GlobalConfig
from .Context import Context, ContextSymbol
from .FileSectionType import FileSectionType


class ElementBase:
    """Represents the base class used for most file sections and symbols.
    """

    def __init__(self, context: Context, vromStart: int, vromEnd: int, inFileOffset: int, vram: int, name: str, words: list[int], sectionType: FileSectionType, segmentVromStart: int, overlayType: str|None):
        """Constructor

        Args:
            context (Context):
            vromStart (int): The VROM address of this element
            vromEnd (int): The end of this element's VROM address
            inFileOffset (int): The offset of this element relative to the start of its file. It is also used to generate the first column of the disassembled line comment
            vram (int): The VRAM address of this element
            name (str): The name of this element
            words (list[int]): A list of words (4 bytes) corresponding to this element
            sectionType (FileSectionType): The section type this element corresponds to
        """

        self.context: Context = context
        self.vromStart: int = vromStart
        self.vromEnd: int = vromEnd
        self.inFileOffset: int = inFileOffset
        self.vram: int = vram
        self.name: str = name
        self.words: list[int] = words
        self.sectionType: FileSectionType = sectionType

        self.commentOffset: int = 0
        "This value is added to the first column of the disassembled line comment, allowing to change this value without messing inFileOffset"

        self.index: int|None = None
        "The index of the current element inside its parent or `None` if the index is unknown"

        self.parent: ElementBase|None = None
        "For elements that are contained in other elements, like symbols inside of sections"

        self.overlayType: str|None = overlayType
        self.segmentVromStart: int = segmentVromStart


    @property
    def sizew(self) -> int:
        "The amount of words this element has"
        return len(self.words)

    @property
    def vramEnd(self) -> int:
        "The end of this element's VRAM"
        return self.vram + self.sizew * 4


    def setVram(self, vram: int):
        self.vram = vram

    def setCommentOffset(self, commentOffset: int):
        self.commentOffset = commentOffset

    def getVromOffset(self, localOffset: int) -> int:
        return self.vromStart + localOffset

    def getVramOffset(self, localOffset: int) -> int:
        return self.vram + localOffset


    def getLabelFromSymbol(self, sym: ContextSymbol|None) -> str:
        "Generates a glabel for the passed symbol, including an optional index value if it was set and it is enabled in the GlobalConfig"
        if sym is not None:
            label = sym.getSymbolLabel()
            if GlobalConfig.GLABEL_ASM_COUNT:
                if self.index is not None:
                    label += f" # {self.index}"
            label +=  GlobalConfig.LINE_ENDS
            return label
        return ""


    def analyze(self):
        """Scans the words of this element, gathering as much info as possible.

        This method should be called only once for each element.
        """
        pass


    def disassemble(self) -> str:
        """Produces a disassembly of this element.

        Elements assume the `analyze` method was already called at this point.

        This method can be called as many times as the user wants to.
        """
        return ""
