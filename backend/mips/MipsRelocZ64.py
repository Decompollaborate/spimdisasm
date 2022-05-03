#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

# Relocation format used by overlays of Zelda64, Yoshi Story and Doubutsu no Mori (Animal Forest)

from __future__ import annotations

from typing import List

from ..common.Utils import *

from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context
from ..common.FileSectionType import FileSectionType, FileSections_ListBasic

from .MipsSection import Section
from .MipsRelocTypes import RelocTypes
from .Symbols import SymbolData


class RelocEntry:
    def __init__(self, entry: int):
        self.sectionId = entry >> 30
        self.relocType = (entry >> 24) & 0x3F
        self.offset = entry & 0x00FFFFFF

    @property
    def reloc(self):
        return (self.sectionId << 30) | (self.relocType << 24) | (self.offset)

    def getSectionType(self) -> FileSectionType:
        return FileSectionType.fromId(self.sectionId)

    def getRelocType(self) -> RelocTypes:
        return RelocTypes.fromValue(self.relocType)

    def __str__(self) -> str:
        section = self.getSectionType().toStr()
        reloc = self.getRelocType().name
        return f"{section} {reloc} 0x{self.offset:X}"
    def __repr__(self) -> str:
        return self.__str__()


class RelocZ64(Section):
    def __init__(self, context: Context, vram: int|None, filename: str, array_of_bytes: bytearray):
        super().__init__(context, vram, filename, array_of_bytes, FileSectionType.Reloc)

        self.seekup = self.words[-1]

        self.setCommentOffset(self.size - self.seekup)

        # Remove non reloc stuff
        self.bytes = self.bytes[-self.seekup:]
        self.words = self.words[-self.seekup // 4:]

        self.sectionSizes = {
            FileSectionType.Text: self.words[0],
            FileSectionType.Data: self.words[1],
            FileSectionType.Rodata: self.words[2],
            FileSectionType.Bss: self.words[3],
        }
        self.relocCount = self.words[4]

        self.tail = self.words[self.relocCount+5:-1]

        self.entries: List[RelocEntry] = list()
        for word in self.words[5:self.relocCount+5]:
            self.entries.append(RelocEntry(word))

        self.differentSegment: bool = False

    @property
    def nRelocs(self) -> int:
        return len(self.entries)

    @property
    def textSize(self) -> int:
        return self.sectionSizes[FileSectionType.Text]
    @property
    def dataSize(self) -> int:
        return self.sectionSizes[FileSectionType.Data]
    @property
    def rodataSize(self) -> int:
        return self.sectionSizes[FileSectionType.Rodata]
    @property
    def bssSize(self) -> int:
        return self.sectionSizes[FileSectionType.Bss]


    def analyze(self):
        if self.vram is not None:
            currentVram = self.getVramOffset(0)
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.name}_OverlayInfo")

            currentVram += 0x14
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.name}_OverlayRelocations")

            currentVram = self.getVramOffset(self.size - 4)
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.name}_OverlayInfoOffset")

        localOffset = 0
        for fileSect in FileSections_ListBasic:
            sectName = fileSect.toCapitalizedStr()

            currentVram = self.getVramOffset(localOffset)
            sym = SymbolData(self.context, localOffset + self.inFileOffset, currentVram, "", [self.sectionSizes[fileSect]])
            sym.setCommentOffset(self.commentOffset)
            sym.endOfLineComment = f" # _{self.name}Segment{sectName}Size"
            sym.analyze()
            self.symbolList.append(sym)
            localOffset += 4

        currentVram = self.getVramOffset(localOffset)
        sym = SymbolData(self.context, localOffset + self.inFileOffset, currentVram, "", [self.relocCount])
        sym.setCommentOffset(self.commentOffset)
        sym.endOfLineComment = f" # reloc_count"
        sym.analyze()
        self.symbolList.append(sym)
        localOffset += 4

        for r in self.entries:
            currentVram = self.getVramOffset(localOffset)
            sym = SymbolData(self.context, localOffset + self.inFileOffset, currentVram, "", [r.reloc])
            sym.setCommentOffset(self.commentOffset)
            sym.endOfLineComment = f" # {str(r)}"
            sym.analyze()
            self.symbolList.append(sym)
            localOffset += 4

        for pad in self.tail:
            currentVram = self.getVramOffset(localOffset)
            sym = SymbolData(self.context, localOffset + self.inFileOffset, currentVram, "", [pad])
            sym.setCommentOffset(self.commentOffset)
            sym.analyze()
            self.symbolList.append(sym)
            localOffset += 4

        currentVram = self.getVramOffset(localOffset)
        sym = SymbolData(self.context, localOffset + self.inFileOffset, currentVram, "", [self.seekup])
        sym.setCommentOffset(self.commentOffset)
        sym.analyze()
        self.symbolList.append(sym)
