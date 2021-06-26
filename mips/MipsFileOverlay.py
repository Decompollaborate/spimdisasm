#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig

from .MipsFile import File
from .MipsText import Text
from .MipsData import Data
from .MipsRodata import Rodata
from .MipsBss import Bss
from .MipsReloc import Reloc
from .MipsFileGeneric import FileGeneric
from .MipsContext import Context

from .Instructions import wordToInstruction
from .ZeldaTables import OverlayTableEntry


class FileOverlay(FileGeneric):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, context: Context, tableEntry: OverlayTableEntry=None):
        super().__init__(array_of_bytes, filename, version, context)

        if tableEntry is not None:
            self.vRamStart = tableEntry.vramStart
            self.initVarsAddress = tableEntry.initVars

        seekup = self.words[-1]
        self.headerBPos = self.size - seekup
        self.headerWPos = self.headerBPos//4

        text_size = self.words[self.headerWPos]
        data_size = self.words[self.headerWPos+1]
        rodata_size = self.words[self.headerWPos+2]
        bss_size = self.words[self.headerWPos+3]
        reloc_size = 4*5 + 4*self.words[self.headerWPos+4]

        start = 0
        end = text_size
        self.text = Text(self.bytes[start:end], filename, version, context)
        self.text.parent = self
        self.text.offset = start
        self.text.vRamStart = self.vRamStart
        self.text.initVarsAddress = self.initVarsAddress

        start += text_size
        end += data_size
        self.data = Data(self.bytes[start:end], filename, version, context)
        self.data.parent = self
        self.data.offset = start
        self.data.vRamStart = self.vRamStart
        self.data.initVarsAddress = self.initVarsAddress

        start += data_size
        end += rodata_size
        self.rodata = Rodata(self.bytes[start:end], filename, version, context)
        self.rodata.parent = self
        self.rodata.offset = start
        self.rodata.vRamStart = self.vRamStart
        self.rodata.initVarsAddress = self.initVarsAddress

        #start += rodata_size
        #end += bss_size
        #self.bss = Bss(self.bytes[start:end], filename, version)
        # TODO
        self.bss = Bss(self.bytes[0:0], filename, version, context)
        self.bss.parent = self
        self.bss.offset = start
        self.bss.vRamStart = self.vRamStart
        self.bss.initVarsAddress = self.initVarsAddress

        start += rodata_size
        self.reloc = Reloc(self.bytes[start:], filename, version, context)
        self.reloc.parent = self
        self.reloc.offset = start
        self.reloc.vRamStart = self.vRamStart
        self.reloc.initVarsAddress = self.initVarsAddress


    def analyze(self):
        for entry in self.reloc.entries:
            section = entry.getSectionName()
            # type_name = entry.getTypeName()
            offset = entry.offset
            if entry.reloc == 0:
                continue
            if section == ".text":
                self.text.pointersOffsets.append(offset)
            elif section == ".data":
                self.data.pointersOffsets.append(offset)
            elif section == ".rodata":
                self.rodata.pointersOffsets.append(offset)
            elif section == ".bss":
                self.bss.pointersOffsets.append(offset)

        self.text.removeTrailingNops()

        self.text.analyze()
        self.data.analyze()
        self.rodata.analyze()
        self.bss.analyze()
        self.reloc.analyze()


    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)

        if isinstance(other_file, FileOverlay):
            result["filesections"]["reloc"] = self.reloc.compareToFile(other_file.reloc)

        return result

    def blankOutDifferences(self, other_file: File) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = super().blankOutDifferences(other_file)
        if isinstance(other_file, FileOverlay):
            was_updated = self.reloc.blankOutDifferences(other_file.reloc) or was_updated

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        for entry in self.reloc.entries:
            section = entry.getSectionName()
            type_name = entry.getTypeName()
            offset = entry.offset//4
            if entry.reloc == 0:
                continue
            if section == ".text":
                for func in self.text.functions[::-1]:
                    if entry.offset >= func.inFileOffset:
                        offset = (entry.offset- func.inFileOffset)//4
                        instr = func.instructions[offset]
                        if type_name == "R_MIPS_26":
                            func.instructions[offset] = wordToInstruction(instr.instr & 0xFC000000)
                        elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                            func.instructions[offset] = wordToInstruction(instr.instr & 0xFFFF0000)
                        else:
                            raise RuntimeError(f"Invalid <{type_name}> in .text of file '{self.version}/{self.filename}'. Reloc: {entry}")
                        break
            elif section == ".data":
                word = self.data.words[offset]
                if type_name == "R_MIPS_32":
                    self.data.words[offset] = word & 0xFF000000
                elif type_name == "R_MIPS_26":
                    self.data.words[offset] = word & 0xFC000000
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.data.words[offset] = word & 0xFFFF0000
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .data of file '{self.version}/{self.filename}'. Reloc: {entry}")
            elif section == ".rodata":
                word = self.rodata.words[offset]
                if type_name == "R_MIPS_32":
                    self.rodata.words[offset] = word & 0xFF000000
                elif type_name == "R_MIPS_26":
                    self.rodata.words[offset] = word & 0xFC000000
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.rodata.words[offset] = word & 0xFFFF0000
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .rodata of file '{self.version}/{self.filename}'. Reloc: {entry}")
            elif section == ".bss":
                word = self.bss.words[offset]
                if type_name == "R_MIPS_32":
                    self.bss.words[offset] = word & 0xFF000000
                elif type_name == "R_MIPS_26":
                    self.bss.words[offset] = word & 0xFC000000
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.bss.words[offset] = word & 0xFFFF0000
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .bss of file '{self.version}/{self.filename}'. Reloc: {entry}")
            else:
                pass
                #raise RuntimeError(f"Invalid reloc section <{section}> in file '{self.version}/{self.filename}'. Reloc: {entry}")

        was_updated = self.reloc.nRelocs >= 0
        was_updated = super().removePointers() or was_updated
        was_updated = self.reloc.removePointers() or was_updated

        return was_updated

    def updateBytes(self):
        self.text.updateBytes()
        self.data.updateBytes()
        self.rodata.updateBytes()
        self.bss.updateBytes()
        self.reloc.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath)
        self.reloc.saveToFile(filepath)
