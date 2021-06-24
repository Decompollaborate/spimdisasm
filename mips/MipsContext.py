#!/usr/bin/python3

from __future__ import annotations

from .Utils import *

class Context:
    def __init__(self):
        self.funcsInFiles: Dict[str, List[int]] = dict()
        self.funcAddresses: Dict[int, str] = dict()

        self.labels: Dict[int, str] = dict()
        self.symbols: Dict[int, str] = dict()
        # Functions jumped into Using J instead of JAL
        self.fakeFunctions: Dict[int, str] = dict()

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
