#!/usr/bin/python3

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context

class FileBase:
    def __init__(self, array_of_bytes: bytearray, filename: str, context: Context):
        self.bytes: bytearray = array_of_bytes
        self.words: List[int] = bytesToBEWords(self.bytes)
        self.filename: str = filename
        self.context: Context = context

        self.parent: FileBase|None = None
        self.offset: int = 0 # in-file offset
        self.commentOffset: int = 0
        self.vRamStart: int = -1

        self.pointersOffsets: dict[int, str|None] = dict()
        self.symbolNameOffsets: dict[int, str|None] = dict()

        self.isHandwritten: bool = False
        self.isRsp: bool = False
        self.newStuffSuffix = ""

    @property
    def size(self) -> int:
        return len(self.bytes)
    @property
    def sizew(self) -> int:
        return len(self.words)

    def setVRamStart(self, vRamStart: int):
        self.vRamStart = vRamStart

    def getVramOffset(self, localOffset: int) -> int:
        if self.vRamStart < 0:
            return self.offset + localOffset
        return self.vRamStart + self.offset + localOffset

    def getHash(self) -> str:
        return getStrHash(self.bytes)

    def analyze(self):
        pass

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

    def saveToFile(self, filepath: str):
        if self.size == 0 or not GlobalConfig.WRITE_BINARY:
            return
        if filepath == "-":
            return
        writeBytearrayToFile(filepath, self.bytes)

    def setCommentOffset(self, commentOffset: int):
        self.commentOffset = commentOffset

def createEmptyFile() -> FileBase:
    return FileBase(bytearray(0), "", Context())
