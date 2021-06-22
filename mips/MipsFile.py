#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig

class File:
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str):
        self.bytes: bytearray = array_of_bytes
        self.words: List[int] = bytesToBEWords(self.bytes)
        self.filename: str = filename
        self.version: str = version
        self.parent: File|None = None
        self.offset: int = 0 # in-file offset
        self.vRamStart: int = -1
        self.initVarsAddress: int = -1

    @property
    def size(self) -> int:
        return len(self.bytes)
    @property
    def sizew(self) -> int:
        return len(self.words)

    def getVramOffset(self, localOffset: int) -> int:
        if self.vRamStart < 0:
            return self.offset + localOffset
        return self.vRamStart + self.offset + localOffset

    def getHash(self) -> str:
        return getStrHash(self.bytes)

    def compareToFile(self, other_file: File) -> dict:
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

        if not result["equal"]:
            min_len = min(self.size, other_file.size)
            for i in range(min_len):
                if self.bytes[i] != other_file.bytes[i]:
                    result["diff_bytes"] += 1

            min_len = min(self.sizew, other_file.sizew)
            for i in range(min_len):
                if self.words[i] != other_file.words[i]:
                    result["diff_words"] += 1

        return result

    def blankOutDifferences(self, other: File) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        if GlobalConfig.IGNORE_80 or GlobalConfig.IGNORE_06 or GlobalConfig.IGNORE_04:
            min_len = min(self.sizew, other.sizew)
            for i in range(min_len):
                if GlobalConfig.IGNORE_80:
                    if ((self.words[i] >> 24) & 0xFF) == 0x80 and ((other.words[i] >> 24) & 0xFF) == 0x80:
                        self.words[i] = 0x80000000
                        other.words[i] = 0x80000000
                        was_updated = True
                if GlobalConfig.IGNORE_06:
                    if ((self.words[i] >> 24) & 0xFF) == 0x06 and ((other.words[i] >> 24) & 0xFF) == 0x06:
                        self.words[i] = 0x06000000
                        other.words[i] = 0x06000000
                        was_updated = True
                if GlobalConfig.IGNORE_04:
                    if ((self.words[i] >> 24) & 0xFF) == 0x04 and ((other.words[i] >> 24) & 0xFF) == 0x04:
                        self.words[i] = 0x04000000
                        other.words[i] = 0x04000000
                        was_updated = True

        return was_updated

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
        writeBytearrayToFile(filepath, self.bytes)
