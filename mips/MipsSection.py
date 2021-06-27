#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase

class Section(FileBase):
    def blankOutDifferences(self, other: FileBase) -> bool:
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
