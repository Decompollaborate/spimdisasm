#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFile import File


class Rodata(File):
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

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".rodata")

        if self.size == 0:
            return

        with open(filepath + ".rodata.asm", "w") as f:
            f.write(".section .rodata\n\n")
            offset = 0
            for w in self.words:
                offsetHex = toHex(offset, 5)[2:]
                vramHex = ""
                label = ""
                if self.vRamStart != -1:
                    currentVram = self.getVramOffset(offset)
                    vramHex = toHex(currentVram, 8)[2:]

                    if self.context is not None:
                        auxLabel = self.context.getGenericLabel(currentVram) or self.context.getGenericSymbol(currentVram)
                        if auxLabel is not None:
                            label = "glabel " + auxLabel + "\n"
                rodataHex = toHex(w, 8)[2:]
                value = toHex(w, 8)

                comment = ""
                if GlobalConfig.ASM_COMMENT:
                    comment = f" /* {offsetHex} {vramHex} {rodataHex} */ "

                line = f"{label}{comment} .word  {value}"
                f.write(line + "\n")
                offset += 4
