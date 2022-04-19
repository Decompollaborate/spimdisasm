#!/usr/bin/env python3

# Relocation format used by overlays of Zelda64, Yoshi Story and Doubutsu no Mori (Animal Forest)

from __future__ import annotations

from typing import List

from ..common.Utils import *

from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context
from ..common.FileSectionType import FileSectionType, FileSections_ListBasic

from .MipsFileBase import FileBase
from .MipsSection import Section
from .MipsRelocTypes import RelocTypes


class RelocEntry:
    sectionNames = {
        #0: ".bss",
        1: ".text",
        2: ".data",
        3: ".rodata",
        4: ".bss", # ?
    }
    relocationsNames = {
        2: "R_MIPS_32",
        4: "R_MIPS_26",
        5: "R_MIPS_HI16",
        6: "R_MIPS_LO16",
    }

    def __init__(self, entry: int):
        self.sectionId = entry >> 30
        self.relocType = (entry >> 24) & 0x3F
        self.offset = entry & 0x00FFFFFF

    @property
    def reloc(self):
        return (self.sectionId << 30) | (self.relocType << 24) | (self.offset)

    def getSectionName(self) -> str:
        return RelocEntry.sectionNames.get(self.sectionId, str(self.sectionId))

    def getTypeName(self) -> str:
        return RelocEntry.relocationsNames.get(self.relocType, str(self.relocType))

    def __str__(self) -> str:
        section = self.getSectionName()
        reloc = self.getTypeName()
        return f"{section} {reloc} 0x{self.offset:X}"
    def __repr__(self) -> str:
        return self.__str__()


class RelocZ64(Section):
    def __init__(self, array_of_bytes: bytearray, filename: str, context: Context):
        super().__init__(array_of_bytes, filename, context)

        self.sectionSizes = {
            FileSectionType.Text: self.words[0],
            FileSectionType.Data: self.words[1],
            FileSectionType.Rodata: self.words[2],
            FileSectionType.Bss: self.words[3],
        }
        self.relocCount = self.words[4]

        self.tail = self.words[self.relocCount+5:-1]

        self.seekup = self.words[-1]

        self.entries: List[RelocEntry] = list()
        for word in self.words[5:self.relocCount+5]:
            self.entries.append(RelocEntry(word))

    @property
    def nRelocs(self) -> int:
        return len(self.entries)

    @property
    def textSize(self) -> int:
        return self.sectionSizes[FileSectionType.Text]
    @property
    def dataSize(self) -> int:
        return self.sectionSizes[FileSectionType.Data]
    @property
    def rodataSize(self) -> int:
        return self.sectionSizes[FileSectionType.Rodata]
    @property
    def bssSize(self) -> int:
        return self.sectionSizes[FileSectionType.Bss]


    def analyze(self):
        if self.vRamStart > -1:
            currentVram = self.getVramOffset(0)
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.filename}_OverlayInfo")

            currentVram += 0x14
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.filename}_OverlayRelocations")

            currentVram = self.getVramOffset(self.size - 4)
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.filename}_OverlayInfoOffset")


    def compareToFile(self, other_file: FileBase):
        result = super().compareToFile(other_file)
        # TODO
        return result

    def blankOutDifferences(self, other_file: FileBase) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        # TODO ?
        # super().blankOutDifferences(File)
        return False

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        # TODO ?
        # super().removePointers()
        return False


    def disassemble(self) -> str:
        result = ""

        offset = 0
        inFileOffset = self.offset
        currentVram = self.getVramOffset(offset)

        result += self.getSymbolLabelAtVram(currentVram, f"\nglabel {self.filename}_OverlayInfo\n")

        for fileSect in FileSections_ListBasic:
            offsetHex = f"{inFileOffset + self.commentOffset:06X}"

            vramHex = ""
            if self.vRamStart > -1:
                vramHex = f"{currentVram:08X}"

            sectName = fileSect.toCapitalizedStr()
            value = f"_{self.filename}Segment{sectName}Size"

            comment = ""
            if GlobalConfig.ASM_COMMENT:
                dataHex = f"{self.sectionSizes[fileSect]:08X}"
                comment = f"/* {offsetHex} {vramHex} {dataHex} */"

            line = f"{comment} .word {value}\n"
            result += line

            offset += 4
            inFileOffset += 4
            currentVram += 4

        result += f"\n"

        offsetHex = f"{inFileOffset + self.commentOffset:06X}"

        vramHex = ""
        if self.vRamStart > -1:
            vramHex = f"{currentVram:08X}"

        value = f"{self.relocCount} # reloc_count"

        comment = ""
        if GlobalConfig.ASM_COMMENT:
            dataHex = f"{self.relocCount:08X}"
            comment = f"/* {offsetHex} {vramHex} {dataHex} */"

        line = f"{comment} .word {value}\n"
        result += line

        offset += 4
        inFileOffset += 4
        currentVram += 4


        result += self.getSymbolLabelAtVram(currentVram, f"\nglabel {self.filename}_OverlayRelocations\n")

        for r in self.entries:
            offsetHex = f"{inFileOffset + self.commentOffset:06X}"
            vramHex = ""
            if self.vRamStart > -1:
                vramHex = f"{currentVram:08X}"

            relocHex = f"{r.reloc:08X}"

            comment = ""
            if GlobalConfig.ASM_COMMENT:
                comment = f"/* {offsetHex} {vramHex} {relocHex} */"

            line = f"{comment} .word 0x{relocHex} # {str(r)}\n"
            result += line

            offset += 4
            inFileOffset += 4
            currentVram += 4

        result += "\n"
        for pad in self.tail:
            offsetHex = f"{inFileOffset + self.commentOffset:06X}"
            vramHex = ""
            if self.vRamStart > -1:
                vramHex = f"{currentVram:08X}"

            padcHex = f"{pad:08X}"

            comment = ""
            if GlobalConfig.ASM_COMMENT:
                comment = f"/* {offsetHex} {vramHex} {padcHex} */"

            line = f"{comment} .word 0x{padcHex}\n"
            result += line

            offset += 4
            inFileOffset += 4
            currentVram += 4

        result += self.getSymbolLabelAtVram(currentVram, f"\nglabel {self.filename}_OverlayInfoOffset\n")

        offsetHex = f"{inFileOffset + self.commentOffset:06X}"

        vramHex = ""
        if self.vRamStart > -1:
            vramHex = f"{currentVram:08X}"

        value = f"0x{self.seekup:02X}"

        comment = ""
        if GlobalConfig.ASM_COMMENT:
            dataHex = f"{self.seekup:08X}"
            comment = f"/* {offsetHex} {vramHex} {dataHex} */"

        line = f"{comment} .word {value}\n"
        result += line

        return result

    def disassembleToFile(self, f: TextIO):
        f.write(".include \"macro.inc\"\n")
        f.write("\n")
        f.write("# assembler directives\n")
        f.write(".set noat      # allow manual use of $at\n")
        f.write(".set noreorder # don't insert nops after branches\n")
        f.write(".set gp=64     # allow use of 64-bit general purpose registers\n")
        f.write("\n")
        f.write(".section .ovl\n")
        f.write("\n")
        f.write(".balign 16\n")

        f.write(self.disassemble())

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".reloc")

        if self.size == 0:
            return

        if filepath == "-":
            self.disassembleToFile(sys.stdout)
        else:
            with open(filepath + ".reloc.s", "w") as f:
                self.disassembleToFile(f)
