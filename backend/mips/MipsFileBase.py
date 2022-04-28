#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context

from .MipsElementBase import ElementBase
from .Symbols import SymbolBase


class FileBase(ElementBase):
    def __init__(self, context: Context, vram: int|None, filename: str, array_of_bytes: bytearray):
        super().__init__(context, 0, vram, filename, bytesToBEWords(array_of_bytes))
        self.bytes: bytearray = array_of_bytes # TODO: Necessary?

        self.symbolList: list[SymbolBase] = []

        self.pointersOffsets: set[int] = set()

        self.isHandwritten: bool = False
        self.isRsp: bool = False

    @property
    def size(self) -> int:
        return len(self.bytes)

    def setCommentOffset(self, commentOffset: int):
        self.commentOffset = commentOffset
        for sym in self.symbolList:
            sym.setCommentOffset(self.commentOffset)

    def getVramOffset(self, localOffset: int) -> int:
        if self.vram is None:
            return self.inFileOffset + localOffset
        # return self.vram + localOffset
        return self.vram + self.inFileOffset + localOffset

    def generateAsmLineComment(self, localOffset: int, wordValue: int|None = None) -> str:
        if not GlobalConfig.ASM_COMMENT:
            return ""
        offsetHex = f"{localOffset + self.inFileOffset + self.commentOffset:06X}"

        vramHex = ""
        if self.vram is not None:
            currentVram = self.getVramOffset(localOffset)
            vramHex = f"{currentVram:08X} "

        wordValueHex = ""
        if wordValue is not None:
            wordValueHex = f"{wordValue:08X} "

        return f"/* {offsetHex} {vramHex}{wordValueHex}*/"

    def getAsmPrelude(self) -> str:
        output = ""

        output += ".include \"macro.inc\"\n"
        output += "\n"
        output += "# assembler directives\n"
        output += ".set noat      # allow manual use of $at\n"
        output += ".set noreorder # don't insert nops after branches\n"
        output += ".set gp=64     # allow use of 64-bit general purpose registers\n"
        output += "\n"
        output += f".section {self.sectionType.toSectionName()}\n"
        output += "\n"
        output += ".balign 16\n"

        return output

    def getHash(self) -> str:
        return getStrHash(self.bytes)

    def printAnalyzisResults(self):
        pass

    def compareToFile(self, other_file: FileBase) -> dict:
        hash_one = self.getHash()
        hash_two = other_file.getHash()

        result = {
            "equal": hash_one == hash_two,
            "hash_one": hash_one,
            "hash_two": hash_two,
            "size_one": self.size,
            "size_two": other_file.size,
            "diff_bytes": 0,
            "diff_words": 0,
        }

        diff_bytes = 0
        diff_words = 0

        if not result["equal"]:
            min_len = min(self.size, other_file.size)
            for i in range(min_len):
                if self.bytes[i] != other_file.bytes[i]:
                    diff_bytes += 1

            min_len = min(self.sizew, other_file.sizew)
            for i in range(min_len):
                if self.words[i] != other_file.words[i]:
                    diff_words += 1

        result["diff_bytes"] = diff_bytes
        result["diff_words"] = diff_words

        return result

    def blankOutDifferences(self, other: FileBase) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        return False

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        return False

    def updateBytes(self):
        beWordsToBytes(self.words, self.bytes)
        # Truncate extra data
        self.bytes = self.bytes[:self.sizew*4]


    def disassemble(self) -> str:
        output = ""
        for sym in self.symbolList:
            output += sym.disassemble()
        return output

    def disassembleToFile(self, f: TextIO):
        f.write(self.getAsmPrelude())
        f.write(self.disassemble())


    def saveToFile(self, filepath: str):
        if len(self.symbolList) == 0:
            return

        if filepath == "-":
            self.disassembleToFile(sys.stdout)
        else:
            if GlobalConfig.WRITE_BINARY:
                if self.size > 0:
                    writeBytearrayToFile(filepath + self.sectionType.toStr(), self.bytes)
            with open(filepath + self.sectionType.toStr() + ".s", "w") as f:
                self.disassembleToFile(f)


def createEmptyFile() -> FileBase:
    return FileBase(Context(), None, "", bytearray())
