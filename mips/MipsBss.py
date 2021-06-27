#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section


class Bss(Section):
    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False
        # TODO ?
        # super().removePointers()
        return False

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".bss")

        if self.size == 0:
            return

        with open(filepath + ".bss.asm", "w") as f:
            # f.write(".section .bss\n\n.balign 16\n\n")
            offset = 0
            for w in self.words:
                offsetHex = toHex(offset, 5)[2:]
                bssHex = toHex(w, 8)[2:]
                line = toHex(w, 8)

                f.write(f"/* {offsetHex} {bssHex} */  .word  {line}\n")
                offset += 4
