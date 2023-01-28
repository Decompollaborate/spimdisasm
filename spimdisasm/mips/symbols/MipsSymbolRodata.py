#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import rabbitizer

from ... import common
from ... import elf32

from . import SymbolBase


class SymbolRodata(SymbolBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, inFileOffset: int, vram: int, words: list[int], segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, inFileOffset, vram, words, common.FileSectionType.Rodata, segmentVromStart, overlayCategory)


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
        if self.contextSym.forceMigration:
            return True

        if self.contextSym.forceNotMigration:
            return False

        if self.isRdata():
            if common.GlobalConfig.COMPILER not in {common.Compiler.SN64, common.Compiler.PSYQ}:
                return False

        return True


    def analyze(self):
        if self.contextSym.isDouble():
            if self.sizew % 2 != 0:
                # doubles require an even amount of words
                self.contextSym.type = None
            else:
                for i in range(self.sizew // 2):
                    if not self.isDouble(i*2):
                        # checks there's no other overlaping symbols
                        self.contextSym.type = None
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

    def getNthWord(self, i: int, canReferenceSymbolsWithAddends: bool=False, canReferenceConstants: bool=False) -> tuple[str, int]:
        localOffset = 4*i
        w = self.words[i]
        vrom = self.getVromOffset(localOffset)

        label = ""
        rodataWord: int|None = w
        value: str = f"0x{w:08X}"

        dotType = ".word"
        skip = 0

        relocInfo = self.context.globalRelocationOverrides.get(vrom)
        if relocInfo is not None:
            if relocInfo.staticReference is not None:
                relocVram = relocInfo.staticReference.sectionVram + w
                labelSym = self.getSymbol(relocVram, tryPlusOffset=False)
                if labelSym is not None:
                    relocInfo.symbol = labelSym.getName()
            comment = self.generateAsmLineComment(localOffset, rodataWord)
            return f"{label}{comment} {relocInfo.getNameWithReloc()}{common.GlobalConfig.LINE_ENDS}", skip

        if self.contextSym.isJumpTable():
            if self.contextSym.isGot and common.GlobalConfig.GP_VALUE is not None:
                labelAddr = common.GlobalConfig.GP_VALUE + rabbitizer.Utils.from2Complement(w, 32)
                labelSym = self.getSymbol(labelAddr, tryPlusOffset=False)
                if labelSym is not None and labelSym.type == common.SymbolSpecialType.jumptablelabel:
                    dotType = ".gpword"
            else:
                labelSym = self.getSymbol(w, tryPlusOffset=False)

            if labelSym is not None and labelSym.type == common.SymbolSpecialType.jumptablelabel:
                value = labelSym.getName()
        else:
            labelSym = self.getSymbol(w, tryPlusOffset=canReferenceSymbolsWithAddends)
            if labelSym is not None and not self.context.isAddressBanned(labelSym.vram):
                value = labelSym.getSymbolPlusOffset(w)

        comment = self.generateAsmLineComment(localOffset, rodataWord)
        return f"{label}{comment} {dotType} {value}{common.GlobalConfig.LINE_ENDS}", skip
