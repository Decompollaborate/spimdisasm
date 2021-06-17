#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .MipsFile import File
from .MipsText import Text
from .MipsReloc import Reloc


class Overlay(File):
    def __init__(self, array_of_bytes):
        super().__init__(array_of_bytes)

        seekup = self.words[-1]
        self.headerBPos = self.size - seekup
        self.headerWPos = self.headerBPos//4

        text_size = self.words[self.headerWPos]
        data_size = self.words[self.headerWPos+1]
        rodata_size = self.words[self.headerWPos+2]
        bss_size = self.words[self.headerWPos+3]
        header_size = 4*5
        reloc_size = 4*self.words[self.headerWPos+4]

        start = 0
        end = text_size
        self.text = Text(self.bytes[start:end])

        start += text_size
        end += data_size
        self.data = File(self.bytes[start:end])

        start += data_size
        end += rodata_size
        self.rodata = File(self.bytes[start:end])

        start += rodata_size
        end += bss_size
        self.bss = File(self.bytes[start:end])

        start += bss_size
        end += header_size
        self.header = bytesToBEWords(self.bytes[start:end])

        start += header_size
        end += reloc_size
        self.reloc = Reloc(self.bytes[start:end])

        self.tail = bytesToBEWords(self.bytes[end:])


    def compareToFile(self, other_file: File, args):
        result = super().compareToFile(other_file, args)

        if isinstance(other_file, Overlay):
            result["ovl"] = {
                "text": self.text.compareToFile(other_file.text, args),
                "data": self.data.compareToFile(other_file.data, args),
                "rodata": self.rodata.compareToFile(other_file.rodata, args),
                "bss": self.bss.compareToFile(other_file.bss, args),
                "reloc": self.reloc.compareToFile(other_file.reloc, args),
            }

        return result

    def blankOutDifferences(self, other_file: File, args):
        super().blankOutDifferences(other_file, args)
        if not isinstance(other_file, Overlay):
            return

        self.text.blankOutDifferences(other_file.text, args)

        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words + self.header + self.reloc.words + self.tail
        self.updateBytes()
        
        other_file.words = other_file.text.words + other_file.data.words  + other_file.rodata.words + other_file.bss.words + other_file.header + other_file.reloc.words + other_file.tail
        other_file.updateBytes()

    def relocate(self, allocatedVRamAddress: int, vRamAddress: int):
        allocu32 = allocatedVRamAddress

        sections = [0, 0, 0, 0]
        sections[0] = 0
        sections[1] = allocu32
        sections[2] = allocu32 + self.text.size
        sections[3] = sections[2] + self.data.size

        for reloc in self.reloc.entries:
            relocDataPtr = reloc.sectionId
            # relocData = *relocDataPtr
            if reloc.relocType == 2:
                pass
            elif reloc.relocType == 4:
                pass
            elif reloc.relocType == 5:
                pass
            elif reloc.relocType == 6:
                pass
