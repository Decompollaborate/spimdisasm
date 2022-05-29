#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ... import common

from .. import symbols

from . import SectionBase


class SectionData(SectionBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, vram: int, filename: str, array_of_bytes: bytearray, segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, vram, filename, array_of_bytes, common.FileSectionType.Data, segmentVromStart, overlayCategory)


    def analyze(self):
        self.checkAndCreateFirstSymbol()

        symbolList: list[tuple[int, common.ContextSymbol]] = []
        localOffset = 0

        for w in self.words:
            currentVram = self.getVramOffset(localOffset)

            contextSym = self.getSymbol(currentVram, tryPlusOffset=False)
            if contextSym is not None:
                symbolList.append((localOffset, contextSym))

            if w >= self.vram and w > 0x80000000 and w < 0x84000000:
                if self.getSymbol(w, tryPlusOffset=False) is None:
                    self.addPointerInDataReference(w)

            localOffset += 4

        for i, (offset, contextSym) in enumerate(symbolList):
            if i + 1 == len(symbolList):
                words = self.words[offset//4:]
            else:
                nextOffset = symbolList[i+1][0]
                words = self.words[offset//4:nextOffset//4]

            vrom = self.getVromOffset(offset)
            vromEnd = vrom + 4*len(words)
            sym = symbols.SymbolData(self.context, vrom, vromEnd, offset + self.inFileOffset, contextSym.vram, words, self.segmentVromStart, self.overlayCategory)
            sym.parent = self
            sym.setCommentOffset(self.commentOffset)
            sym.analyze()
            self.symbolList.append(sym)


    def removePointers(self) -> bool:
        if not common.GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        for i in range(self.sizew):
            top_byte = (self.words[i] >> 24) & 0xFF
            if top_byte == 0x80:
                self.words[i] = top_byte << 24
                was_updated = True
            if (top_byte & 0xF0) == 0x00 and (top_byte & 0x0F) != 0x00:
                self.words[i] = top_byte << 24
                was_updated = True

        return was_updated
