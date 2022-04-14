#!/usr/bin/env python3

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context
from ..common.FileSectionType import FileSectionType

from .MipsSection import Section


class Bss(Section):
    def __init__(self, bssVramStart: int, bssVramEnd: int, filename: str, context: Context):
        super().__init__(bytearray(), filename, context)

        self.bssVramStart: int = bssVramStart
        self.bssVramEnd: int = bssVramEnd

        self.bssTotalSize: int = bssVramEnd - bssVramStart

        self.vRamStart = bssVramStart


    def setVRamStart(self, vRamStart: int):
        super().setVRamStart(vRamStart)

        self.bssVramStart = vRamStart
        self.bssVramEnd = vRamStart + self.bssTotalSize

    def analyze(self):
        # Check if the very start of the file has a bss variable and create it if it doesn't exist yet
        if self.context.getSymbol(self.bssVramStart, False) is None:
            contextSym = self.context.addSymbol(self.bssVramStart, f"B_{self.bssVramStart:08X}")
            contextSym.isDefined = True
            contextSym.sectionType = FileSectionType.Bss
            contextSym.isAutogenerated = True

        # If something that could be a pointer found in data happens to be in the middle of this bss file's addresses space
        # Then consider it a new bss variable
        for ptr in sorted(self.context.newPointersInData):
            if ptr < self.bssVramStart:
                continue
            if ptr >= self.bssVramEnd:
                break

            contextSym = self.context.getGenericSymbol(ptr)
            if contextSym is None:
                contextSym = self.context.addSymbol(ptr, f"B_{ptr:08X}")
                contextSym.isAutogenerated = True
                contextSym.isDefined = True
                contextSym.sectionType = FileSectionType.Bss
            else:
                if contextSym.isAutogenerated:
                    contextSym.name = f"B_{ptr:08X}"

        # Mark every known symbol that happens to be in this address space as defined
        for vram in self.context.symbols.irange(minimum=self.bssVramStart, maximum=self.bssVramEnd, inclusive=(True, False)):
            contextSym = self.context.symbols[vram]
            contextSym.isDefined = True
            contextSym.sectionType = FileSectionType.Bss
            if contextSym.isAutogenerated:
                contextSym.name = f"B_{vram:08X}"


    def disassembleToFile(self, f: TextIO):
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

        offsetSymbolsInSection = self.context.offsetSymbols[FileSectionType.Bss]
        bssSymbolOffsets = {offset: sym for offset, sym in offsetSymbolsInSection.items()}

        # Needs to move this to a list because the algorithm requires to check the size of a bss variable based on the next bss variable' vram
        if self.bssVramStart > 0:
            for symbolVram in self.context.symbols.irange(minimum=self.bssVramStart, maximum=self.bssVramEnd, inclusive=(True, False)):
                bssSymbolOffsets[symbolVram-self.bssVramStart] = self.context.symbols[symbolVram]

        sortedOffsets = sorted(bssSymbolOffsets.items())

        i = 0
        while i < len(sortedOffsets):
            symbolOffset, contextSym = sortedOffsets[i]
            symbolVram = self.bssVramStart + symbolOffset

            if symbolVram in self.context.symbols:
                self.context.symbols[symbolVram].isDefined = True

            inFileOffset = self.offset + symbolOffset

            offsetHex = f"{inFileOffset + self.commentOffset:06X}"
            vramHex = f"{symbolVram:08X}"

            # Calculate the space of the bss variable
            space = self.bssTotalSize - symbolOffset
            if i + 1 < len(sortedOffsets):
                nextSymbolOffset, _ = sortedOffsets[i+1]
                if nextSymbolOffset <= self.bssTotalSize:
                    space = nextSymbolOffset - symbolOffset

            # label = f"\nglabel {symbolName}\n"
            # if symbolName.startswith("."):
            #     label = f"\n/* static variable */\n{symbolName}\n"
            label = ""
            if contextSym.isStatic:
                label = "\n/* static variable */"
            label += f"\nglabel {contextSym.name}\n"

            comment = ""
            if GlobalConfig.ASM_COMMENT:
                comment = f"/* {offsetHex} {vramHex} */"

            line = f"{label}{comment}  .space  0x{space:02X}"
            f.write(line + "\n")
            i += 1

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".bss")

        if filepath == "-":
            self.disassembleToFile(sys.stdout)
        else:
            with open(filepath + ".bss.s", "w") as f:
                self.disassembleToFile(f)
