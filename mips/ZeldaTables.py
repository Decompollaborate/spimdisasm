#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class OverlayTableEntry:
    def __init__(self, array_of_bytes: bytearray):
        wordsArray = bytesToBEWords(array_of_bytes)
        self.vromStart = wordsArray[0]
        self.vromEnd = wordsArray[1]
        self.vramStart = wordsArray[2]
        self.vramEnd = wordsArray[3]
        self.ramAddress = wordsArray[4]
        self.initVars = wordsArray[5]
        self.filenameAddres = wordsArray[6]
        self.allocationType = (wordsArray[7] > 16) & 0xFFFF
        self.instancesNum = (wordsArray[7] > 8) & 0xFF
