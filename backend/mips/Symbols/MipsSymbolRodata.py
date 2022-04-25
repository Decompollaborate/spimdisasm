#!/usr/bin/env python3

from __future__ import annotations

from ...common.Utils import *
from ...common.GlobalConfig import GlobalConfig
from ...common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ...common.FileSectionType import FileSectionType

from .MipsSymbolBase import SymbolBase


class SymbolRodata(SymbolBase):
    def __init__(self, context: Context, inFileOffset: int, vram: int|None, name: str, words: list[int]=[]):
        super().__init__(context, inFileOffset, vram, name, words)

        self.sectionType = FileSectionType.Rodata


    def getNthWord(self, i: int) -> Tuple[str, int]:
        localOffset = 4*i
        inFileOffset = self.inFileOffset
        w = self.words[i]

        label = ""
        rodataWord = w
        value: Any = toHex(w, 8)

        # try to get the symbol name from the offset of the file (possibly from a .o elf file)
        possibleSymbolName = self.context.getOffsetGenericSymbol(inFileOffset, FileSectionType.Rodata)
        if possibleSymbolName is not None:
            if possibleSymbolName.isStatic:
                label = "\n/* static variable */"
            label += f"\nglabel {possibleSymbolName.name}\n"

        possibleReference = self.context.getRelocSymbol(inFileOffset, FileSectionType.Rodata)
        if possibleReference is not None:
            value = possibleReference.getNamePlusOffset(w)
            if possibleReference.type == "@jumptablelabel":
                if w in self.context.offsetJumpTablesLabels:
                    value = self.context.offsetJumpTablesLabels[w].name

        isFloat = False
        isDouble = False
        isAsciz = False
        dotType = ".word"
        skip = 0

        if self.vram is not None:
            currentVram = self.getVramOffset(localOffset)

            label += self.getSymbolLabelAtVram(currentVram, label)

            contextVar = self.context.getSymbol(currentVram, True, False)
            if contextVar is not None:
                # Uncomment this line to force unknown rodata to be extracted as strings
                # isAsciz = True
                type = contextVar.type
                if type in ("f32", "Vec3f"):
                    # Filter out NaN and infinity
                    if (w & 0x7F800000) != 0x7F800000:
                        isFloat = True
                    contextVar.isLateRodata = True
                elif type == "f64":
                    # Filter out NaN and infinity
                    if (((w << 32) | self.words[i+1]) & 0x7FF0000000000000) != 0x7FF0000000000000:
                        # Prevent accidentally losing symbols
                        if self.context.getGenericSymbol(currentVram+4, False) is None:
                            isDouble = True
                    contextVar.isLateRodata = True
                elif type == "char":
                    isAsciz = True
                elif GlobalConfig.STRING_GUESSER and contextVar.isMaybeString:
                    isAsciz = True

                if contextVar.vram == currentVram:
                    contextVar.isDefined = True

        if isFloat:
            dotType = ".float"
            value = wordToFloat(w)
        elif isDouble:
            dotType = ".double"
            otherHalf = self.words[i+1]
            value = qwordToDouble((w << 32) | otherHalf)
            rodataWord = (w << 32) | otherHalf
            skip = 1
        elif w in self.context.jumpTablesLabels:
            value = self.context.jumpTablesLabels[w].name
        elif isAsciz:
            try:
                buffer = bytearray(4*len(self.words))
                beWordsToBytes(self.words, buffer)
                decodedValue, rawStringSize = decodeString(buffer, 4*i)
                dotType = ".asciz"
                value = f'"{decodedValue}"'
                value += "\n" + (22 * " ") + ".balign 4"
                rodataWord = None
                skip = rawStringSize // 4
            except (UnicodeDecodeError, RuntimeError):
                # Not a string
                isAsciz = False
                pass

        comment = self.generateAsmLineComment(localOffset, rodataWord)
        return f"{label}{comment} {dotType} {value}", skip


    def disassembleAsRodata(self) -> str:
        if not self.words:
            return ""

        # output = f"glabel {self.name}\n"
        output = ""

        i = 0
        while i < len(self.words):
            data, skip = self.getNthWord(i)
            output += data + "\n"

            i += skip

            i += 1
        return output

    def disassemble(self) -> str:
        return self.disassembleAsRodata()
