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
from .MipsInstructions import wordToInstruction
# TODO: remove?
from .ZeldaTables import OverlayTableEntry


class Overlay(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, tableEntry: OverlayTableEntry=None):
        super().__init__(array_of_bytes, filename, version, tableEntry=tableEntry)

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
        self.text = Text(self.bytes[start:end], filename, version, tableEntry=tableEntry)

        start += text_size
        end += data_size
        self.data = Data(self.bytes[start:end], filename, version, tableEntry=tableEntry)

        start += data_size
        end += rodata_size
        self.rodata = Rodata(self.bytes[start:end], filename, version, tableEntry=tableEntry)

        #start += rodata_size
        #end += bss_size
        #self.bss = Bss(self.bytes[start:end], filename, version, tableEntry=tableEntry)
        # TODO
        self.bss = Bss(self.bytes[0:0], filename, version, tableEntry=tableEntry)

        #start += bss_size
        start += rodata_size
        end += header_size
        self.header = bytesToBEWords(self.bytes[start:end])

        start += header_size
        end += reloc_size
        self.reloc = Reloc(self.bytes[start:end], filename, version, tableEntry=tableEntry)

        self.tail = bytesToBEWords(self.bytes[end:])

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

        if isinstance(other_file, Overlay):
            result["ovl"] = {
                "text": self.text.compareToFile(other_file.text),
                "data": self.data.compareToFile(other_file.data),
                "rodata": self.rodata.compareToFile(other_file.rodata),
                "bss": self.bss.compareToFile(other_file.bss),
                "reloc": self.reloc.compareToFile(other_file.reloc),
            }

        return result

    def blankOutDifferences(self, other_file: File):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        super().blankOutDifferences(other_file)
        if not isinstance(other_file, Overlay):
            return

        self.text.blankOutDifferences(other_file.text)

        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words + self.header + self.reloc.words + self.tail
        self.updateBytes()

        other_file.words = other_file.text.words + other_file.data.words  + other_file.rodata.words + other_file.bss.words + other_file.header + other_file.reloc.words + other_file.tail
        other_file.updateBytes()

    def removePointers(self):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        super().removePointers()

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


        self.text.removePointers()
        self.data.removePointers()
        self.rodata.removePointers()
        self.bss.removePointers()
        self.reloc.removePointers()

        self.updateBytes()

    def updateBytes(self):
        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words + self.header + self.reloc.words + self.tail
        super().updateBytes()

    def saveToFile(self, filepath: str):
        self.text.saveToFile(filepath)
        self.data.saveToFile(filepath)
        self.rodata.saveToFile(filepath)
        self.bss.saveToFile(filepath)
        self.reloc.saveToFile(filepath)
