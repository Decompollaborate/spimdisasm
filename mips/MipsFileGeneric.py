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
        if isinstance(other_file, FileGeneric):
            return {"filesections": {
                    "text": self.text.compareToFile(other_file.text),
                    "data": self.data.compareToFile(other_file.data),
                    "rodata": self.rodata.compareToFile(other_file.rodata),
                    # "bss": self.bss.compareToFile(other_file.bss),
                }
            }

        return super().compareToFile(other_file)

    def blankOutDifferences(self, other_file: File) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        if not isinstance(other_file, FileGeneric):
            return False

        was_updated = False
        was_updated = self.text.blankOutDifferences(other_file.text) or was_updated
        was_updated = self.data.blankOutDifferences(other_file.data) or was_updated
        was_updated = self.rodata.blankOutDifferences(other_file.rodata) or was_updated
        was_updated = self.bss.blankOutDifferences(other_file.bss) or was_updated

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        was_updated = self.text.removePointers() or was_updated
        was_updated = self.data.removePointers() or was_updated
        was_updated = self.rodata.removePointers() or was_updated
        was_updated = self.bss.removePointers() or was_updated

        return was_updated

    def updateBytes(self):
        self.text.updateBytes()
        self.data.updateBytes()
        self.rodata.updateBytes()
        self.bss.updateBytes()

    def saveToFile(self, filepath: str):
        self.text.saveToFile(filepath)
        self.data.saveToFile(filepath)
        self.rodata.saveToFile(filepath)
        self.bss.saveToFile(filepath)
