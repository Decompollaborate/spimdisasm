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

# Not intended to be instanced
class FileGeneric(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str):
        super().__init__(array_of_bytes, filename, version)

        self.text = Text(bytearray(0), filename, version)
        self.text.parent = self

        self.data = Data(bytearray(0), filename, version)
        self.data.parent = self

        self.rodata = Rodata(bytearray(0), filename, version)
        self.rodata.parent = self

        self.bss = Bss(bytearray(0), filename, version)
        self.rodata.parent = self

    @property
    def nFuncs(self) -> int:
        return self.text.nFuncs

    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)

        if isinstance(other_file, FileGeneric):
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

        if not isinstance(other_file, FileGeneric):
            return

        self.text.blankOutDifferences(other_file.text)
        self.data.blankOutDifferences(other_file.data)
        self.rodata.blankOutDifferences(other_file.rodata)
        self.bss.blankOutDifferences(other_file.bss)

        self.updateBytes()
        other_file.updateBytes()

    def removePointers(self):
        if not GlobalConfig.REMOVE_POINTERS:
            return

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
