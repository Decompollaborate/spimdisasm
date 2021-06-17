#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .MipsFile import File


class RelocEntry:
    def __init__(self, entry: int):
        self.sectionId = entry >> 30
        self.relocType = (entry >> 24) & 0x3F
        self.offset = entry & 0x00FFFFFF

class Reloc(File):
    def __init__(self, array_of_bytes):
        super().__init__(array_of_bytes)

        self.entries: List[RelocEntry] = list()
        for word in self.words:
            self.entries.append(RelocEntry(word))

    @property
    def nRelocs(self):
        return len(self.entries)

    def compareToFile(self, other_file: File, args):
        result = super().compareToFile(other_file, args)
        # TODO
        return result
