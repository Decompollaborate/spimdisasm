#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import rabbitizer

from ... import common

from .. import symbols

from . import SectionBase


class SectionRodata(SectionBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, vram: int, filename: str, array_of_bytes: bytes, segmentVromStart: int, overlayCategory: str|None) -> None:
        if common.GlobalConfig.ENDIAN_RODATA is not None:
            words = common.Utils.endianessBytesToWords(common.GlobalConfig.ENDIAN_RODATA, array_of_bytes, vromStart, vromEnd)
        else:
            words = common.Utils.bytesToWords(array_of_bytes, vromStart, vromEnd)
        super().__init__(context, vromStart, vromEnd, vram, filename, words, common.FileSectionType.Rodata, segmentVromStart, overlayCategory)

        self.stringEncoding = common.GlobalConfig.RODATA_STRING_ENCODING


    def _analyze_processJumptable(self, localOffset: int, w: int, contextSym: common.ContextSymbol|None, lastVramSymbol: common.ContextSymbol, jumpTableSym: common.ContextSymbol|None, firstJumptableWord: int) -> tuple[common.ContextSymbol|None, int]:
        if contextSym is not None and contextSym.isJumpTable():
            # New jumptable
            jumpTableSym = contextSym
            firstJumptableWord = w

        elif jumpTableSym is not None:
            # The last symbol found was part of a jumptable, check if this word still is part of the jumptable

            if localOffset not in self.pointersOffsets:
                if w == 0:
                    return None, firstJumptableWord

                elif contextSym is not None:
                    return None, firstJumptableWord

                elif ((w >> 24) & 0xFF) != ((firstJumptableWord >> 24) & 0xFF):
                    if not (
                        lastVramSymbol.isJumpTable()
                        and lastVramSymbol.isGot
                        and common.GlobalConfig.GP_VALUE is not None
                    ):
                        return None, firstJumptableWord
        else:
            # No jumptable
            return None, firstJumptableWord

        # Generate the current label
        labelAddr = w
        if lastVramSymbol.isGot and common.GlobalConfig.GP_VALUE is not None:
            labelAddr = common.GlobalConfig.GP_VALUE + rabbitizer.Utils.from2Complement(w, 32)
        labelSym = self.addJumpTableLabel(labelAddr, isAutogenerated=True)

        if labelSym.unknownSegment:
            return None, firstJumptableWord

        labelSym.referenceCounter += 1
        if jumpTableSym.parentFunction is not None:
            labelSym.parentFunction = jumpTableSym.parentFunction
            labelSym.parentFileName = jumpTableSym.parentFunction.parentFileName
            jumpTableSym.parentFunction.branchLabels.add(labelSym.vram, labelSym)

        return jumpTableSym, firstJumptableWord


    def analyze(self) -> None:
        lastVramSymbol: common.ContextSymbol = self._checkAndCreateFirstSymbol()

        symbolList: list[tuple[int, common.ContextSymbol]] = []
        localOffset = 0
        localOffsetsWithSymbols: set[int] = set()

        needsFurtherAnalyzis = False

        jumpTableSym: common.ContextSymbol|None = None
        firstJumptableWord = -1

        for w in self.words:
            currentVram = self.getVramOffset(localOffset)
            currentVrom = self.getVromOffset(localOffset)

            # Check if we have a symbol at this address, if not then use the last known one
            contextSym = self.getSymbol(currentVram, vromAddress=currentVrom, tryPlusOffset=False)
            if contextSym is not None:
                lastVramSymbol = contextSym

            jumpTableSym, firstJumptableWord = self._analyze_processJumptable(localOffset, w, contextSym, lastVramSymbol, jumpTableSym, firstJumptableWord)

            if jumpTableSym is None:
                if contextSym is not None or self.popPointerInDataReference(currentVram) is not None or (lastVramSymbol.isJumpTable() and w != 0):
                    contextSym = self._addOwnedSymbol(localOffset)
                    lastVramSymbol = contextSym

                self.checkWordIsASymbolReference(w)

            if contextSym is not None:
                symbolList.append((localOffset, contextSym))
                localOffsetsWithSymbols.add(localOffset)

                self._createAutoPadFromSymbol(localOffset, contextSym)

            elif jumpTableSym is None and self.popPointerInDataReference(currentVram) is not None:
                contextSym = self._addOwnedSymbol(localOffset)
                symbolList.append((localOffset, contextSym))
                localOffsetsWithSymbols.add(localOffset)

            if not lastVramSymbol.notPointerByType():
                if self.checkWordIsASymbolReference(w):
                    if w < currentVram and self.containsVram(w):
                        # References a data symbol from this section and it is behind this current symbol
                        needsFurtherAnalyzis = True

            localOffset += 4

        if needsFurtherAnalyzis:
            localOffset = 0
            for w in self.words:
                currentVram = self.getVramOffset(localOffset)

                if self.popPointerInDataReference(currentVram) is not None and localOffset not in localOffsetsWithSymbols:
                    contextSym = self._addOwnedSymbol(localOffset)
                    symbolList.append((localOffset, contextSym))
                    localOffsetsWithSymbols.add(localOffset)

                localOffset += 4

            # Since we appended new symbols, this list is not sorted anymore
            symbolList.sort()

        previousSymbolWasLateRodata = False
        previousSymbolExtraPadding = 0

        for i, (offset, contextSym) in enumerate(symbolList):
            if i + 1 == len(symbolList):
                words = self.words[offset//4:]
            else:
                nextOffset = symbolList[i+1][0]
                words = self.words[offset//4:nextOffset//4]

            vrom = self.getVromOffset(offset)
            vromEnd = vrom + len(words)*4
            sym = symbols.SymbolRodata(self.context, vrom, vromEnd, offset + self.inFileOffset, contextSym.vram, words, self.segmentVromStart, self.overlayCategory)
            sym.parent = self
            sym.setCommentOffset(self.commentOffset)
            sym.stringEncoding = self.stringEncoding
            sym.analyze()
            self.symbolList.append(sym)
            self.symbolsVRams.add(contextSym.vram)

            # File boundaries detection
            if sym.inFileOffset % 16 == 0:
                # Files are always 0x10 aligned

                if previousSymbolWasLateRodata and not sym.contextSym.isLateRodata():
                    # late rodata followed by normal rodata implies a file split
                    self.fileBoundaries.append(sym.inFileOffset)
                elif previousSymbolExtraPadding > 0:
                    if sym.isDouble(0):
                        # doubles require a bit extra of alignment
                        if previousSymbolExtraPadding >= 2:
                            self.fileBoundaries.append(sym.inFileOffset)
                    elif sym.isJumpTable() and common.GlobalConfig.COMPILER.value.prevAlign_jumptable is not None and common.GlobalConfig.COMPILER.value.prevAlign_jumptable >= 3:
                        if previousSymbolExtraPadding >= 2:
                            self.fileBoundaries.append(sym.inFileOffset)
                    elif sym.isString() and common.GlobalConfig.COMPILER.value.prevAlign_string is not None and common.GlobalConfig.COMPILER.value.prevAlign_string >= 3:
                        if previousSymbolExtraPadding >= 2:
                            self.fileBoundaries.append(sym.inFileOffset)
                    else:
                        self.fileBoundaries.append(sym.inFileOffset)

            previousSymbolWasLateRodata = sym.contextSym.isLateRodata()
            previousSymbolExtraPadding = sym.countExtraPadding()

        self.processStaticRelocs()

        # Filter out repeated values and sort
        self.fileBoundaries = sorted(set(self.fileBoundaries))


    def removePointers(self) -> bool:
        if not common.GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = super().removePointers()
        for i in range(self.sizew):
            top_byte = (self.words[i] >> 24) & 0xFF
            if top_byte == 0x80:
                self.words[i] = top_byte << 24
                was_updated = True
            if (top_byte & 0xF0) == 0x00 and (top_byte & 0x0F) != 0x00:
                self.words[i] = top_byte << 24
                was_updated = True

        return was_updated
