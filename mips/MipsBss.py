#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section
from .MipsContext import Context


class Bss(Section):
    def __init__(self, bssVramStart: int, bssVramEnd: int, filename: str, version: str, context: Context):
        super().__init__(bytearray(), filename, version, context)

        self.bssVramStart: int = bssVramStart
        self.bssVramEnd: int = bssVramEnd


    def analyze(self):
        if self.context is not None:
            self.context.symbols

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False
        # TODO ?
        # super().removePointers()
        return False

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".bss")

        if self.context is None:
            return

        with open(filepath + ".bss.s", "w") as f:
            f.write(".include \"macro.inc\"\n")
            f.write("\n")
            f.write("# assembler directives\n")
            f.write(".set noat      # allow manual use of $at\n")
            f.write(".set noreorder # don't insert nops after branches\n")
            f.write(".set gp=64     # allow use of 64-bit general purpose registers\n")
            f.write("\n")
            f.write(".section .bss\n")
            f.write("\n")
            f.write(".balign 16\n")

            offset = 0
            inFileOffset = self.offset
            sortedSymbols = sorted(self.context.symbols.items())
            i = 0
            while i < len(sortedSymbols):
                symbolVram, symbol = sortedSymbols[i]
                if symbolVram < self.bssVramStart:
                    i += 1
                    continue
                if symbolVram >= self.bssVramEnd:
                    break

                self.context.symbols[symbolVram].isDefined = True

                offsetHex = toHex(inFileOffset + self.commentOffset, 6)[2:]
                vramHex = toHex(symbolVram, 8)[2:]

                space = self.bssVramEnd - symbolVram
                if i + 1 < len(sortedSymbols):
                    space = sortedSymbols[i+1][0] - symbolVram

                label = f"\nglabel {symbol.name}\n"
                f.write(f"{label}/* {offsetHex} {vramHex} */  .space  {toHex(space, 2)}\n")
                offset += 4
                inFileOffset += 4
                i += 1
