#!/usr/bin/python3

from __future__ import annotations

import ast
from sortedcontainers import SortedDict

from mips.GlobalConfig import GlobalConfig

from .Utils import *

class ContextFile:
    def __init__(self, name: str, vram: int):
        self.name: str = name
        self.vram: int = vram
        #self.references: List[int] = list()

class ContextSegment:
    def __init__(self, segmentName: str, segmentInputPath: str, segmentType: str, subsections):
        self.name: str = segmentName
        self.inputPath: str = segmentInputPath
        self.type: str = segmentType
        self.subsections: list = subsections

class ContextSymbol:
    def __init__(self, vram: int, name: str):
        self.vram: int = vram
        self.name: str = name
        self.type: str = ""
        self.arrayInfo: str = ""
        self.size: int = 4
        self.isDefined = False
        self.isUserDefined = False

class Context:
    def __init__(self):
        self.files: Dict[int, ContextFile] = dict()
        self.segments: Dict[str, ContextSegment] = dict()

        self.funcsInFiles: Dict[str, List[int]] = dict()
        self.symbolToFile: Dict[int, str] = dict()

        self.funcAddresses: Dict[int, ContextSymbol] = dict()

        self.labels: Dict[int, str] = dict()
        # self.symbols: SortedDict[int, ContextSymbol]
        self.symbols = SortedDict()

        # Where the jump table is
        self.jumpTables: Dict[int, str] = dict()
        # The addresses each jump table has
        self.jumpTablesLabels: Dict[int, str] = dict()

        # Functions jumped into Using J instead of JAL
        self.fakeFunctions: Dict[int, str] = dict()


    def getAnySymbol(self, vramAddress: int) -> str|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress].name

        if vramAddress in self.jumpTablesLabels:
            return self.jumpTablesLabels[vramAddress]

        if vramAddress in self.labels:
            return self.labels[vramAddress]

        if vramAddress in self.jumpTables:
            return self.jumpTables[vramAddress]

        if vramAddress in self.symbols:
            return self.symbols[vramAddress].name

        if vramAddress in self.fakeFunctions:
            return self.fakeFunctions[vramAddress]

        return None

    def getGenericSymbol(self, vramAddress: int, tryPlusOffset: bool = True) -> str|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress].name

        if vramAddress in self.jumpTables:
            return self.jumpTables[vramAddress]

        if vramAddress in self.symbols:
            return self.symbols[vramAddress].name

        if GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET and tryPlusOffset:
            rangeObj = self.symbols.irange(maximum=vramAddress, reverse=True)
            for vram in rangeObj:
                contextSym: ContextSymbol = self.symbols[vram]

                symbolName = contextSym.name
                symbolSize = contextSym.size
                if vramAddress > vram and vramAddress < vram + symbolSize:
                    return f"{symbolName} + {toHex(vramAddress - vram, 1)}"

                # Only one iteration
                break

        return None

    def getSymbol(self, vramAddress: int, tryPlusOffset: bool = True, checkUpperLimit: bool = True) -> ContextSymbol|None:
        if vramAddress in self.symbols:
            return self.symbols[vramAddress]

        if GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET and tryPlusOffset:
            rangeObj = self.symbols.irange(maximum=vramAddress, reverse=True)
            for vram in rangeObj:
                contextSym: ContextSymbol = self.symbols[vram]

                symbolSize = contextSym.size
                if vramAddress > vram:
                    if checkUpperLimit:
                        if vramAddress >= vram + symbolSize:
                            break
                    return contextSym

                # Only one iteration
                break
        return None

    def getGenericLabel(self, vramAddress: int) -> str|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress].name

        if vramAddress in self.jumpTablesLabels:
            return self.jumpTablesLabels[vramAddress]

        if vramAddress in self.labels:
            return self.labels[vramAddress]

        return None

    def getFunction(self, vramAddress: int) -> ContextSymbol|None:
        if vramAddress in self.funcAddresses:
            return self.funcAddresses[vramAddress]

        return None


    def addFunction(self, filename: str|None, vramAddress: int, name: str):
        #if filename is not None and filename in self.files:
        #    if vramAddress not in self.files[filename].references:
        #        self.files[filename].references.append(vramAddress)
        if vramAddress not in self.funcAddresses:
            contextSymbol = ContextSymbol(vramAddress, name)
            contextSymbol.type = "@function"
            self.funcAddresses[vramAddress] = contextSymbol
        if vramAddress not in self.symbolToFile and filename is not None:
            self.symbolToFile[vramAddress] = filename

        if vramAddress in self.fakeFunctions:
            del self.fakeFunctions[vramAddress]

        if vramAddress in self.symbols:
            # ??
            self.symbols[vramAddress].isDefined = True

    def addBranchLabel(self, vramAddress: int, name: str):
        if vramAddress not in self.labels:
            self.labels[vramAddress] = name

        if vramAddress in self.symbols:
            # ??
            self.symbols[vramAddress].isDefined = True

    def addJumpTable(self, vramAddress: int, name: str):
        if vramAddress not in self.jumpTables:
            self.jumpTables[vramAddress] = name

        if vramAddress in self.symbols:
            # ??
            self.symbols[vramAddress].isDefined = True

    def addFakeFunction(self, vramAddress: int, name: str):
        if vramAddress not in self.fakeFunctions:
            self.fakeFunctions[vramAddress] = name

        if vramAddress in self.symbols:
            # ??
            self.symbols[vramAddress].isDefined = True


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
            contextFuncSymbol = ContextSymbol(vram, func_name)
            contextFuncSymbol.type = "@function"
            contextFuncSymbol.isUserDefined = True
            self.funcAddresses[vram] = contextFuncSymbol
            self.symbolToFile[vram] = filename

    def readMMAddressMaps(self, filesPath: str, functionsPath: str, variablesPath: str):
        with open(filesPath) as infile:
            files_spec = ast.literal_eval(infile.read())

        for segmentName, segmentInputPath, segmentType, subsections, subfiles  in files_spec:
            self.segments[segmentName] = ContextSegment(segmentName, segmentInputPath, segmentType, subsections)
            for vram, subname in subfiles.items():
                if subname == "":
                    subname = f"{segmentName}_{toHex(vram, 8)[2:]}"
                self.files[vram] = ContextFile(subname, vram)

        with open(functionsPath) as infile:
            functions_ast = ast.literal_eval(infile.read())

        for vram, funcData in functions_ast.items():
            funcName = funcData[0]
            self.addFunction(None, vram, funcName)
            self.funcAddresses[vram].isUserDefined = True

        with open(variablesPath) as infile:
            variables_ast = ast.literal_eval(infile.read())

        for vram, varData in variables_ast.items():
            varName, varType, varArrayInfo, varSize = varData
            contVar = ContextSymbol(vram, varName)
            contVar.type = varType
            contVar.arrayInfo = varArrayInfo
            contVar.size = varSize
            contVar.isUserDefined = True
            self.symbols[vram] = contVar

    def readVariablesCsv(self, filepath: str):
        if not os.path.exists(filepath):
            return

        variables_file = readCsv(filepath)
        for row in variables_file:
            if len(row) == 0:
                continue

            vram, varName, varType, varSize = row

            vram = int(vram, 16)
            varSize = int(varSize, 0)
            contVar = ContextSymbol(vram, varName)
            contVar.type = varType
            contVar.size = varSize
            contVar.isUserDefined = True
            self.symbols[vram] = contVar

    def readFunctionsCsv(self, filepath: str):
        if not os.path.exists(filepath):
            return

        functions_file = readCsv(filepath)
        for row in functions_file:
            if len(row) == 0:
                continue

            vram, funcName = row

            vram = int(vram, 16)
            self.addFunction(None, vram, funcName)
            self.funcAddresses[vram].isUserDefined = True

    def saveContextToFile(self, filepath: str):
        with open(filepath, "w") as f:
            for address, symbol in self.funcAddresses.items():
                file = self.symbolToFile.get(address, "")
                jal = (address & 0x00FFFFFF) >> 2
                jal = 0x0C000000 | jal
                f.write(f"function,{file},{toHex(address, 8)},{symbol.name},{toHex(jal, 8)},{symbol.isDefined}\n")

            for address, name in self.jumpTables.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"jump_table,{file},{toHex(address, 8)},{name},\n")

            for address, name in self.jumpTablesLabels.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"jump_table_label,{file},{toHex(address, 8)},{name},\n")

            for address, name in self.labels.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"label,{file},{toHex(address, 8)},{name},\n")

            for address, symbol in self.symbols.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"symbol,{file},{toHex(address, 8)},{symbol.name},{symbol.type},{symbol.size},{symbol.isDefined}\n")

            for address, name in self.fakeFunctions.items():
                file = self.symbolToFile.get(address, "")
                f.write(f"fake_function,{file},{toHex(address, 8)},{name},\n")

