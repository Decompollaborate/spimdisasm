#!/usr/bin/env python3

from __future__ import annotations

from ...common.Utils import *
from ...common.GlobalConfig import GlobalConfig
from ...common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ...common.FileSectionType import FileSectionType


class SymbolBase:
    def __init__(self, context: Context, name: str, inFileOffset: int, vram: int|None, words: list[int]=[]):
        self.context: Context = context
        self.name: str = name
        self.inFileOffset: int = inFileOffset
        self.vram: int|None = vram
        self.words: list[int] = words

        self.commentOffset: int = 0
        self.index: int = -1

        self.parent: Any = None

        self.sectionType: FileSectionType = FileSectionType.Unknown

    @property
    def sizew(self) -> int:
        return len(self.words)

    def setCommentOffset(self, commentOffset: int):
        self.commentOffset = commentOffset

    # TODO: avoid duplicated code here and in MipsFileBase
    def getVramOffset(self, localOffset: int) -> int:
        if self.vram is None:
            return self.inFileOffset + localOffset
        return self.vram + localOffset

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


    def analyze(self):
        pass


    def disassembleAsData(self) -> str:
        output = f"\nglabel {self.name}\n"

        localOffset = 0
        inFileOffset = self.inFileOffset
        i = 0
        while i < self.sizew:
            w = self.words[i]

            label = ""
            if i != 0 or self.name == "":
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
