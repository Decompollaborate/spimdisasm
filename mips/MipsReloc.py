#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFile import File


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
        return f"{section} {reloc} {hex(self.offset)}"
    def __repr__(self) -> str:
        return self.__str__()

class Reloc(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str):
        super().__init__(array_of_bytes, filename, version)

        self.entries: List[RelocEntry] = list()
        for word in self.words:
            self.entries.append(RelocEntry(word))

    @property
    def nRelocs(self):
        return len(self.entries)

    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)
        # TODO
        return result

    def removePointers(self):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        super().removePointers()
        self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".reloc")

        if self.size == 0:
            return

        with open(filepath + ".reloc.asm", "w") as f:
            # f.write(".section .rodata\n\n.balign 16\n\n")
            offset = 0
            for r in self.entries:
                offsetHex = toHex(offset, 5)[2:]
                relocHex = toHex(r.reloc, 8)[2:]
                line = str(r)

                f.write(f"/* {offsetHex} {relocHex} */  {line}\n")
                offset += 4
