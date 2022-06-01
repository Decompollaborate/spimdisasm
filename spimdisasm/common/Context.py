#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .FileSectionType import FileSectionType
from .ContextSymbols import SymbolSpecialType, ContextOffsetSymbol, ContextRelocSymbol
from .SymbolsSegment import SymbolsSegment


class Context:
    N64DefaultBanned = {0x80000010, 0x80000020}

    def __init__(self):
        # Arbitrary initial range
        self.globalSegment = SymbolsSegment(0x0, 0x1000, 0x80000000, 0x80001000, overlayCategory=None)
        # For symbols that we don't know where they come from
        self.unknownSegment = SymbolsSegment(None, None, 0x00000000, 0xFFFFFFFF, overlayCategory=None)

        self.overlaySegments: dict[str, dict[int, SymbolsSegment]] = dict()
        "Outer key is overlay type, inner key is the vrom of the overlay's segment"

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


    def addOverlaySegment(self, overlayCategory: str, segmentVromStart: int, segmentVromEnd: int, segmentVramStart: int, segmentVramEnd: int) -> None:
        if overlayCategory not in self.overlaySegments:
            self.overlaySegments[overlayCategory] = dict()
        self.overlaySegments[overlayCategory][segmentVromStart] = SymbolsSegment(segmentVromStart, segmentVromEnd, segmentVramStart, segmentVramEnd, overlayCategory=overlayCategory)


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


    def saveContextToFile(self, contextPath: Path):
        with contextPath.open("w") as f:
            self.globalSegment.saveContextToFile(f)

        unknownPath = contextPath.with_stem(f"{contextPath.stem}_unksegment")
        with unknownPath.open("w") as f:
            self.unknownSegment.saveContextToFile(f)

        for overlayCategory, segmentsPerVrom in self.overlaySegments.items():
            for segmentVrom, overlaySegment in segmentsPerVrom.items():

                ovlPath = contextPath.with_stem(f"{contextPath.stem}_{overlayCategory}_{segmentVrom:06X}")
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


        symbolsConfig = parser.add_argument_group("Context default symbols configuration")

        symbolsConfig.add_argument("--default-banned", help="Toggles filling the list of default banned symbols. Defaults to True", action=argparse.BooleanOptionalAction)
        symbolsConfig.add_argument("--libultra-syms", help="Toggles using the built-in libultra symbols. Defaults to True", action=argparse.BooleanOptionalAction)
        symbolsConfig.add_argument("--hardware-regs", help="Toggles using the built-in hardware registers symbols. Defaults to True", action=argparse.BooleanOptionalAction)
        symbolsConfig.add_argument("--named-hardware-regs", help="Use actual names for the hardware registers", action=argparse.BooleanOptionalAction)


    def parseArgs(self, args: argparse.Namespace):
        if args.default_banned != False:
            self.fillDefaultBannedSymbols()
        if args.libultra_syms != False:
            self.globalSegment.fillLibultraSymbols()
        if not args.hardware_regs != False:
            self.globalSegment.fillHardwareRegs(args.named_hardware_regs)

        if args.functions is not None:
            for funcsPath in args.functions:
                self.globalSegment.readFunctionsCsv(funcsPath)
        if args.variables is not None:
            for varsPath in args.variables:
                self.globalSegment.readVariablesCsv(varsPath)
        if args.constants is not None:
            for constantsPath in args.constants:
                self.globalSegment.readConstantsCsv(constantsPath)
