#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from typing import TextIO

import dataclasses

from .. import common

from . import sections
from . import symbols


@dataclasses.dataclass
class FunctionRodataEntry:
    function: symbols.SymbolFunction | None = None
    rodataSyms: list[symbols.SymbolBase] = dataclasses.field(default_factory=list)
    lateRodataSyms: list[symbols.SymbolBase] = dataclasses.field(default_factory=list)

    def hasRodataSyms(self) -> bool:
        return len(self.rodataSyms) > 0 or len(self.lateRodataSyms) > 0

    def writeToFile(self, f: TextIO, writeFunction: bool=True):
        if len(self.rodataSyms) > 0:
            # Write the rdata
            f.write(f".section .rodata{common.GlobalConfig.LINE_ENDS}")
            for sym in self.rodataSyms:
                f.write(sym.disassemble(migrate=True, useGlobalLabel=True))
                f.write(common.GlobalConfig.LINE_ENDS)

        if len(self.lateRodataSyms) > 0:
            assert self.function is not None
            # Write the late_rodata
            f.write(f".section .late_rodata{common.GlobalConfig.LINE_ENDS}")

            lateRodataSize = 0
            for sym in self.lateRodataSyms:
                lateRodataSize += sym.sizew

            if lateRodataSize / len(self.function.instructions) > 1/3:
                align = 4
                firstLateRodataVram = self.lateRodataSyms[0].vram
                if firstLateRodataVram is not None and firstLateRodataVram % 8 == 0:
                    align = 8
                f.write(f".late_rodata_alignment {align}{common.GlobalConfig.LINE_ENDS}")
            for sym in self.lateRodataSyms:
                f.write(sym.disassemble(migrate=True, useGlobalLabel=True))
                f.write(common.GlobalConfig.LINE_ENDS)

        if len(self.rodataSyms) > 0 or len(self.lateRodataSyms) > 0:
            f.write(f"{common.GlobalConfig.LINE_ENDS}.section .text{common.GlobalConfig.LINE_ENDS}")

        if writeFunction and self.function is not None:
            # Write the function itself
            f.write(self.function.disassemble(migrate=True))

    @staticmethod
    def getEntryFromSection(func: symbols.SymbolFunction, rodataSection: sections.SectionRodata) -> FunctionRodataEntry:
        rodataList: list[symbols.SymbolBase] = []
        lateRodataList: list[symbols.SymbolBase] = []

        intersection = func.instrAnalyzer.referencedVrams & rodataSection.symbolsVRams
        if len(intersection) == 0:
            return FunctionRodataEntry(func)

        for rodataSym in rodataSection.symbolList:
            if rodataSym.vram not in intersection:
                continue

            if not rodataSym.shouldMigrate():
                continue

            if rodataSym.contextSym.isLateRodata():
                lateRodataList.append(rodataSym)
            else:
                rodataList.append(rodataSym)

        return FunctionRodataEntry(func, rodataList, lateRodataList)

    @staticmethod
    def getEntryFromPossibleRodataSections(func: symbols.SymbolFunction, rodataFileList: list[sections.SectionBase]) -> FunctionRodataEntry:
        for rodataSection in rodataFileList:
            assert isinstance(rodataSection, sections.SectionRodata)

            # Skip the file if there's nothing in this file refenced by the current function
            intersection = func.instrAnalyzer.referencedVrams & rodataSection.symbolsVRams
            if len(intersection) == 0:
                continue

            entry = FunctionRodataEntry.getEntryFromSection(func, rodataSection)
            if entry.hasRodataSyms():
                return entry

        return FunctionRodataEntry(func)

    @staticmethod
    def getAllEntriesFromSections(textSection: sections.SectionText, rodataSection: sections.SectionRodata) -> list[FunctionRodataEntry]:
        allUnmigratedRodataSymbols: list[symbols.SymbolBase] = []

        for rodataSym in rodataSection.symbolList:
            if not rodataSym.shouldMigrate():
                # We only care for the symbols which will not be migrated
                allUnmigratedRodataSymbols.append(rodataSym)

        allEntries: list[FunctionRodataEntry] = []

        for func in textSection.symbolList:
            assert isinstance(func, symbols.SymbolFunction)

            entry = FunctionRodataEntry.getEntryFromSection(func, rodataSection)

            if len(entry.rodataSyms) > 0:
                firstFuncRodataSym = entry.rodataSyms[0]

                while len(allUnmigratedRodataSymbols) > 0:
                    rodataSym = allUnmigratedRodataSymbols[0]

                    if rodataSym.vram >= firstFuncRodataSym.vram:
                        # Take all the symbols up to the first rodata sym referenced by the current function
                        break

                    allEntries.append(FunctionRodataEntry(rodataSyms=[rodataSym]))
                    del allUnmigratedRodataSymbols[0]

            allEntries.append(entry)

        # Check if there's any rodata symbol remaining and add it to the list
        for rodataSym in allUnmigratedRodataSymbols:
            allEntries.append(FunctionRodataEntry(rodataSyms=[rodataSym]))

        return allEntries
