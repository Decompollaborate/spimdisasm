#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import dataclasses
from pathlib import Path

from . import Utils
from .ContextSymbols import ContextSymbol
from .SymbolsSegment import SymbolsSegment
from .GlobalOffsetTable import GlobalOffsetTable
from .Relocation import RelocationInfo, RelocType


@dataclasses.dataclass
class SymbolRange:
    start: int
    end: int

    def isInRange(self, address: int) -> bool:
        return self.start <= address < self.end

    def decreaseStart(self, address: int) -> None:
        if address < self.start:
            self.start = address
        return None

    def increaseEnd(self, address: int) -> None:
        if address > self.end:
            self.end = address
        return None


class Context:
    N64DefaultBanned = {
        0x7FFFFFE0, # osInvalICache
        0x7FFFFFF0, # osInvalDCache, osWritebackDCache, osWritebackDCacheAll
        0x7FFFFFFF,
        0x80000010,
        0x80000020,
    }

    def __init__(self):
        # Arbitrary initial range
        self.globalSegment = SymbolsSegment(self, 0x0, 0x1000, 0x80000000, 0x80001000, overlayCategory=None)
        # For symbols that we don't know where they come from
        self.unknownSegment = SymbolsSegment(self, None, None, 0x00000000, 0xFFFFFFFF, overlayCategory=None)
        self._isTheUnknownSegment = True

        self.overlaySegments: dict[str, dict[int, SymbolsSegment]] = dict()
        "Outer key is overlay type, inner key is the vrom of the overlay's segment"

        self.totalVramRange: SymbolRange = SymbolRange(self.globalSegment.vramStart, self.globalSegment.vramEnd)
        self._defaultVramRanges: bool = True

        # Stuff that looks like pointers, but the disassembler shouldn't count it as a pointer
        self.bannedSymbols: set[int] = set()
        self.bannedRangedSymbols: list[SymbolRange] = list()

        self.globalRelocationOverrides: dict[int, RelocationInfo] = dict()
        "key: vrom address"

        self.got: GlobalOffsetTable = GlobalOffsetTable()


    def changeGlobalSegmentRanges(self, vromStart: int, vromEnd: int, vramStart: int, vramEnd: int):
        if vromStart == vromEnd:
            Utils.eprint(f"Warning: globalSegment's will has its vromStart equal to the vromEnd (0x{vromStart:X})")
        if vramStart == vramEnd:
            Utils.eprint(f"Warning: globalSegment's will has its vramStart equal to the vramEnd (0x{vramStart:X})")
        self.globalSegment.changeRanges(vromStart, vromEnd, vramStart, vramEnd)
        if self._defaultVramRanges:
            self.totalVramRange.start = vramStart
            self.totalVramRange.end = vramEnd
            self._defaultVramRanges = False
        self.totalVramRange.decreaseStart(vramStart)
        self.totalVramRange.increaseEnd(vramEnd)

    def addOverlaySegment(self, overlayCategory: str, segmentVromStart: int, segmentVromEnd: int, segmentVramStart: int, segmentVramEnd: int) -> SymbolsSegment:
        if overlayCategory not in self.overlaySegments:
            self.overlaySegments[overlayCategory] = dict()
        segment = SymbolsSegment(self, segmentVromStart, segmentVromEnd, segmentVramStart, segmentVramEnd, overlayCategory=overlayCategory)
        self.overlaySegments[overlayCategory][segmentVromStart] = segment

        if self._defaultVramRanges:
            self.totalVramRange.start = segmentVramStart
            self.totalVramRange.end = segmentVramEnd
            self._defaultVramRanges = False
        self.totalVramRange.decreaseStart(segmentVramStart)
        self.totalVramRange.increaseEnd(segmentVramEnd)

        return segment


    def initGotTable(self, pltGot: int, localsTable: list[int], globalsTable: list[int]):
        self.got.initTables(pltGot, localsTable, globalsTable)

        for gotEntry in self.got.globalsTable:
            gotEntry.contextSym = self.globalSegment.addSymbol(gotEntry.address)
            gotEntry.contextSym.isUserDeclared = True
            gotEntry.contextSym.isGotGlobal = True


    def fillDefaultBannedSymbols(self):
        self.bannedSymbols |= self.N64DefaultBanned


    def isAddressInGlobalRange(self, address: int) -> bool:
        return self.totalVramRange.isInRange(address)


    def addBannedSymbol(self, address: int):
        self.bannedSymbols.add(address)

    def addBannedSymbolRange(self, rangeStart: int, rangeEnd: int):
        self.bannedRangedSymbols.append(SymbolRange(rangeStart, rangeEnd))

    def addBannedSymbolRangeBySize(self, rangeStart: int, size: int):
        self.bannedRangedSymbols.append(SymbolRange(rangeStart, rangeStart + size))

    def isAddressBanned(self, address: int) -> bool:
        if address in self.bannedSymbols:
            return True
        for ranged in self.bannedRangedSymbols:
            if ranged.isInRange(address):
                return True
        return False

    def addGlobalReloc(self, vromAddres: int, relocType: RelocType, symbol: ContextSymbol|str, addend: int=0) -> RelocationInfo:
        reloc = RelocationInfo(relocType, symbol, addend)
        self.globalRelocationOverrides[vromAddres] = reloc
        return reloc

    def saveContextToFile(self, contextPath: Path):
        with contextPath.open("w") as f:
            self.globalSegment.saveContextToFile(f)

        # unknownPath = contextPath.with_stem(f"{contextPath.stem}_unksegment")
        unknownPath = contextPath.with_name(f"{contextPath.stem}_unksegment" + contextPath.suffix)
        with unknownPath.open("w") as f:
            self.unknownSegment.saveContextToFile(f)

        for overlayCategory, segmentsPerVrom in self.overlaySegments.items():
            for segmentVrom, overlaySegment in segmentsPerVrom.items():

                # ovlPath = contextPath.with_stem(f"{contextPath.stem}_{overlayCategory}_{segmentVrom:06X}")
                ovlPath = contextPath.with_name(f"{contextPath.stem}_{overlayCategory}_{segmentVrom:06X}" + contextPath.suffix)
                with ovlPath.open("w") as f:
                    overlaySegment.saveContextToFile(f)


    @staticmethod
    def addParametersToArgParse(parser: argparse.ArgumentParser):
        contextParser = parser.add_argument_group("Context configuration")

        contextParser.add_argument("--save-context", help="Saves the context to a file", metavar="FILENAME")


        csvConfig = parser.add_argument_group("Context .csv input files")

        csvConfig.add_argument("--functions", help="Path to a functions csv", action="append")
        csvConfig.add_argument("--variables", help="Path to a variables csv", action="append")
        csvConfig.add_argument("--constants", help="Path to a constants csv", action="append")
        csvConfig.add_argument("--symbol-addrs", help="Path to a splat-compatible symbol_addrs.txt file", action="append")


        symbolsConfig = parser.add_argument_group("Context default symbols configuration")

        symbolsConfig.add_argument("--default-banned", help="Toggles filling the list of default banned symbols. Defaults to True", action=Utils.BooleanOptionalAction)
        symbolsConfig.add_argument("--libultra-syms", help="Toggles using the built-in libultra symbols. Defaults to True", action=Utils.BooleanOptionalAction)
        symbolsConfig.add_argument("--hardware-regs", help="Toggles using the built-in hardware registers symbols. Defaults to True", action=Utils.BooleanOptionalAction)
        symbolsConfig.add_argument("--named-hardware-regs", help="Use actual names for the hardware registers", action=Utils.BooleanOptionalAction)


    def parseArgs(self, args: argparse.Namespace):
        if args.default_banned != False:
            self.fillDefaultBannedSymbols()
        if args.libultra_syms != False:
            self.globalSegment.fillLibultraSymbols()
        if args.hardware_regs != False:
            self.globalSegment.fillHardwareRegs(args.named_hardware_regs)

        if args.functions is not None:
            for funcsPath in args.functions:
                self.globalSegment.readFunctionsCsv(Path(funcsPath))
        if args.variables is not None:
            for varsPath in args.variables:
                self.globalSegment.readVariablesCsv(Path(varsPath))
        if args.constants is not None:
            for constantsPath in args.constants:
                self.globalSegment.readConstantsCsv(Path(constantsPath))
        if args.symbol_addrs is not None:
            for filepath in args.symbol_addrs:
                self.globalSegment.readSplatSymbolAddrs(Path(filepath))
