#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class File:
    def __init__(self, array_of_bytes: bytearray):
        self.bytes: bytearray = array_of_bytes
        self.words: List[int] = bytesToBEWords(self.bytes)

    @property
    def size(self):
        return len(self.bytes)
    @property
    def sizew(self):
        return len(self.words)

    def getHash(self):
        return get_str_hash(self.bytes)

    def compareToFile(self, other_file: File, args):
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
                    #if isinstance(self, Text) and isinstance(other_file, Text):
                        #eprint(f"Differing instruction: {self.instructions[i]}")
                        #eprint(f"Differing instruction: {other_file.instructions[i]}")
                        #eprint(f"")
                        #pass

        return result

    def blankOutDifferences(self, other: File, args):
        was_updated = False
        if args.ignore80 or args.ignore06 or args.ignore04:
            min_len = min(self.sizew, other.sizew)
            for i in range(min_len):
                if args.ignore80:
                    if ((self.words[i] >> 24) & 0xFF) == 0x80 and ((other.words[i] >> 24) & 0xFF) == 0x80:
                        self.words[i] = 0x80000000
                        other.words[i] = 0x80000000
                        was_updated = True
                if args.ignore06:
                    if ((self.words[i] >> 24) & 0xFF) == 0x06 and ((other.words[i] >> 24) & 0xFF) == 0x06:
                        self.words[i] = 0x06000000
                        other.words[i] = 0x06000000
                        was_updated = True
                if args.ignore04:
                    if ((self.words[i] >> 24) & 0xFF) == 0x04 and ((other.words[i] >> 24) & 0xFF) == 0x04:
                        self.words[i] = 0x04000000
                        other.words[i] = 0x04000000
                        was_updated = True
        if was_updated:
            self.updateBytes()
            other.updateBytes()

    def updateBytes(self):
        beWordsToBytes(self.words, self.bytes)
