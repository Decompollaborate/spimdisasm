#!/usr/bin/python3

from __future__ import annotations

import enum

from .Utils import *


@enum.unique
class FileSectionType(enum.Enum):
    Invalid = -1

    Text    = enum.auto()
    Data    = enum.auto()
    Rodata  = enum.auto()
    Bss     = enum.auto()
    Reloc   = enum.auto()


class FileSplitFormat:
    def __init__(self, csvPath: str):
        self.splits = readCsv(csvPath)
        self.splits = [x for x in self.splits if len(x) > 0]

    def __len__(self):
        return len(self.splits)

    def __iter__(self):
        section = FileSectionType.Invalid

        for i, row in enumerate(self.splits):
            offset, vram, fileName = row

            isHandwritten = False
            isRsp = False
            offset = offset.upper()
            if offset[-1] == "H":
                isHandwritten = True
                offset = offset[:-1]
            elif offset[-1] == "R":
                isRsp = True
                offset = offset[:-1]

            if fileName == ".text":
                section = FileSectionType.Text
                continue
            elif fileName == ".data":
                section = FileSectionType.Data
                continue
            elif fileName == ".rodata":
                section = FileSectionType.Rodata
                continue
            elif fileName == ".bss":
                section = FileSectionType.Bss
                continue
            elif fileName == ".end":
                break

            vram = int(vram, 16)
            offset = int(offset, 16)
            nextOffset = 0xFFFFFF
            if i + 1 < len(self.splits):
                if self.splits[i+1][2] == ".end":
                    nextOffsetStr = self.splits[i+1][0]
                elif self.splits[i+1][2].startswith("."):
                    nextOffsetStr = self.splits[i+2][0]
                else:
                    nextOffsetStr = self.splits[i+1][0]
                if nextOffsetStr.upper()[-1] == "H":
                    nextOffsetStr = nextOffsetStr[:-1]
                nextOffset = int(nextOffsetStr, 16)

            yield (offset, vram, fileName, section, nextOffset, isHandwritten, isRsp)
