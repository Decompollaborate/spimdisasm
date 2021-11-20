#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section


class Data(Section):
    def analyze(self):
        if self.vRamStart > -1:
            offset = 0
            for w in self.words:
                currentVram = self.getVramOffset(offset)

                contextSymbol = self.context.getSymbol(currentVram, tryPlusOffset=False)
                if contextSymbol is not None:
                    contextSymbol.isDefined = True

                offset += 4


    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
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

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".data")

        if self.size == 0:
            return

        with open(filepath + ".data.s", "w") as f:
            f.write(".include \"macro.inc\"\n")
            f.write("\n")
            f.write("# assembler directives\n")
            f.write(".set noat      # allow manual use of $at\n")
            f.write(".set noreorder # don't insert nops after branches\n")
            f.write(".set gp=64     # allow use of 64-bit general purpose registers\n")
            f.write("\n")
            f.write(".section .data\n")
            f.write("\n")
            f.write(".balign 16\n")

            offset = 0
            inFileOffset = self.offset
            i = 0
            while i < self.sizew:
                w = self.words[i]
                offsetHex = toHex(inFileOffset + self.commentOffset, 6)[2:]
                vramHex = ""
                label = ""
                if self.vRamStart > -1:
                    currentVram = self.getVramOffset(offset)
                    vramHex = toHex(currentVram, 8)[2:]

                    if self.context is not None:
                        auxLabel = self.context.getGenericLabel(currentVram) or self.context.getGenericSymbol(currentVram, tryPlusOffset=False)
                        if auxLabel is not None:
                            label = "\nglabel " + auxLabel + "\n"

                        contVariable = self.context.getSymbol(currentVram, False)
                        if contVariable is not None:
                            contVariable.isDefined = True

                dataHex = toHex(w, 8)[2:]
                value = toHex(w, 8)
                if self.context is not None:
                    symbol = self.context.getAnySymbol(w)
                    if symbol is not None:
                        value = symbol

                #comment = " "
                comment = ""
                if GlobalConfig.ASM_COMMENT:
                    #comment = f"/* {offsetHex} {vramHex} {dataHex} */"
                    comment = f"/* {offsetHex} {vramHex} */"

                line = f"{label}{comment} .word {value}"
                f.write(line + "\n")
                i += 1
                offset += 4
                inFileOffset += 4
