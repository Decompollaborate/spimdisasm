#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from typing import TextIO, Generator, TYPE_CHECKING
from pathlib import Path

from . import Utils
from .SortedDict import SortedDict
from .GlobalConfig import GlobalConfig
from .FileSectionType import FileSectionType
from .ContextSymbols import SymbolSpecialType, ContextSymbol

if TYPE_CHECKING:
    from .Context import Context

class SymbolsSegment:
    def __init__(self, context: "Context", vromStart: int|None, vromEnd: int|None, vramStart: int, vramEnd: int, overlayCategory: str|None=None) -> None:
        assert vramStart < vramEnd
        if vromStart is not None and vromEnd is not None:
            assert vromStart <= vromEnd, f"0x{vromStart:06X} <= 0x{vromEnd:06X}"

        self.vromStart: int|None = vromStart
        self.vromEnd: int|None = vromEnd

        self.vramStart: int = vramStart
        self.vramEnd: int = vramEnd

        self.context: "Context" = context

        self.overlayCategory: str|None = overlayCategory

        self.symbols: SortedDict[ContextSymbol] = SortedDict()

        self.constants: dict[int, ContextSymbol] = dict()

        self.newPointersInData: SortedDict[int] = SortedDict()
        "Stuff that looks like pointers, found referenced by data"

        self._isTheUnknownSegment: bool = False


    @property
    def vromSize(self) -> int|None:
        if self.vromStart is None:
            return None
        if self.vromEnd is None:
            return None
        return self.vromEnd - self.vromStart

    @property
    def vramSize(self) -> int:
        return self.vramEnd - self.vramStart


    def isVromInRange(self, vrom: int) -> bool:
        if self.vromStart is None:
            return False
        if self.vromEnd is None:
            return False
        return self.vromStart <= vrom < self.vromEnd

    def isVramInRange(self, vram: int) -> bool:
        return self.vramStart <= vram < self.vramEnd

    def changeRanges(self, vromStart: int, vromEnd: int, vramStart: int, vramEnd: int) -> None:
        assert vromStart <= vromEnd, f"0x{vromStart:06X} <= 0x{vromEnd:06X}"
        assert vramStart <= vramEnd, f"0x{vramStart:08X} <= 0x{vramEnd:08X}"

        self.vromStart = vromStart
        self.vromEnd = vromEnd

        self.vramStart = vramStart
        self.vramEnd = vramEnd


    def vromToVram(self, vrom: int) -> int|None:
        if self.vromStart is None:
            return None
        return vrom - self.vromStart + self.vramStart


    def addSymbol(self, address: int, sectionType: FileSectionType=FileSectionType.Unknown, isAutogenerated: bool=False, vromAddress: int|None=None, allowAddendInstead: bool=False) -> ContextSymbol:
        contextSym = self.getSymbol(address, tryPlusOffset=allowAddendInstead, checkUpperLimit=True)
        if contextSym is None:
            contextSym = ContextSymbol(address)
            contextSym.isAutogenerated = isAutogenerated
            contextSym.sectionType = sectionType
            contextSym.overlayCategory = self.overlayCategory
            self.symbols[address] = contextSym

        if contextSym.sectionType == FileSectionType.Unknown:
            contextSym.sectionType = sectionType

        if contextSym.vromAddress is None and vromAddress is not None:
            contextSym.vromAddress = vromAddress

        if self.vromStart is None or self.vromEnd is None or self._isTheUnknownSegment:
            contextSym.unknownSegment = True

        return contextSym

    def addFunction(self, address: int, isAutogenerated: bool=False, vromAddress: int|None=None) -> ContextSymbol:
        contextSym = self.addSymbol(address, sectionType=FileSectionType.Text, isAutogenerated=isAutogenerated, vromAddress=vromAddress)
        if contextSym.getTypeSpecial() != SymbolSpecialType.jumptablelabel:
            contextSym.setTypeSpecial(SymbolSpecialType.function, isAutogenerated)
        contextSym.sectionType = FileSectionType.Text
        return contextSym

    def addBranchLabel(self, address: int, isAutogenerated: bool=False, vromAddress: int|None=None) -> ContextSymbol:
        contextSym = self.addSymbol(address, sectionType=FileSectionType.Text, isAutogenerated=isAutogenerated, vromAddress=vromAddress)
        currentType = contextSym.getTypeSpecial()
        if currentType != SymbolSpecialType.jumptablelabel and currentType != SymbolSpecialType.function:
            contextSym.setTypeSpecial(SymbolSpecialType.branchlabel, isAutogenerated)
        return contextSym

    def addJumpTable(self, address: int, isAutogenerated: bool=False, vromAddress: int|None=None) -> ContextSymbol:
        contextSym = self.addSymbol(address, sectionType=FileSectionType.Rodata, isAutogenerated=isAutogenerated, vromAddress=vromAddress)
        if contextSym.getTypeSpecial() != SymbolSpecialType.function:
            contextSym.setTypeSpecial(SymbolSpecialType.jumptable, isAutogenerated)
        return contextSym

    def addJumpTableLabel(self, address: int, isAutogenerated: bool=False, vromAddress: int|None=None) -> ContextSymbol:
        contextSym = self.addSymbol(address, sectionType=FileSectionType.Text, isAutogenerated=isAutogenerated, vromAddress=vromAddress)
        contextSym.setTypeSpecial(SymbolSpecialType.jumptablelabel, isAutogenerated)
        contextSym.sectionType = FileSectionType.Text
        return contextSym

    def addGccExceptTable(self, address: int, isAutogenerated: bool=False, vromAddress: int|None=None) -> ContextSymbol:
        contextSym = self.addSymbol(address, sectionType=FileSectionType.GccExceptTable, isAutogenerated=isAutogenerated, vromAddress=vromAddress)
        contextSym.setTypeSpecial(SymbolSpecialType.gccexcepttable, isAutogenerated)
        return contextSym

    def addGccExceptTableLabel(self, address: int, isAutogenerated: bool=False, vromAddress: int|None=None) -> ContextSymbol:
        contextSym = self.addSymbol(address, sectionType=FileSectionType.Text, isAutogenerated=isAutogenerated, vromAddress=vromAddress)
        contextSym.setTypeSpecial(SymbolSpecialType.gccexcepttablelabel, isAutogenerated)
        contextSym.sectionType = FileSectionType.Text
        return contextSym


    def removeSymbol(self, address: int) -> None:
        if address not in self.symbols:
            return

        self.symbols.remove(address)


    def addConstant(self, constantValue: int, name: str, isAutogenerated: bool=False) -> ContextSymbol:
        if constantValue not in self.constants:
            contextSym = ContextSymbol(constantValue)
            contextSym.name = name
            contextSym.setTypeSpecial(SymbolSpecialType.constant, isAutogenerated)
            self.constants[constantValue] = contextSym
            return contextSym
        return self.constants[constantValue]


    def getSymbol(self, address: int, tryPlusOffset: bool = True, checkUpperLimit: bool = True) -> ContextSymbol|None:
        "Searches symbol or a symbol with an addend if `tryPlusOffset` is True"
        if GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET and tryPlusOffset:
            pair = self.symbols.getKeyRight(address, inclusive=True)
            if pair is None:
                return None

            symVram, contextSym = pair
            if checkUpperLimit and address >= symVram + contextSym.getSize():
                return None
            return contextSym

        return self.symbols.get(address, None)

    def getSymbolsRange(self, addressStart: int, addressEnd: int) -> Generator[tuple[int, ContextSymbol], None, None]:
        return self.symbols.getRange(addressStart, addressEnd, startInclusive=True, endInclusive=False)

    def getConstant(self, constantValue: int) -> ContextSymbol|None:
        return self.constants.get(constantValue, None)


    def addPointerInDataReference(self, pointer: int) -> None:
        self.newPointersInData[pointer] = pointer

    def popPointerInDataReference(self, pointer: int) -> int|None:
        return self.newPointersInData.pop(pointer, None)

    def getAndPopPointerInDataReferencesRange(self, low: int, high: int) -> Generator[int, None, None]:
        for key, _ in self.newPointersInData.getRangeAndPop(low, high, startInclusive=True, endInclusive=False):
            yield key


    def saveContextToFile(self, f: TextIO) -> None:
        f.write(f"category,{ContextSymbol.getCsvHeader()}\n")

        for address in self.symbols:
            f.write(f"symbol,{self.symbols[address].toCsv()}\n")

        for address, constant in self.constants.items():
            f.write(f"constants,{constant.toCsv()}\n")

        for address in self.newPointersInData:
            f.write(f"new_pointer_in_data,0x{address:08X}\n")


    N64LibultraSyms: dict[int, tuple[str, str, int]] = {
        0x800001A0: ("leoBootID",      "u32", 0x4),
        0x80000300: ("osTvType",       "u32", 0x4),
        0x80000304: ("osRomType",      "u32", 0x4),
        0x80000308: ("osRomBase",      "u32", 0x4),
        0x8000030C: ("osResetType",    "u32", 0x4),
        0x80000310: ("osCicId",        "u32", 0x4),
        0x80000314: ("osVersion",      "u32", 0x4),
        0x80000318: ("osMemSize",      "u32", 0x4),
        0x8000031C: ("osAppNMIBuffer", "u8",  0x40),
    }

    # iQue specific symbols
    iQueLibultraSyms: dict[int, tuple[str, str, int]] = {
        0x8000035c: ("__osBbEepromAddress", "u32",  0x4),
        0x80000360: ("__osBbEepromSize",    "u32",  0x4),
        0x80000364: ("__osBbFlashAddress",  "u32",  0x4),
        0x80000368: ("__osBbFlashSize",     "u32",  0x4),
        0x8000036c: ("__osBbSramAddress",   "u32",  0x4),
        0x80000370: ("__osBbSramSize",      "u32",  0x4),
        0x80000374: ("__osBbPakAddress",    "u32",  0x4 * 4),
        0x80000384: ("__osBbPakSize",       "u32",  0x4),
        0x80000388: ("__osBbIsBb",          "u32",  0x4),
        0x8000038c: ("__osBbHackFlags",     "u32",  0x4),
        0x80000390: ("__osBbStashMagic",    "u32",  0x4),
        0x80000394: ("__osBbPakBindings",   "s32",  0x4 * 4),
        0x800003a4: ("__osBbStateName",     "char", 0x10),
        0x800003b4: ("__osBbStateDirty",    "u32",  0x4),
        0x800003b8: ("__osBbAuxDataLimit",  "u32",  0x4),
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
        0xA4300010: "D_A4300010",
        0xA4300014: "MI_SK_EXCEPTION_REG",
        0xA4300018: "MI_SK_WATCHDOG_TIMER",
        0xA4300028: "D_A4300028",
        0xA430002C: "MI_RANDOM_BIT",
        0xA4300030: "D_A4300030",
        0xA4300038: "MI_HW_INTR_REG",
        0xA430003C: "MI_HW_INTR_MASK_REG",

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
        0xA4600038: "PI_CARD_STATUS_REG",
        0xA4600040: "PI_ATB_NEXT_CONFIG",
        0xA4600044: "D_A4600044",
        0xA4600048: "PI_CARD_CNT_REG",
        0xA460004C: "PI_CARD_CONFIG_REG",
        0xA4600050: "PI_AES_CNT",
        0xA4600054: "PI_ALLOWED_IO",
        0xA4600058: "PI_EX_RD_LEN_REG",
        0xA460005C: "PI_EX_WR_LEN_REG",
        0xA4600060: "PI_MISC_REG",
        0xA4600064: "D_A4600064",
        0xA4600070: "PI_CARD_BLK_OFFSET_REG",
        0xA4610000: "PI_EX_DMA_BUF",
        0xA4610420: "PI_AES_EXPANDED_KEY",
        0xA46104D0: "PI_AES_IV",
        0xA4610500: "PI_ATB_ENTRY",
        0xA4620000: "D_A4620000",
        0xA46E0000: "PI_RDB_REQ_HI_REG",
        0xA46E0002: "PI_RDB_REQ_LO_REG",
        0xA46E0004: "D_A46E0004",
        0xA46E0400: "D_A46E0400",
        0xA46E0402: "D_A46E0402",
        0xA46E8000: "PI_RDB_STATUS_REG",

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
        0xA480001C: "D_A480001C",

        0xA4900000: "D_A4900000",
        0xA4940010: "USB0_STATUS_REG",
        0xA4A00000: "D_A4A00000",
        0xA4A40010: "USB1_STATUS_REG",

        # libleo (64DD) address range
        0xA5000508: "LEO_CMD",
        0xA5000508: "LEO_STATUS",

        0xA5000510: "LEO_BM_CTL",
        0xA5000510: "LEO_BM_STATUS",

        0xA5000518: "LEO_SEQ_CTL",
        0xA5000518: "LEO_SEQ_STATUS",


        0xA5000000: "LEO_C2_BUFF",      # C2 Sector Buffer
        0xA5000400: "LEO_SECTOR_BUFF",  # Data Sector Buffer
        0xA5000500: "LEO_DATA",         # Data
        0xA5000504: "LEO_MISC_REG",     # Misc Register

        0xA500050C: "LEO_CUR_TK",       # Current Track

        0xA5000514: "LEO_ERR_SECTOR",   # Sector Error Status

        0xA500051C: "LEO_CUR_SECTOR",   # Current Sector
        0xA5000520: "LEO_HARD_RESET",   # Hard Reset
        0xA5000524: "LEO_C1_S0",        # C1
        0xA5000528: "LEO_HOST_SECBYTE", # Sector Size (in bytes)
        0xA500052C: "LEO_C1_S2",        # C1
        0xA5000530: "LEO_SEC_BYTE",     # Sectors per Block, Full Size
        0xA5000534: "LEO_C1_S4",        # C1
        0xA5000538: "LEO_C1_S6",        # C1
        0xA500053C: "LEO_CUR_ADDR",     # Current Address?
        0xA5000540: "LEO_ID_REG",       # ID
        0xA5000544: "LEO_TEST_REG",     # Test Read
        0xA5000548: "LEO_TEST_PIN_SEL", # Test Write
        0xA5000580: "LEO_RAM_ADDR",     # Microsequencer RAM
    }
    "N64 OS hardware registers"

    iQueHardwareReg: dict[int, str] = {
    }
    "iQue OS hardware registers"


    def fillLibultraSymbols(self) -> None:
        lowestVram = 0xFFFFFFFF
        highestVram = 0x00000000
        for vram, (name, type, size) in self.N64LibultraSyms.items():
            contextSym = self.addSymbol(vram)
            contextSym.name = name
            contextSym.setTypeSpecial(type, isAutogenerated=False)
            contextSym.userDeclaredSize = size
            contextSym.isDefined = True
            contextSym.isUserDeclared = True
            if vram > highestVram:
                highestVram = vram
            if vram < lowestVram:
                lowestVram = vram
        self.context.totalVramRange.addSpecialRange(lowestVram, highestVram)

    def fillIQueSymbols(self) -> None:
        lowestVram = 0xFFFFFFFF
        highestVram = 0x00000000
        for vram, (name, type, size) in self.iQueLibultraSyms.items():
            contextSym = self.addSymbol(vram)
            contextSym.name = name
            contextSym.setTypeSpecial(type, isAutogenerated=False)
            contextSym.userDeclaredSize = size
            contextSym.isDefined = True
            contextSym.isUserDeclared = True
            if vram > highestVram:
                highestVram = vram
            if vram < lowestVram:
                lowestVram = vram
        self.context.totalVramRange.addSpecialRange(lowestVram, highestVram)

    def fillHardwareRegs(self, useRealNames: bool=False) -> None:
        lowestVram = 0xFFFFFFFF
        highestVram = 0x00000000
        for vram, name in self.N64HardwareRegs.items():
            nameToUse = None
            if useRealNames:
                nameToUse = name
            contextSym = self.addSymbol(vram)
            contextSym.name = nameToUse
            contextSym.setTypeSpecial(SymbolSpecialType.hardwarereg, isAutogenerated=False)
            contextSym.userDeclaredSize = 4
            contextSym.isDefined = True
            contextSym.isUserDeclared = True

            if useRealNames:
                contextSym = self.addConstant(vram, name, isAutogenerated=False)
                contextSym.setTypeSpecial(SymbolSpecialType.hardwarereg, isAutogenerated=False)
                contextSym.userDeclaredSize = 4
                contextSym.isDefined = True
                contextSym.isUserDeclared = True
            if vram > highestVram:
                highestVram = vram
            if vram < lowestVram:
                lowestVram = vram
        self.context.totalVramRange.addSpecialRange(lowestVram, highestVram)


    def readVariablesCsv(self, filepath: Path) -> None:
        if not filepath.exists():
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
            varSize = None
            if varSizeStr not in {"", "-"}:
                varSize = int(varSizeStr, 16)
                if varSize == 0:
                    varSize = None
            if varType in {"", "-"}:
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
            contextSym.setTypeSpecial(varType, isAutogenerated=False)
            contextSym.userDeclaredSize = varSize
            contextSym.isUserDeclared = True

    def readFunctionsCsv(self, filepath: Path) -> None:
        if not filepath.exists():
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

    def readConstantsCsv(self, filepath: Path) -> None:
        if not filepath.exists():
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

    def readSplatSymbolAddrs(self, filepath: Path) -> None:
        if not filepath.exists():
            return

        with filepath.open() as f:
            for line in f:
                info, *extra = line.strip().split("//")
                colonSeparatedPairs = "//".join(extra)

                info = info.strip().strip(";")

                if "=" not in info:
                    continue

                name, addressStr = info.split("=")
                address = int(addressStr.strip(), 0)
                name = name.strip()

                pairs = Utils.parseColonSeparatedPairLine(colonSeparatedPairs)

                symSize = Utils.getMaybeIntFromMaybeStr(pairs.get("size"))

                if Utils.getMaybeBooleyFromMaybeStr(pairs.get("ignore")):
                    if symSize is not None and symSize > 0:
                        self.context.addBannedSymbolRangeBySize(address, symSize)
                    else:
                        self.context.addBannedSymbol(address)
                    continue

                symType = pairs.get("type")
                rom = Utils.getMaybeIntFromMaybeStr(pairs.get("rom"))
                if symType == "func":
                    contextSym = self.addFunction(address, isAutogenerated=False, vromAddress=rom)
                elif symType == "jtbl":
                    contextSym = self.addJumpTable(address, isAutogenerated=False, vromAddress=rom)
                elif symType == "jtbl_label":
                    contextSym = self.addJumpTableLabel(address, isAutogenerated=False, vromAddress=rom)
                elif symType == "label":
                    contextSym = self.addBranchLabel(address, isAutogenerated=False, vromAddress=rom)
                else:
                    contextSym = self.addSymbol(address, isAutogenerated=False, vromAddress=rom)
                    if symType is not None:
                        contextSym.setTypeSpecial(symType, isAutogenerated=False)

                contextSym.name = name
                contextSym.isUserDeclared = True
                contextSym.nameEnd = pairs.get("name_end")
                contextSym.userDeclaredSize = Utils.getMaybeIntFromMaybeStr(pairs.get("size"))

                defined = Utils.getMaybeBooleyFromMaybeStr(pairs.get("defined"))
                if defined is not None:
                    contextSym.isDefined = defined

                forceMigration = Utils.getMaybeBooleyFromMaybeStr(pairs.get("force_migration"))
                if forceMigration is not None:
                    contextSym.forceMigration = forceMigration
                forceNotMigration = Utils.getMaybeBooleyFromMaybeStr(pairs.get("force_not_migration"))
                if forceNotMigration is not None:
                    contextSym.forceNotMigration = forceNotMigration
                contextSym.functionOwnerForMigration = pairs.get("function_owner")

                allowAddend = Utils.getMaybeBooleyFromMaybeStr(pairs.get("allow_addend"))
                if allowAddend is not None:
                    contextSym.allowedToReferenceAddends = allowAddend
                dontAllowAddend = Utils.getMaybeBooleyFromMaybeStr(pairs.get("dont_allow_addend"))
                if dontAllowAddend is not None:
                    contextSym.notAllowedToReferenceAddends = dontAllowAddend

                allowReference = Utils.getMaybeBooleyFromMaybeStr(pairs.get("allow_reference"))
                if allowReference is not None:
                    contextSym.allowedToReferenceSymbols = allowReference

                allowBeReferenced = Utils.getMaybeBooleyFromMaybeStr(pairs.get("allow_be_referenced"))
                if allowBeReferenced is not None:
                    contextSym.allowedToBeReferenced = allowBeReferenced

                contextSym.visibility = pairs.get("visibility")

                align = pairs.get("align")
                if align is not None:
                    contextSym.setAlignment(int(align, 0))
