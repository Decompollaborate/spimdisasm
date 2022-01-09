#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section
from .MipsContext import Context, ContextSymbol


class Bss(Section):
    def __init__(self, bssVramStart: int, bssVramEnd: int, filename: str, version: str, context: Context):
        super().__init__(bytearray(), filename, version, context)

        self.bssVramStart: int = bssVramStart
        self.bssVramEnd: int = bssVramEnd

    def analyze(self):
        # Check if the very start of the file has a bss variable and create it if it doesn't exists yet
        if self.context.getSymbol(self.bssVramStart, False) is None:
            contextSym = ContextSymbol(self.bssVramStart, "D_" + toHex(self.bssVramStart, 8)[2:])
            contextSym.isDefined = True
            contextSym.isBss = True
            if self.newStuffSuffix:
                contextSym.name += f"_{self.newStuffSuffix}"
            self.context.symbols[self.bssVramStart] = contextSym

        # If something that could be a pointer found in data happens to be in the middle of this bss file's addresses space
        # Then consider it a new bss variable
        for ptr in sorted(self.context.newPointersInData):
            if ptr < self.bssVramStart:
                continue
            if ptr >= self.bssVramEnd:
                break

            if self.context.getGenericSymbol(ptr) is None:
                contVar = ContextSymbol(ptr, f"D_{ptr:08X}")
                contVar.isDefined = True
                contVar.isBss = True

                self.context.symbols[ptr] = contVar

        # Mark every known symbol that happens to be in this address space as defined
        for vram in self.context.symbols.irange(minimum=self.bssVramStart, maximum=self.bssVramEnd, inclusive=(True, False)):
            self.context.symbols[vram].isDefined = True
            self.context.symbols[vram].isBss = True


    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".bss")

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
            # Needs to move this to a list because the algorithm requires to check the size of a bss variable based on the next bss variable' vram
            # TODO: sorted() may not be required here anymore because of SortedDict. Test if removing it doesn't break anything
            sortedSymbols = sorted(self.context.symbols.irange(minimum=self.bssVramStart, maximum=self.bssVramEnd, inclusive=(True, False)))
            i = 0
            while i < len(sortedSymbols):
                symbolVram = sortedSymbols[i]
                symbol = self.context.symbols[symbolVram]

                self.context.symbols[symbolVram].isDefined = True

                offsetHex = toHex(self.offset + (symbolVram - self.bssVramStart) + self.commentOffset, 6)[2:]
                vramHex = toHex(symbolVram, 8)[2:]

                # Calculate the space of the bss variable
                space = self.bssVramEnd - symbolVram
                if i + 1 < len(sortedSymbols):
                    if sortedSymbols[i+1] <= self.bssVramEnd:
                        space = sortedSymbols[i+1] - symbolVram

                label = f"\nglabel {symbol.name}\n"
                f.write(f"{label}/* {offsetHex} {vramHex} */  .space  {toHex(space, 2)}\n")
                offset += 4
                i += 1
