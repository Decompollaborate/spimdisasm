#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .MipsFile import File


class Rodata(File):
    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().removePointers()

        was_updated = False
        for i in range(self.sizew):
            top_byte = (self.words[i] >> 24) & 0xFF
            if top_byte == 0x80:
                self.words[i] = top_byte << 24
                was_updated = True
            if (top_byte & 0xF0) == 0x00 and (top_byte & 0x0F) != 0x00:
                self.words[i] = top_byte << 24
                was_updated = True

        if was_updated:
            self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".rodata")

        if self.size == 0:
            return

        with open(filepath + ".rodata.asm", "w") as f:
            # f.write(".section .rodata\n\n.balign 16\n\n")
            offset = 0
            for w in self.words:
                offsetHex = toHex(offset, 5)[2:]
                rodataHex = toHex(w, 8)[2:]
                line = toHex(w, 8)

                f.write(f"/* {offsetHex} {rodataHex} */  .word  {line}\n")
                offset += 4

