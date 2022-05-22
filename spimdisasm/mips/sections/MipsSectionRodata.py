#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ... import common

from .. import symbols

from . import SectionBase


class SectionRodata(SectionBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, vram: int, filename: str, array_of_bytes: bytearray, segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, vram, filename, array_of_bytes, common.FileSectionType.Rodata, segmentVromStart, overlayCategory)

        self.bytes: bytearray = bytearray(self.sizew*4)
        common.Utils.beWordsToBytes(self.words, self.bytes)


    def _stringGuesser(self, contextSym: common.ContextSymbol, localOffset: int) -> bool:
        if contextSym.isMaybeString or contextSym.isString():
            return True

        if not common.GlobalConfig.STRING_GUESSER:
            return False

        if not contextSym.hasNoType() or contextSym.referenceCounter > 1:
            return False

        # This would mean the string is an empty string, which is not very likely
        if self.bytes[localOffset] == 0:
            return False

        try:
            common.Utils.decodeString(self.bytes, localOffset)
        except (UnicodeDecodeError, RuntimeError):
            # String can't be decoded
            return False
        return True

    def _processElfRelocSymbols(self) -> None:
        if len(self.context.relocSymbols[self.sectionType]) == 0:
            return

        # Process reloc symbols (probably from a .elf file)
        inFileOffset = self.inFileOffset
        for w in self.words:
            relocSymbol = self.context.getRelocSymbol(inFileOffset, self.sectionType)
            if relocSymbol is not None:
                if relocSymbol.name is not None and relocSymbol.name.startswith("."):
                    sectType = common.FileSectionType.fromStr(relocSymbol.name)
                    relocSymbol.sectionType = sectType

                    relocName = f"{relocSymbol.name}_{w:06X}"
                    contextOffsetSym = common.ContextOffsetSymbol(w, relocName, sectType)
                    if sectType == common.FileSectionType.Text:
                        # jumptable
                        relocName = f"L{w:06X}"
                        contextOffsetSym = self.context.addOffsetJumpTableLabel(w, relocName, common.FileSectionType.Text)
                        relocSymbol.type = contextOffsetSym.type
                        offsetSym = self.context.getOffsetSymbol(inFileOffset, self.sectionType)
                        if offsetSym is not None:
                            offsetSym.type = common.SymbolSpecialType.jumptable
                    self.context.offsetSymbols[sectType][w] = contextOffsetSym
                    relocSymbol.name = relocName
                    # print(relocSymbol.name, f"{w:X}")
            inFileOffset += 4

    def analyze(self):
        self.checkAndCreateFirstSymbol()

        symbolList = []
        localOffset = 0

        partOfJumpTable = False
        for w in self.words:
            currentVram = self.getVramOffset(localOffset)
            contextSym = self.getSymbol(currentVram, tryPlusOffset=False)

            if contextSym is not None and contextSym.isJumpTable():
                partOfJumpTable = True

            elif partOfJumpTable:
                if localOffset in self.pointersOffsets:
                    partOfJumpTable = True

                elif contextSym is not None:
                    partOfJumpTable = False

                elif ((w >> 24) & 0xFF) != 0x80:
                    partOfJumpTable = False

            if partOfJumpTable:
                labelSym = self.addJumpTableLabel(w, isAutogenerated=True)
                labelSym.referenceCounter += 1

            elif self.popPointerInDataReference(currentVram) is not None:
                if common.GlobalConfig.ADD_NEW_SYMBOLS:
                    contextSym = self.addSymbol(currentVram, self.sectionType, isAutogenerated=True)
                    contextSym.isMaybeString = self._stringGuesser(contextSym, localOffset)

            elif contextSym is not None:
                contextSym.isMaybeString = self._stringGuesser(contextSym, localOffset)

            contextSym = self.getSymbol(currentVram, tryPlusOffset=False)
            if contextSym is not None:
                self.symbolsVRams.add(currentVram)

                symbolList.append((localOffset, currentVram))

            localOffset += 4

        previousSymbolWasLateRodata = False
        previousSymbolExtraPadding = 0

        for i, (offset, vram) in enumerate(symbolList):
            if i + 1 == len(symbolList):
                words = self.words[offset//4:]
            else:
                nextOffset = symbolList[i+1][0]
                words = self.words[offset//4:nextOffset//4]

            vrom = self.getVromOffset(offset)
            vromEnd = vrom + len(words)*4
            sym = symbols.SymbolRodata(self.context, vrom, vromEnd, offset + self.inFileOffset, vram, words, self.segmentVromStart, self.overlayCategory)
            sym.parent = self
            sym.setCommentOffset(self.commentOffset)
            sym.analyze()
            self.symbolList.append(sym)

            # file boundary detection
            if sym.inFileOffset % 16 == 0:
                if previousSymbolWasLateRodata and not sym.contextSym.isLateRodata():
                    # late rodata followed by normal rodata implies a file split
                    self.fileBoundaries.append(sym.inFileOffset)
                elif previousSymbolExtraPadding > 0:
                    if sym.contextSym.isDouble():
                        # doubles require a bit extra of alignment
                        if previousSymbolExtraPadding >= 2:
                            self.fileBoundaries.append(sym.inFileOffset)
                    else:
                        self.fileBoundaries.append(sym.inFileOffset)

            previousSymbolWasLateRodata = sym.contextSym.isLateRodata()
            previousSymbolExtraPadding = sym.countExtraPadding()

        self._processElfRelocSymbols()


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
