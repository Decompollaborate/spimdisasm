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

        self.textSize = self.words[0]
        self.dataSize = self.words[1]
        self.rodataSize = self.words[2]
        self.bssSize = self.words[3]
        self.relocCount = self.words[4]

        self.tail = self.words[self.relocCount+5:-1]

        self.seekup = self.words[-1]

        self.entries: List[RelocEntry] = list()
        for word in self.words[5:self.relocCount+5]:
            self.entries.append(RelocEntry(word))

    @property
    def nRelocs(self):
        return len(self.entries)

    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)
        # TODO
        return result

    def blankOutDifferences(self, other_file: File):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        # TODO ?
        # super().blankOutDifferences(File)
        # self.updateBytes()
        return

    def removePointers(self):
        # TODO ?
        return
        #if not GlobalConfig.REMOVE_POINTERS:
        #    return
        #super().removePointers()
        #self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".reloc")

        if self.size == 0:
            return

        with open(filepath + ".reloc.asm", "w") as f:
            offset = 0
            currentVram = self.getVramOffset(offset)

            f.write(".section .ovl\n\n")

            f.write(f"glabel {self.filename}OverlayInfo\n")

            f.write(f"/* %05X %08X %08X */  .word  {self.textSize} # {self.filename}TextSize\n" % (offset + 0x0, currentVram + 0x0, self.textSize))
            f.write(f"/* %05X %08X %08X */  .word  {self.dataSize} # {self.filename}DataSize\n" % (offset + 0x4, currentVram + 0x4, self.dataSize))
            f.write(f"/* %05X %08X %08X */  .word  {self.rodataSize} # {self.filename}RodataSize\n" % (offset + 0x8, currentVram + 0x8, self.rodataSize))
            f.write(f"/* %05X %08X %08X */  .word  {self.bssSize} # {self.filename}BssSize\n" % (offset + 0xC, currentVram + 0xC, self.bssSize))
            f.write(f"/* %05X %08X %08X */  .word  {self.relocCount} # {self.filename}RelocCount\n" % (offset + 0x10, currentVram + 0x10, self.relocCount))
            f.write(f"\n")

            f.write(f"glabel {self.filename}OverlayRelocations\n")
            for r in self.entries:
                offsetHex = toHex(offset, 5)[2:]
                vramHex = ""
                if self.vRamStart != -1:
                    currentVram = self.getVramOffset(offset)
                    vramHex = toHex(currentVram, 8)[2:]
                relocHex = toHex(r.reloc, 8)[2:]
                line = str(r)

                f.write(f"/* {offsetHex} {vramHex} {relocHex} */  {line}\n")
                offset += 4

            f.write("\n")
            for pad in self.tail:
                offsetHex = toHex(offset, 5)[2:]
                vramHex = ""
                if self.vRamStart != -1:
                    currentVram = self.getVramOffset(offset)
                    vramHex = toHex(currentVram, 8)[2:]
                padcHex = toHex(pad, 8)

                f.write(f"/* {offsetHex} {vramHex} {padcHex[2:]} */  {padcHex}\n")
                offset += 4

            f.write(f"glabel {self.filename}OverlayInfoOffset\n")
            currentVram = self.getVramOffset(offset)
            f.write(f"/* %05X %08X %08X */  .word  {self.seekup}\n" % (offset + 0x0, currentVram + 0x0, self.seekup))
