#!/usr/bin/python3

from __future__ import annotations

from .Utils import *

class Context:
    def __init__(self):
        self.funcsInFiles: Dict[str, List[int]] = dict()
        self.funcAddresses: Dict[int, str] = dict()

        self.labels: Dict[int, str] = dict()
        self.symbols: Dict[int, str] = dict()

        # Where the jump table is
        self.jumpTables: Dict[int, str] = dict()
        # The addresses each jump table has
        self.jumpTablesLabels: Dict[int, str] = dict()

        # Functions jumped into Using J instead of JAL
        self.fakeFunctions: Dict[int, str] = dict()


    def getGenericSymbol(self, vramAddress: int) -> str|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress]

        if vramAddress in self.jumpTables:
            return self.jumpTables[vramAddress]

        if vramAddress in self.symbols:
            return self.symbols[vramAddress]

        return None

    def getGenericLabel(self, vramAddress: int) -> str|None:
        if vramAddress in self.jumpTablesLabels:
            return self.jumpTablesLabels[vramAddress]

        if vramAddress in self.labels:
            return self.labels[vramAddress]

        return None


    def getFunctionName(self, vramAddress: int) -> str|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress]

        return None


    def readFunctionMap(self, version: str):
        functionmap_filename = f"functionmap/{version}.csv"
        if not os.path.exists(functionmap_filename):
            return

        functionmap_file = readCsv(functionmap_filename)
        for row in functionmap_file:
            filename = row[0]
            vram = int(row[1], 16)
            func_name = row[2]

            if filename not in self.funcsInFiles:
                self.funcsInFiles[filename] = []
            self.funcsInFiles[filename].append(vram)
            self.funcAddresses[vram] = func_name
