#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ... import common

from . import SymbolBase


class SymbolRodata(SymbolBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, inFileOffset: int, vram: int, words: list[int], segmentVromStart: int, overlayCategory: str|None) -> None:
        super().__init__(context, vromStart, vromEnd, inFileOffset, vram, words, common.FileSectionType.Rodata, segmentVromStart, overlayCategory)

        self.stringEncoding = common.GlobalConfig.RODATA_STRING_ENCODING

    def isJumpTable(self) -> bool:
        # jumptables must have at least 3 labels
        if self.sizew < 3:
            return False
        return self.contextSym.isJumpTable()


    def isMaybeConstVariable(self) -> bool:
        if self.isFloat(0):
            if self.sizew > 1:
                for w in self.words[1:]:
                    if w != 0:
                        return True
            return False
        elif self.isDouble(0):
            if self.sizew > 2:
                for w in self.words[2:]:
                    if w != 0:
                        return True
            return False
        elif self.isJumpTable():
            return False
        elif self.isString():
            return False
        elif self.isPascalString():
            return False
        return True

    def isRdata(self) -> bool:
        "Checks if the current symbol is .rdata"
        if self.isMaybeConstVariable():
            return True

        # This symbol could be an unreferenced non-const variable
        if len(self.contextSym.referenceFunctions) == 1:
            # This const variable was already used in a function
            return False

        return True

    def shouldMigrate(self) -> bool:
        if self.contextSym.functionOwnerForMigration is not None:
            return True

        if self.contextSym.forceMigration:
            return True

        if self.contextSym.forceNotMigration:
            return False

        if self.contextSym.isMips1Double:
            return True

        if len(self.contextSym.referenceSymbols) > 0:
            return False

        if self.isRdata():
            if not common.GlobalConfig.COMPILER.value.allowRdataMigration:
                return False

        return True


    def analyze(self) -> None:
        if self.contextSym.isDouble():
            if self.sizew % 2 != 0:
                # doubles require an even amount of words
                self.contextSym.setTypeSpecial(None, isAutogenerated=True)
            else:
                for i in range(self.sizew // 2):
                    if not self.isDouble(i*2):
                        # checks there's no other overlaping symbols
                        self.contextSym.setTypeSpecial(None, isAutogenerated=True)
                        break

        super().analyze()


    def countExtraPadding(self) -> int:
        if self.contextSym.hasUserDeclaredSize():
            if self.sizew * 4 == self.contextSym.getSize():
                return 0

        count = 0
        if self.isString():
            for i in range(len(self.words)-1, 0, -1):
                if self.words[i] != 0:
                    break
                if (self.words[i-1] & 0x000000FF) != 0:
                    break
                count += 1
        elif self.isDouble(0):
            for i in range(len(self.words)-1, 0, -2):
                if self.words[i] != 0 or self.words[i-1] != 0:
                    break
                count += 2
        else:
            for i in range(len(self.words)-1, 0, -1):
                if self.words[i] != 0:
                    break
                count += 1
        return count
