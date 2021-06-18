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

from .MipsInstructions import wordToInstruction
from .ZeldaTables import OverlayTableEntry


class FileOverlay(FileGeneric):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, tableEntry: OverlayTableEntry=None):
        super().__init__(array_of_bytes, filename, version)

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
        self.text = Text(self.bytes[start:end], filename, version)
        self.text.parent = self
        self.text.offset = start
        self.text.vRamStart = self.vRamStart
        self.text.initVarsAddress = self.initVarsAddress

        start += text_size
        end += data_size
        self.data = Data(self.bytes[start:end], filename, version)
        self.data.parent = self
        self.data.offset = start
        self.data.vRamStart = self.vRamStart
        self.data.initVarsAddress = self.initVarsAddress

        start += data_size
        end += rodata_size
        self.rodata = Rodata(self.bytes[start:end], filename, version)
        self.rodata.parent = self
        self.rodata.offset = start
        self.rodata.vRamStart = self.vRamStart
        self.rodata.initVarsAddress = self.initVarsAddress

        #start += rodata_size
        #end += bss_size
        #self.bss = Bss(self.bytes[start:end], filename, version)
        # TODO
        self.bss = Bss(self.bytes[0:0], filename, version)
        self.bss.parent = self
        self.bss.offset = start
        self.bss.vRamStart = self.vRamStart
        self.bss.initVarsAddress = self.initVarsAddress

        start += rodata_size
        self.reloc = Reloc(self.bytes[start:], filename, version)
        self.reloc.parent = self
        self.reloc.offset = start
        self.reloc.vRamStart = self.vRamStart
        self.reloc.initVarsAddress = self.initVarsAddress

        """
        functions = set()
        lastHigh = 0
        for relocEntry in self.reloc.entries:
            if relocEntry.getSectionName() == ".text":
                relocType = relocEntry.getTypeName()
                offset = relocEntry.offset >> 2
                if relocType == "R_MIPS_26":
                    # print(self.text.instructions[offset])
                    functions.add(self.text.instructions[offset].instr_index<<2)
                elif relocType == "R_MIPS_HI16":
                    lastHigh = self.text.instructions[offset].immediate
                elif relocType == "R_MIPS_LO16":
                    low = self.text.instructions[offset].immediate
                    # print (toHex((lastHigh << 16) | low, 8)[2:])
        #print(len(functions))
        #for f in sorted(functions):
        #    print("func_80"+toHex(f, 6)[2:])
        """

        self.text.findFunctions()


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
                instr = self.text.instructions[offset]
                if type_name == "R_MIPS_26":
                    self.text.instructions[offset] = wordToInstruction(instr.instr & 0xFC000000)
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.text.instructions[offset] = wordToInstruction(instr.instr & 0xFFFF0000)
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .text of file '{self.version}/{self.filename}'. Reloc: {entry}")
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

        was_updated = super().removePointers()
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
