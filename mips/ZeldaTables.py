#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class DmaEntry:
    def __init__(self, vromStart: int, vromEnd: int, romStart: int, romEnd: int):
        self.vromStart: int = vromStart
        self.vromEnd: int = vromEnd
        self.romStart: int = romStart
        self.romEnd: int = romEnd


def getDmaAddresses(version: str) -> Dict[str, DmaEntry]:
    filetable = f'baserom_{version}/dma_addresses.txt'
    table: Dict[str, DmaEntry] = dict()
    if os.path.exists(filetable):
        with open(filetable) as f:
            for line in f:
                filename, *data = line.strip().split(",")
                virtStart, virtEnd, physStart, physEnd = map(int, data)
                table[filename] = DmaEntry(virtStart, virtEnd, physStart, physEnd)
    return table


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
