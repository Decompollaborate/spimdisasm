#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section
from .MipsContext import Context


class Rodata(Section):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, context: Context):
        super().__init__(array_of_bytes, filename, version, context)

        # addresses of symbols in this rodata section
        self.symbolsVRams: Set[int] = set()


    def analyze(self):
        offset = 0
        partOfJumpTable = False
        if self.vRamStart != -1:
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
                        self.context.jumpTablesLabels[w] = f"jmplabel_{toHex(w, 8)[2:]}"

                auxLabel = self.context.getGenericLabel(currentVram) or self.context.getGenericSymbol(currentVram, tryPlusOffset=False)
                if auxLabel is not None:
                    self.symbolsVRams.add(currentVram)

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

    def getNthWord(self, i: int) -> str:
        offset = i * 4
        w = self.words[i]

        offsetHex = toHex(offset + self.commentOffset, 6)[2:]
        vramHex = ""
        label = ""
        rodataHex = toHex(w, 8)[2:]
        value = toHex(w, 8)

        if self.vRamStart != -1:
            currentVram = self.getVramOffset(offset)
            vramHex = toHex(currentVram, 8)[2:]

            if self.context is not None:
                auxLabel = self.context.getGenericLabel(currentVram) or self.context.getGenericSymbol(currentVram, tryPlusOffset=False)
                if auxLabel is not None:
                    label = "\nglabel " + auxLabel + "\n"

        if w in self.context.jumpTablesLabels:
            value = self.context.jumpTablesLabels[w]

        comment = ""
        if GlobalConfig.ASM_COMMENT:
            comment = f"/* {offsetHex} {vramHex} {rodataHex} */ "

        return f"{label}{comment} .word  {value}"


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

            for i in range(len(self.words)):
                f.write(self.getNthWord(i) + "\n")
