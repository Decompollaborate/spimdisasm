#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFile import File
from .MipsText import Text
from .MipsData import Data
from .MipsRodata import Rodata
from .MipsBss import Bss
from .ZeldaOffsets import codeVramStart, codeDataStart, codeRodataStart


class FileCode(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str):
        super().__init__(array_of_bytes, filename, version)

        self.vRamStart = codeVramStart[version]

        text_start = 0
        data_start = codeDataStart[version]
        rodata_start = codeRodataStart[version]
        # bss_start = codeBssStart[version]
        bss_start = self.size

        self.text = Text(self.bytes[text_start:data_start], filename, version)
        self.text.parent = self
        self.text.offset = text_start
        self.text.vRamStart = self.vRamStart

        self.data = Data(self.bytes[data_start:rodata_start], filename, version)
        self.data.parent = self
        self.data.offset = data_start
        self.data.vRamStart = self.vRamStart

        self.rodata = Rodata(self.bytes[rodata_start:bss_start], filename, version)
        self.rodata.parent = self
        self.rodata.offset = rodata_start
        self.rodata.vRamStart = self.vRamStart

        self.bss = Bss(self.bytes[bss_start:self.size], filename, version)
        self.bss.parent = self
        self.bss.offset = bss_start
        self.bss.vRamStart = self.vRamStart

        self.text.findFunctions()


    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)

        if isinstance(other_file, FileCode):
            result["filesections"] = {
                "text": self.text.compareToFile(other_file.text),
                "data": self.data.compareToFile(other_file.data),
                "rodata": self.rodata.compareToFile(other_file.rodata),
                "bss": self.bss.compareToFile(other_file.bss),
            }

        return result

    def blankOutDifferences(self, other_file: File):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        super().blankOutDifferences(other_file)
        if not isinstance(other_file, FileCode):
            return

        self.text.blankOutDifferences(other_file.text)

        self.updateBytes()
        other_file.updateBytes()


    def removePointers(self):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        super().removePointers()

        self.text.removePointers()
        self.data.removePointers()
        self.rodata.removePointers()
        self.bss.removePointers()

        self.updateBytes()

    def updateBytes(self):
        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words
        super().updateBytes()

    def saveToFile(self, filepath: str):
        self.text.saveToFile(filepath)
        self.data.saveToFile(filepath)
        self.rodata.saveToFile(filepath)
        self.bss.saveToFile(filepath)
