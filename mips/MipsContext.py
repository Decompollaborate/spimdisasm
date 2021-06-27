#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class ContextFile:
    def __init__(self, name: str, vram: int):
        self.name: str = name
        self.vram: int = vram
        self.references: List[int] = list()


class Context:
    def __init__(self):
        self.files: Dict[str, ContextFile] = dict()

        self.funcsInFiles: Dict[str, List[int]] = dict()
        self.symbolToFile: Dict[int, str] = dict()

        self.funcAddresses: Dict[int, str] = dict()

        self.labels: Dict[int, str] = dict()
        self.symbols: Dict[int, str] = dict()

        # Where the jump table is
        self.jumpTables: Dict[int, str] = dict()
        # The addresses each jump table has
        self.jumpTablesLabels: Dict[int, str] = dict()

        # Functions jumped into Using J instead of JAL
        self.fakeFunctions: Dict[int, str] = dict()


    def getAnySymbol(self, vramAddress: int) -> str|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress]

        if vramAddress in self.jumpTablesLabels:
            return self.jumpTablesLabels[vramAddress]

        if vramAddress in self.labels:
            return self.labels[vramAddress]

        if vramAddress in self.jumpTables:
            return self.jumpTables[vramAddress]

        if vramAddress in self.symbols:
            return self.symbols[vramAddress]

        if vramAddress in self.fakeFunctions:
            return self.fakeFunctions[vramAddress]

        return None

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


    def addFunction(self, filename: str, vramAddress: int, name: str):
        if filename in self.files:
            if vramAddress not in self.files[filename].references:
                self.files[filename].references.append(vramAddress)
        if vramAddress not in self.funcAddresses:
            self.funcAddresses[vramAddress] = name
        if vramAddress not in self.symbolToFile:
            self.symbolToFile[vramAddress] = filename


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
            self.symbolToFile[vram] = filename

    def saveContextToFile(self, filepath: str):
        with open(filepath, "w") as f:
            for address, name in self.funcAddresses.items():
                file = self.symbolToFile.get(address, "")
                jal = (address & 0x00FFFFFF) >> 2
                jal = 0x0C000000 | jal
                f.write(f"{file},{toHex(address, 8)},{name},{toHex(jal, 8)}\n")

            for address, name in self.labels.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"{file},{toHex(address, 8)},{name},\n")

            for address, name in self.symbols.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"{file},{toHex(address, 8)},{name},\n")

            for address, name in self.jumpTables.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"{file},{toHex(address, 8)},{name},\n")

            for address, name in self.jumpTablesLabels.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"{file},{toHex(address, 8)},{name},\n")

            for address, name in self.fakeFunctions.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"{file},{toHex(address, 8)},{name},\n")

