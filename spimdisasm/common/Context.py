#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import ast
import argparse
from typing import Generator
import os

from . import Utils
from .FileSectionType import FileSectionType
from .ContextSymbols import SymbolSpecialType, ContextSymbol, ContextOffsetSymbol, ContextRelocSymbol
from .SymbolsSegment import SymbolsSegment


class Context:
    N64DefaultBanned = {0x80000010, 0x80000020}

    N64LibultraSyms = {
        0x80000300: ("osTvType",       "u32", 0x4),
        0x80000304: ("osRomType",      "u32", 0x4),
        0x80000308: ("osRomBase",      "u32", 0x4),
        0x8000030C: ("osResetType",    "u32", 0x4),
        0x80000310: ("osCicId",        "u32", 0x4),
        0x80000314: ("osVersion",      "u32", 0x4),
        0x80000304: ("osRomType",      "u32", 0x4),
        0x80000318: ("osMemSize",      "u32", 0x4),
        0x8000031C: ("osAppNmiBuffer", "u8",  0x40),
    }

    N64HardwareRegs = {
        # Signal Processor Registers
        0xA4040000: "SP_MEM_ADDR_REG",
        0xA4040004: "SP_DRAM_ADDR_REG",
        0xA4040008: "SP_RD_LEN_REG",
        0xA404000C: "SP_WR_LEN_REG",
        0xA4040010: "SP_STATUS_REG",
        0xA4040014: "SP_DMA_FULL_REG",
        0xA4040018: "SP_DMA_BUSY_REG",
        0xA404001C: "SP_SEMAPHORE_REG",

        0xA4080000: "SP_PC",

        # Display Processor Command Registers / Rasterizer Interface
        0xA4100000: "DPC_START_REG",
        0xA4100004: "DPC_END_REG",
        0xA4100008: "DPC_CURRENT_REG",
        0xA410000C: "DPC_STATUS_REG",
        0xA4100010: "DPC_CLOCK_REG",
        0xA4100014: "DPC_BUFBUSY_REG",
        0xA4100018: "DPC_PIPEBUSY_REG",
        0xA410001C: "DPC_TMEM_REG",

        # Display Processor Span Registers
        0xA4200000: "DPS_TBIST_REG", # DPS_TBIST_REG / DP_TMEM_BIST
        0xA4200004: "DPS_TEST_MODE_REG",
        0xA4200008: "DPS_BUFTEST_ADDR_REG",
        0xA420000C: "DPS_BUFTEST_DATA_REG",

        # MIPS Interface Registers
        0xA4300000: "MI_MODE_REG", # MI_MODE_REG / MI_INIT_MODE_REG
        0xA4300004: "MI_VERSION_REG",
        0xA4300008: "MI_INTR_REG",
        0xA430000C: "MI_INTR_MASK_REG",

        # Video Interface Registers
        0xA4400000: "VI_STATUS_REG", # VI_STATUS_REG / VI_CONTROL_REG
        0xA4400004: "VI_DRAM_ADDR_REG", # VI_DRAM_ADDR_REG / VI_ORIGIN_REG
        0xA4400008: "VI_WIDTH_REG",
        0xA440000C: "VI_INTR_REG",
        0xA4400010: "VI_CURRENT_REG",
        0xA4400014: "VI_BURST_REG", # VI_BURST_REG / VI_TIMING_REG
        0xA4400018: "VI_V_SYNC_REG",
        0xA440001C: "VI_H_SYNC_REG",
        0xA4400020: "VI_LEAP_REG",
        0xA4400024: "VI_H_START_REG",
        0xA4400028: "VI_V_START_REG",
        0xA440002C: "VI_V_BURST_REG",
        0xA4400030: "VI_X_SCALE_REG",
        0xA4400034: "VI_Y_SCALE_REG",

        # Audio Interface Registers
        0xA4500000: "AI_DRAM_ADDR_REG",
        0xA4500004: "AI_LEN_REG",
        0xA4500008: "AI_CONTROL_REG",
        0xA450000C: "AI_STATUS_REG",
        0xA4500010: "AI_DACRATE_REG",
        0xA4500014: "AI_BITRATE_REG",

        # Peripheral/Parallel Interface Registers
        0xA4600000: "PI_DRAM_ADDR_REG",
        0xA4600004: "PI_CART_ADDR_REG",
        0xA4600005: "D_A4600005", # TODO: figure out its name
        0xA4600006: "D_A4600006", # TODO: figure out its name
        0xA4600007: "D_A4600007", # TODO: figure out its name
        0xA4600008: "PI_RD_LEN_REG",
        0xA460000C: "PI_WR_LEN_REG",
        0xA4600010: "PI_STATUS_REG",
        0xA4600014: "PI_BSD_DOM1_LAT_REG", # PI dom1 latency
        0xA4600018: "PI_BSD_DOM1_PWD_REG", # PI dom1 pulse width
        0xA460001C: "PI_BSD_DOM1_PGS_REG", # PI dom1 page size
        0xA4600020: "PI_BSD_DOM1_RLS_REG", # PI dom1 release
        0xA4600024: "PI_BSD_DOM2_LAT_REG", # PI dom2 latency
        0xA4600028: "PI_BSD_DOM2_LWD_REG", # PI dom2 pulse width
        0xA460002C: "PI_BSD_DOM2_PGS_REG", # PI dom2 page size
        0xA4600030: "PI_BSD_DOM2_RLS_REG", # PI dom2 release

        # RDRAM Interface Registers
        0xA4700000: "RI_MODE_REG",
        0xA4700004: "RI_CONFIG_REG",
        0xA4700008: "RI_CURRENT_LOAD_REG",
        0xA470000C: "RI_SELECT_REG",
        0xA4700010: "RI_REFRESH_REG",
        0xA4700014: "RI_LATENCY_REG",
        0xA4700018: "RI_RERROR_REG",
        0xA470001C: "RI_WERROR_REG",

        # Serial Interface Registers
        0xA4800000: "SI_DRAM_ADDR_REG",
        0xA4800004: "SI_PIF_ADDR_RD64B_REG",
        0xA4800008: "D_A4800008", # reserved
        0xA480000C: "D_A480000C", # reserved
        0xA4800010: "SI_PIF_ADDR_WR64B_REG",
        0xA4800014: "D_A4800014", # reserved
        0xA4800018: "SI_STATUS_REG",
    }
    "N64 OS hardware registers"

    def __init__(self):
        # Arbitrary initial range
        self.globalSegment = SymbolsSegment(0x80000000, 0x80001000)
        # For symbols that we don't know where they come from
        self.unknownSegment = SymbolsSegment(0x00000000, 0xFFFFFFFF)

        self.newPointersInData: set[int] = set()

        self.loPatches: dict[int, int] = dict()
        "key: address of %lo, value: symbol's vram to use instead"

        self.dataSymbolsWithReferencesWithAddends: set[int] = set()
        "Contains the address of data symbols which are allowed to have references to other symbols with addends"

        self.dataReferencingConstants: set[int] = set()
        "Set of addresses of data symbols which are allowed to reference named constants"

        # Stuff that looks like pointers, but the disassembler shouldn't count it as a pointer
        self.bannedSymbols: set[int] = set()

        # First key is the section type, sub key is offset relative to the start of that section
        self.offsetSymbols: dict[FileSectionType, dict[int, ContextOffsetSymbol]] = {
            FileSectionType.Text: dict(),
            FileSectionType.Data: dict(),
            FileSectionType.Rodata: dict(),
            FileSectionType.Bss: dict(),
        }

        self.relocSymbols: dict[FileSectionType, dict[int, ContextRelocSymbol]] = {
            FileSectionType.Text: dict(),
            FileSectionType.Data: dict(),
            FileSectionType.Rodata: dict(),
            FileSectionType.Bss: dict(),
        }

        # Where the jump table is
        self.offsetJumpTables: dict[int, ContextOffsetSymbol] = dict()
        # The addresses every jump table has
        self.offsetJumpTablesLabels: dict[int, ContextOffsetSymbol] = dict()


    def getSymbol(self, vramAddress: int, tryPlusOffset: bool = True, checkUpperLimit: bool = True) -> ContextSymbol|None:
        "Searches symbol or a symbol with an addend if `tryPlusOffset` is True"
        return self.globalSegment.getSymbol(vramAddress, tryPlusOffset=tryPlusOffset, checkUpperLimit=checkUpperLimit)

    def getSymbolRangeIter(self, addressStart: int, addressEnd: int) -> Generator[ContextSymbol, None, None]:
        return self.globalSegment.getSymbolRangeIter(addressStart, addressEnd)

    def getSymbolRange(self, addressStart: int, addressEnd: int):
        return self.globalSegment.getSymbolRange(addressStart, addressEnd)

    def getConstant(self, constantValue: int) -> ContextSymbol|None:
        return self.globalSegment.getConstant(constantValue)


    def getLoPatch(self, loInstrVram: int|None) -> int|None:
        if loInstrVram is None:
            return None
        return self.loPatches.get(loInstrVram, None)


    def getOffsetSymbol(self, offset: int, sectionType: FileSectionType) -> ContextOffsetSymbol|None:
        if sectionType in self.offsetSymbols:
            symbolsInSection = self.offsetSymbols[sectionType]
            if offset in symbolsInSection:
                return symbolsInSection[offset]
        return None

    def getOffsetGenericSymbol(self, offset: int, sectionType: FileSectionType) -> ContextOffsetSymbol|None:
        if offset in self.offsetJumpTables:
            return self.offsetJumpTables[offset]

        if sectionType in self.offsetSymbols:
            symbolsInSection = self.offsetSymbols[sectionType]
            if offset in symbolsInSection:
                return symbolsInSection[offset]

        return None

    def getRelocSymbol(self, offset: int, sectionType: FileSectionType) -> ContextRelocSymbol|None:
        if sectionType in self.relocSymbols:
            relocsInSection = self.relocSymbols[sectionType]
            if offset in relocsInSection:
                return relocsInSection[offset]
        return None

    def getOffsetGenericLabel(self, offset: int, sectionType: FileSectionType) -> ContextOffsetSymbol|None:
        if offset in self.offsetJumpTablesLabels:
            return self.offsetJumpTablesLabels[offset]
        return None


    def addSymbol(self, vramAddress: int, sectionType: FileSectionType=FileSectionType.Unknown, isAutogenerated: bool=False) -> ContextSymbol:
        return self.globalSegment.addSymbol(vramAddress, sectionType=sectionType, isAutogenerated=isAutogenerated)

    def addFunction(self, vramAddress: int, isAutogenerated: bool=False) -> ContextSymbol:
        return self.globalSegment.addFunction(vramAddress, isAutogenerated=isAutogenerated)

    def addBranchLabel(self, vramAddress: int, isAutogenerated: bool=False) -> ContextSymbol:
        return self.globalSegment.addBranchLabel(vramAddress, isAutogenerated=isAutogenerated)

    def addJumpTable(self, vramAddress: int, isAutogenerated: bool=False) -> ContextSymbol:
        return self.globalSegment.addJumpTable(vramAddress, isAutogenerated=isAutogenerated)

    def addJumpTableLabel(self, vramAddress: int, isAutogenerated: bool=False) -> ContextSymbol:
        return self.globalSegment.addJumpTableLabel(vramAddress, isAutogenerated=isAutogenerated)


    def addConstant(self, constantValue: int, name: str) -> ContextSymbol:
        return self.globalSegment.addConstant(constantValue, name)


    def addOffsetJumpTable(self, offset: int, sectionType: FileSectionType) -> ContextOffsetSymbol:
        if offset not in self.offsetJumpTables:
            contextOffsetSym = ContextOffsetSymbol(offset, f"jtbl_{offset:06X}", sectionType)
            contextOffsetSym.type = SymbolSpecialType.jumptable
            self.offsetJumpTables[offset] = contextOffsetSym
            return contextOffsetSym
        return self.offsetJumpTables[offset]

    def addOffsetJumpTableLabel(self, offset: int, name: str, sectionType: FileSectionType) -> ContextOffsetSymbol:
        if offset not in self.offsetJumpTablesLabels:
            contextOffsetSym = ContextOffsetSymbol(offset, name, sectionType)
            contextOffsetSym.type = SymbolSpecialType.jumptablelabel
            self.offsetJumpTablesLabels[offset] = contextOffsetSym
            return contextOffsetSym
        return self.offsetJumpTablesLabels[offset]


    def fillDefaultBannedSymbols(self):
        self.bannedSymbols |= self.N64DefaultBanned

    def fillLibultraSymbols(self):
        for vram, (name, type, size) in self.N64LibultraSyms.items():
            contextSym = self.addSymbol(vram, name)
            contextSym.type = type
            contextSym.size = size
            contextSym.isDefined = True
            contextSym.isUserDeclared = True

    def fillHardwareRegs(self, useRealNames: bool=False):
        for vram, name in self.N64HardwareRegs.items():
            nameToUse = None
            if useRealNames:
                nameToUse = name
            contextSym = self.addSymbol(vram)
            contextSym.name = nameToUse
            contextSym.type = SymbolSpecialType.hardwarereg
            contextSym.size = 4
            contextSym.isDefined = True
            contextSym.isUserDeclared = True


    def readMMAddressMaps(self, functionsPath: str, variablesPath: str):
        with open(functionsPath) as infile:
            functions_ast = ast.literal_eval(infile.read())

        for vram, funcData in functions_ast.items():
            funcName = funcData[0]
            contextSym = self.addFunction(vram, funcName)
            contextSym.isUserDeclared = True

        with open(variablesPath) as infile:
            variables_ast = ast.literal_eval(infile.read())

        for vram, varData in variables_ast.items():
            varName, varType, varArrayInfo, varSize = varData
            if varType == "":
                varType = None

            contextSym = self.addSymbol(vram, varName)
            contextSym.type = varType
            contextSym.size = varSize
            contextSym.isUserDeclared = True

    def readVariablesCsv(self, filepath: str):
        if not os.path.exists(filepath):
            return

        variables_file = Utils.readCsv(filepath)
        for row in variables_file:
            if len(row) == 0:
                continue

            varType: SymbolSpecialType|str|None
            vramStr, varName, varType, varSizeStr = row

            if vramStr == "-":
                continue

            vram = int(vramStr, 16)
            varSize = int(varSizeStr, 16)
            if varType == "":
                varType = None

            specialType = SymbolSpecialType.fromStr(varType)
            if specialType is not None:
                varType = specialType
                if specialType == SymbolSpecialType.function:
                    contextSym = self.addFunction(vram)
                elif specialType == SymbolSpecialType.branchlabel:
                    contextSym = self.addBranchLabel(vram)
                elif specialType == SymbolSpecialType.jumptable:
                    contextSym = self.addJumpTable(vram)
                elif specialType == SymbolSpecialType.jumptablelabel:
                    contextSym = self.addJumpTableLabel(vram)
                elif specialType == SymbolSpecialType.hardwarereg:
                    contextSym = self.addSymbol(vram)
                else:
                    contextSym = self.addSymbol(vram)
            else:
                contextSym = self.addSymbol(vram)

            contextSym.name = varName
            contextSym.type = varType
            contextSym.size = varSize
            contextSym.isUserDeclared = True

    def readFunctionsCsv(self, filepath: str):
        if not os.path.exists(filepath):
            return

        functions_file = Utils.readCsv(filepath)
        for row in functions_file:
            if len(row) == 0:
                continue

            vramStr, funcName = row

            if vramStr == "-":
                continue

            vram = int(vramStr, 16)
            contextSym = self.addFunction(vram)
            contextSym.name = funcName
            contextSym.isUserDeclared = True

    def readConstantsCsv(self, filepath: str):
        if not os.path.exists(filepath):
            return

        constants_file = Utils.readCsv(filepath)
        for row in constants_file:
            if len(row) == 0:
                continue

            constantValueStr, constantName = row

            if constantValueStr == "-":
                continue

            constantValue = int(constantValueStr, 16)
            contextSym = self.addConstant(constantValue, constantName)
            contextSym.isUserDeclared = True


    def saveContextToFile(self, filepath: str):
        with open(filepath, "w") as f:
            self.globalSegment.saveContextToFile(f)

            for address in self.newPointersInData:
                f.write(f"new_pointer_in_data,0x{address:08X}\n")

            for address in self.bannedSymbols:
                f.write(f"banned_symbol,0x{address:08X}\n")


    @staticmethod
    def addParametersToArgParse(parser: argparse.ArgumentParser):
        contextParser = parser.add_argument_group("Context configuration")

        contextParser.add_argument("--save-context", help="Saves the context to a file", metavar="FILENAME")


        csvConfig = parser.add_argument_group("Context .csv input files")

        csvConfig.add_argument("--functions", help="Path to a functions csv", action="append")
        csvConfig.add_argument("--variables", help="Path to a variables csv", action="append")
        csvConfig.add_argument("--constants", help="Path to a constants csv", action="append")


        symbolsConfig = parser.add_argument_group("Context default symbols configuration")

        symbolsConfig.add_argument("--default-banned", help="Toggles filling the list of default banned symbols. Defaults to True", action=argparse.BooleanOptionalAction)
        symbolsConfig.add_argument("--libultra-syms", help="Toggles using the built-in libultra symbols. Defaults to True", action=argparse.BooleanOptionalAction)
        symbolsConfig.add_argument("--hardware-regs", help="Toggles using the built-in hardware registers symbols. Defaults to True", action=argparse.BooleanOptionalAction)
        symbolsConfig.add_argument("--named-hardware-regs", help="Use actual names for the hardware registers", action=argparse.BooleanOptionalAction)


    def parseArgs(self, args: argparse.Namespace):
        if args.default_banned != False:
            self.fillDefaultBannedSymbols()
        if args.libultra_syms != False:
            self.fillLibultraSymbols()
        if not args.hardware_regs != False:
            self.fillHardwareRegs(args.named_hardware_regs)

        if args.functions is not None:
            for funcsPath in args.functions:
                self.readFunctionsCsv(funcsPath)
        if args.variables is not None:
            for varsPath in args.variables:
                self.readVariablesCsv(varsPath)
        if args.constants is not None:
            for constantsPath in args.constants:
                self.readConstantsCsv(constantsPath)
