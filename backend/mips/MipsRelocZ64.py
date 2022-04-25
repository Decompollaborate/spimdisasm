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
    def __init__(self, entry: int):
        self.sectionId = entry >> 30
        self.relocType = (entry >> 24) & 0x3F
        self.offset = entry & 0x00FFFFFF

    @property
    def reloc(self):
        return (self.sectionId << 30) | (self.relocType << 24) | (self.offset)

    def getSectionType(self) -> FileSectionType:
        return FileSectionType.fromId(self.sectionId)

    def getRelocType(self) -> RelocTypes:
        return RelocTypes.fromValue(self.relocType)

    def __str__(self) -> str:
        section = self.getSectionType().toStr()
        reloc = self.getRelocType().name
        return f"{section} {reloc} 0x{self.offset:X}"
    def __repr__(self) -> str:
        return self.__str__()


class RelocZ64(Section):
    def __init__(self, context: Context, vram: int|None, filename: str, array_of_bytes: bytearray):
        super().__init__(context, vram, filename, array_of_bytes)

        self.sectionType = FileSectionType.Reloc

        self.seekup = self.words[-1]

        self.setCommentOffset(self.size - self.seekup)

        # Remove non reloc stuff
        self.bytes = self.bytes[-self.seekup:]
        self.words = self.words[-self.seekup // 4:]

        self.sectionSizes = {
            FileSectionType.Text: self.words[0],
            FileSectionType.Data: self.words[1],
            FileSectionType.Rodata: self.words[2],
            FileSectionType.Bss: self.words[3],
        }
        self.relocCount = self.words[4]

        self.tail = self.words[self.relocCount+5:-1]

        self.entries: List[RelocEntry] = list()
        for word in self.words[5:self.relocCount+5]:
            self.entries.append(RelocEntry(word))

        self.differentSegment: bool = False

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
        if self.vram is not None:
            currentVram = self.getVramOffset(0)
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.name}_OverlayInfo")

            currentVram += 0x14
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.name}_OverlayRelocations")

            currentVram = self.getVramOffset(self.size - 4)
            if self.context.getSymbol(currentVram, False) is None:
                self.context.addSymbol(currentVram, f"{self.name}_OverlayInfoOffset")


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

        result += self.getSymbolLabelAtVram(self.getVramOffset(offset), f"\nglabel {self.name}_OverlayInfo\n")

        for fileSect in FileSections_ListBasic:
            sectName = fileSect.toCapitalizedStr()

            comment = self.generateAsmLineComment(offset, self.sectionSizes[fileSect])
            result += f"{comment} .word _{self.name}Segment{sectName}Size\n"

            offset += 4

        result += f"\n"

        comment = self.generateAsmLineComment(offset, self.relocCount)
        result += f"{comment} .word {self.relocCount} # reloc_count\n"

        offset += 4

        result += self.getSymbolLabelAtVram(self.getVramOffset(offset), f"\nglabel {self.name}_OverlayRelocations\n")

        for r in self.entries:
            relocHex = f"{r.reloc:08X}"

            comment = self.generateAsmLineComment(offset, r.reloc)
            result += f"{comment} .word 0x{relocHex} # {str(r)}\n"

            offset += 4

        result += "\n"
        for pad in self.tail:
            padcHex = f"{pad:08X}"

            comment = self.generateAsmLineComment(offset, pad)
            result += f"{comment} .word 0x{padcHex}\n"

            offset += 4

        result += self.getSymbolLabelAtVram(self.getVramOffset(offset), f"\nglabel {self.name}_OverlayInfoOffset\n")

        comment = self.generateAsmLineComment(offset, self.seekup)
        result += f"{comment} .word 0x{self.seekup:02X}\n"

        return result


    def saveToFile(self, filepath: str):
        #super().saveToFile(filepath + ".reloc")

        if self.size == 0:
            return

        if filepath == "-":
            self.disassembleToFile(sys.stdout)
        else:
            with open(filepath + ".reloc.s", "w") as f:
                self.disassembleToFile(f)
