#!/usr/bin/env python3

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig

from .MipsFileBase import FileBase

class Section(FileBase):
    def blankOutDifferences(self, other: FileBase) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        if len(GlobalConfig.IGNORE_WORD_LIST) > 0:
            min_len = min(self.sizew, other.sizew)
            for i in range(min_len):
                for upperByte in GlobalConfig.IGNORE_WORD_LIST:
                    word = upperByte << 24
                    if ((self.words[i] >> 24) & 0xFF) == upperByte and ((other.words[i] >> 24) & 0xFF) == upperByte:
                        self.words[i] = word
                        other.words[i] = word
                        was_updated = True

        return was_updated
