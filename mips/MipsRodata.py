#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section
from .MipsContext import Context, ContextSymbol


class Rodata(Section):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, context: Context):
        super().__init__(array_of_bytes, filename, version, context)

        # addresses of symbols in this rodata section
        self.symbolsVRams: Set[int] = set()


    def analyze(self):
        if self.vRamStart > -1:
            offset = 0
            partOfJumpTable = False
            for w in self.words:
                currentVram = self.getVramOffset(offset)
                if currentVram in self.context.jumpTables:
                    partOfJumpTable = True

                elif partOfJumpTable:
                    if offset in self.pointersOffsets:
                        partOfJumpTable = True

                    elif self.context.getGenericSymbol(currentVram) is not None:
                        partOfJumpTable = False

                    elif ((w >> 24) & 0xFF) != 0x80:
                        partOfJumpTable = False

                if partOfJumpTable:
                    if w not in self.context.jumpTablesLabels:
                        self.context.jumpTablesLabels[w] = f"L{toHex(w, 8)[2:]}"
                elif currentVram in self.context.newPointersInData:
                    if self.context.getAnySymbol(currentVram) is None:
                        contextSym = ContextSymbol(currentVram, "D_" + toHex(currentVram, 8)[2:])
                        try:
                            decodeString(self.bytes, offset)
                            if self.bytes[offset] != 0:
                                # Filter out empty strings
                                contextSym.type = "char"
                        except UnicodeDecodeError:
                            pass
                        self.context.symbols[currentVram] = contextSym
                        self.context.newPointersInData.remove(currentVram)

                auxLabel = self.context.getGenericLabel(currentVram)
                if auxLabel is not None:
                    self.symbolsVRams.add(currentVram)

                contextSymbol = self.context.getSymbol(currentVram, tryPlusOffset=False)
                if contextSymbol is not None:
                    self.symbolsVRams.add(currentVram)
                    contextSymbol.isDefined = True

                offset += 4


    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
             False

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

    def getNthWord(self, i: int) -> Tuple[str, int]:
        offset = i * 4
        w = self.words[i]

        offsetHex = toHex(offset + self.commentOffset, 6)[2:]
        vramHex = ""
        label = ""
        rodataHex = toHex(w, 8)[2:]
        value: Any = toHex(w, 8)

        isFloat = False
        isDouble = False
        isAsciz = False
        dotType = ".word"
        skip = 0

        if self.vRamStart > -1:
            currentVram = self.getVramOffset(offset)
            vramHex = toHex(currentVram, 8)[2:]

            auxLabel = self.context.getGenericLabel(currentVram)
            if auxLabel is None:
                auxLabel = self.context.getGenericSymbol(currentVram, tryPlusOffset=False)
            if auxLabel is not None:
                label = "\nglabel " + auxLabel + "\n"

            contextVar = self.context.getSymbol(currentVram, True, False)
            if contextVar is not None:
                type = contextVar.type
                if type in ("f32", "Vec3f"):
                    # Filter out NaN and infinity
                    if (w & 0x7F800000) != 0x7F800000:
                        isFloat = True
                elif type == "f64":
                    # Filter out NaN and infinity
                    if (((w << 32) | self.words[i+1]) & 0x7FF0000000000000) != 0x7FF0000000000000:
                        # Prevent accidentally losing symbols
                        if self.context.getGenericSymbol(currentVram+4, False) is None:
                            isDouble = True
                elif type == "char":
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
            rodataHex += toHex(otherHalf, 8)[2:]
            skip = 1
        elif isAsciz:
            try:
                decodedValue, rawStringSize = decodeString(self.bytes, 4*i)
                dotType = ".asciz"
                value = f'"{decodedValue}"'
                value += "\n" + (24 * " ") + ".balign 4"
                rodataHex = ""
                skip = rawStringSize // 4
            except UnicodeDecodeError:
                # Not a string
                isAsciz = False
                pass
        elif w in self.context.jumpTablesLabels:
            value = self.context.jumpTablesLabels[w]

        comment = ""
        if GlobalConfig.ASM_COMMENT:
            comment = f"/* {offsetHex} {vramHex} {rodataHex} */ "

        return f"{label}{comment} {dotType}  {value}", skip


    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".rodata")

        if self.size == 0:
            return

        with open(filepath + ".rodata.s", "w") as f:
            f.write(".include \"macro.inc\"\n")
            f.write("\n")
            f.write("# assembler directives\n")
            f.write(".set noat      # allow manual use of $at\n")
            f.write(".set noreorder # don't insert nops after branches\n")
            f.write(".set gp=64     # allow use of 64-bit general purpose registers\n")
            f.write("\n")
            f.write(".section .rodata\n")
            f.write("\n")
            f.write(".balign 16\n")

            i = 0
            while i < len(self.words):
                data, skip = self.getNthWord(i)
                f.write(data + "\n")

                i += skip

                i += 1
