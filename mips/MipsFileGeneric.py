#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase, createEmptyFile
from .MipsText import Text
from .MipsData import Data
from .MipsRodata import Rodata
from .MipsBss import Bss
from .MipsContext import Context

# Not intended to be instanced
class FileGeneric(FileBase):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, context: Context):
        super().__init__(array_of_bytes, filename, version, context)

        self.textList: Dict[str, Text] = dict()
        self.dataList: Dict[str, Data] = dict()
        self.rodataList: Dict[str, Rodata] = dict()
        self.bssList: Dict[str, Bss] = dict()

        self.initVarsAddress = -1

    @property
    def nFuncs(self) -> int:
        nFuncs = 0
        for text in self.textList.values():
            nFuncs += text.nFuncs
        return nFuncs

    def setVRamStart(self, vRamStart: int):
        super().setVRamStart(vRamStart)
        for text in self.textList.values():
            text.setVRamStart(vRamStart)
        for data in self.dataList.values():
            data.setVRamStart(vRamStart)
        for rodata in self.rodataList.values():
            rodata.setVRamStart(vRamStart)
        for bss in self.bssList.values():
            bss.setVRamStart(vRamStart)

    def getHash(self) -> str:
        bytes = bytearray(0)
        for section in self.textList.values():
            bytes += section.bytes
        for section in self.dataList.values():
            bytes += section.bytes
        for section in self.rodataList.values():
            bytes += section.bytes
        for section in self.bssList.values():
            bytes += section.bytes
        return getStrHash(bytes)

    def analyze(self):
        for section in self.textList.values():
            section.analyze()
        for section in self.dataList.values():
            section.analyze()
        for section in self.rodataList.values():
            section.analyze()
        for section in self.bssList.values():
            section.analyze()

    def compareToFile(self, other_file: FileBase):
        if isinstance(other_file, FileGeneric):
            filesections = {
                "text": dict(),
                "data": dict(),
                "rodata": dict(),
                # "bss": dict(),
            }

            for section_name, section in self.textList.items():
                if section_name in other_file.textList:
                    other_section = other_file.textList[section_name]
                    filesections["text"][section_name] = section.compareToFile(other_section)
                else:
                    filesections["text"][section_name] = section.compareToFile(createEmptyFile())
            for section_name, other_section in other_file.textList.items():
                if section_name in self.textList:
                    section = self.textList[section_name]
                    if section_name not in filesections["text"]:
                        filesections["text"][section_name] = section.compareToFile(other_section)
                else:
                    filesections["text"][section_name] = createEmptyFile().compareToFile(other_section)

            for section_name, section in self.dataList.items():
                if section_name in other_file.dataList:
                    other_section = other_file.dataList[section_name]
                    filesections["data"][section_name] = section.compareToFile(other_section)
                else:
                    filesections["data"][section_name] = section.compareToFile(createEmptyFile())
            for section_name, other_section in other_file.dataList.items():
                if section_name in self.dataList:
                    section = self.dataList[section_name]
                    if section_name not in filesections["data"]:
                        filesections["data"][section_name] = section.compareToFile(other_section)
                else:
                    filesections["data"][section_name] = createEmptyFile().compareToFile(other_section)

            for section_name, section in self.rodataList.items():
                if section_name in other_file.rodataList:
                    other_section = other_file.rodataList[section_name]
                    filesections["rodata"][section_name] = section.compareToFile(other_section)
                else:
                    filesections["rodata"][section_name] = section.compareToFile(createEmptyFile())
            for section_name, other_section in other_file.rodataList.items():
                if section_name in self.rodataList:
                    section = self.rodataList[section_name]
                    if section_name not in filesections["rodata"]:
                        filesections["rodata"][section_name] = section.compareToFile(other_section)
                else:
                    filesections["rodata"][section_name] = createEmptyFile().compareToFile(other_section)

            return {"filesections": filesections}

        return super().compareToFile(other_file)

    def blankOutDifferences(self, other_file: FileBase) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        if not isinstance(other_file, FileGeneric):
            return False

        was_updated = False
        for section_name, section in self.textList.items():
            if section_name in other_file.textList:
                other_section = other_file.textList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated
        for section_name, other_section in other_file.textList.items():
            if section_name in self.textList:
                section = self.textList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated

        for section_name, section in self.dataList.items():
            if section_name in other_file.dataList:
                other_section = other_file.dataList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated
        for section_name, other_section in other_file.dataList.items():
            if section_name in self.dataList:
                section = self.dataList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated

        for section_name, section in self.rodataList.items():
            if section_name in other_file.rodataList:
                other_section = other_file.rodataList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated
        for section_name, other_section in other_file.rodataList.items():
            if section_name in self.rodataList:
                section = self.rodataList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated

        for section_name, section in self.bssList.items():
            if section_name in other_file.bssList:
                other_section = other_file.bssList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated
        for section_name, other_section in other_file.bssList.items():
            if section_name in self.bssList:
                section = self.bssList[section_name]
                was_updated = section.blankOutDifferences(other_section) or was_updated

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        for section in self.textList.values():
            was_updated = section.removePointers() or was_updated
        for section in self.dataList.values():
            was_updated = section.removePointers() or was_updated
        for section in self.rodataList.values():
            was_updated = section.removePointers() or was_updated
        for section in self.bssList.values():
            was_updated = section.removePointers() or was_updated

        return was_updated

    def updateBytes(self):
        for section in self.textList.values():
            section.updateBytes()
        for section in self.dataList.values():
            section.updateBytes()
        for section in self.rodataList.values():
            section.updateBytes()
        for section in self.bssList.values():
            section.updateBytes()

    def saveToFile(self, filepath: str):
        for name, section in self.textList.items():
            if name != "":
                name = " " + name
            section.saveToFile(filepath + name)
        for name, section in self.dataList.items():
            if name != "":
                name = " " + name
            section.saveToFile(filepath + name)
        for name, section in self.rodataList.items():
            if name != "":
                name = " " + name
            section.saveToFile(filepath + name)
        for name, section in self.bssList.items():
            if name != "":
                name = " " + name
            section.saveToFile(filepath + name)
