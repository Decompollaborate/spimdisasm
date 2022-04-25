#!/usr/bin/env python3

from __future__ import annotations

from ...common.Utils import *
from ...common.GlobalConfig import GlobalConfig
from ...common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ...common.FileSectionType import FileSectionType

from ..MipsElementBase import ElementBase


class SymbolBase(ElementBase):
    def generateAsmLineComment(self, localOffset: int, wordValue: int|None = None) -> str:
        if not GlobalConfig.ASM_COMMENT:
            return ""
        offsetHex = f"{localOffset + self.inFileOffset + self.commentOffset:06X}"

        vramHex = ""
        if self.vram is not None:
            currentVram = self.getVramOffset(localOffset)
            vramHex = f"{currentVram:08X} "

        wordValueHex = ""
        if wordValue is not None:
            wordValueHex = f"{wordValue:08X} "

        return f"/* {offsetHex} {vramHex}{wordValueHex}*/"


    def disassembleAsData(self) -> str:
        output = ""

        localOffset = 0
        inFileOffset = self.inFileOffset
        i = 0
        while i < self.sizew:
            w = self.words[i]

            label = self.getSymbolLabelAtOffset(inFileOffset, "")

            # if we have vram available, try to get the symbol name from the Context
            if self.vram is not None:
                currentVram = self.getVramOffset(localOffset)

                label = self.getSymbolLabelAtVram(currentVram, label)

                contVariable = self.context.getSymbol(currentVram, False)
                if contVariable is not None:
                    contVariable.isDefined = True

            value = f"0x{w:08X}"
            possibleReference = self.context.getRelocSymbol(inFileOffset, self.sectionType)
            if possibleReference is not None:
                value = possibleReference.getNamePlusOffset(w)

            symbol = self.context.getAnySymbol(w)
            if symbol is not None:
                value = symbol.name

            comment = self.generateAsmLineComment(localOffset)
            line = f"{label}{comment} .word {value}"
            output += line + "\n"
            i += 1
            localOffset += 4
            inFileOffset += 4

        return output


    def disassemble(self) -> str:
        return self.disassembleAsData()
