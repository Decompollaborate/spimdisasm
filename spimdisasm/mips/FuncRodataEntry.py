#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2023-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from typing import TextIO

import dataclasses

from .. import common

from . import sections
from . import symbols


@dataclasses.dataclass
class FunctionRodataEntry:
    """
    Class to facilitate rodata migration.

    The algorithm aims to pair zero or more migrable rodata symbols from a
    rodata section to a function.

    This class assumes at least one of the following is true for any given
    instance:
    - `FunctionRodataEntry.function` is not `None`.
    - `FunctionRodataEntry.rodataSyms` has at least one element.

    If `FunctionRodataEntry.lateRodataSyms` has elements then
    `FunctionRodataEntry.function` is not `None`.

    The recommended way to instance this class is by using
    `FunctionRodataEntry.getAllEntriesFromSections`.
    """

    function: symbols.SymbolFunction | None = None
    rodataSyms: list[symbols.SymbolBase] = dataclasses.field(default_factory=list)
    lateRodataSyms: list[symbols.SymbolBase] = dataclasses.field(default_factory=list)
    sectionText: str = ".text"
    sectionRodata: str = ".rodata"
    sectionLateRodata: str = ".late_rodata"

    def hasRodataSyms(self) -> bool:
        return len(self.rodataSyms) > 0 or len(self.lateRodataSyms) > 0

    def writeToFile(self, f: TextIO, writeFunction: bool=True) -> None:
        if len(self.rodataSyms) > 0:
            # Write the rdata
            f.write(f".section {self.sectionRodata}{common.GlobalConfig.LINE_ENDS}")
            for sym in self.rodataSyms:
                f.write(sym.disassemble(migrate=True, useGlobalLabel=True, isSplittedSymbol=True))
                f.write(common.GlobalConfig.LINE_ENDS)

        if len(self.lateRodataSyms) > 0:
            assert self.function is not None
            # Write the late_rodata
            f.write(f".section {self.sectionLateRodata}{common.GlobalConfig.LINE_ENDS}")

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
                f.write(sym.disassemble(migrate=True, useGlobalLabel=True, isSplittedSymbol=True))
                f.write(common.GlobalConfig.LINE_ENDS)

        if self.function is not None:
            if len(self.rodataSyms) > 0 or len(self.lateRodataSyms) > 0:
                f.write(f"{common.GlobalConfig.LINE_ENDS}.section {self.sectionText}{common.GlobalConfig.LINE_ENDS}")

            if writeFunction:
                # Write the function itself
                f.write(self.function.disassemble(migrate=self.hasRodataSyms(), isSplittedSymbol=True))

    def getName(self) -> str:
        assert self.function is not None or self.hasRodataSyms()

        if self.function is not None:
            return self.function.getName()

        rodataSymsLen = len(self.rodataSyms)
        if rodataSymsLen > 0:
            assert rodataSymsLen == 1, rodataSymsLen
            return self.rodataSyms[0].getName()

        lateRodataSyms = len(self.lateRodataSyms)
        assert lateRodataSyms == 1, lateRodataSyms
        return self.lateRodataSyms[0].getName()

    @staticmethod
    def getEntryForFuncFromSection(func: symbols.SymbolFunction, rodataSection: sections.SectionRodata|None) -> FunctionRodataEntry:
        """
        Pairs the given function to the migrable rodata symbols of the given
        rodata section.

        If `rodataSection` is `None` or if the function does not reference any
        of the symbols from the given `rodataSection` then a
        `FunctionRodataEntry` containing the given function.
        """

        if rodataSection is None:
            return FunctionRodataEntry(func)

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
    def getEntryForFuncFromPossibleRodataSections(func: symbols.SymbolFunction, rodataFileList: list[sections.SectionBase]) -> FunctionRodataEntry:
        """
        Searches and try to pair migrable rodata symbols to the given function.

        If no migrable rodata symbols are found then a `FunctionRodataEntry`
        containing only the passed function will be returned.
        """

        for rodataSection in rodataFileList:
            assert isinstance(rodataSection, sections.SectionRodata)

            entry = FunctionRodataEntry.getEntryForFuncFromSection(func, rodataSection)
            if entry.hasRodataSyms():
                return entry

        return FunctionRodataEntry(func)

    @staticmethod
    def getAllEntriesFromSections(textSection: sections.SectionText|None, rodataSection: sections.SectionRodata|None) -> list[FunctionRodataEntry]:
        """
        Return all functions to rodata symbols pairings from the given
        sections.

        The returned list preserves original ordering of both the text and
        rodata sections.

        Either a function or a rodata symbol may be orphaned. This means the
        said symbol is paired to zero symbols of the other section. In these
        cases an entry containing the specific symbol is created and inserted
        in corresponding place of the list so it preserves the ordering of both
        sections.
        """

        allUnmigratedRodataSymbols: list[symbols.SymbolBase] = []

        rodataSymbols = rodataSection.symbolList if rodataSection is not None else []
        for rodataSym in rodataSymbols:
            if not rodataSym.shouldMigrate():
                # We only care for the symbols which will not be migrated
                allUnmigratedRodataSymbols.append(rodataSym)

        allEntries: list[FunctionRodataEntry] = []

        textSymbols = textSection.symbolList if textSection is not None else []
        for func in textSymbols:
            assert isinstance(func, symbols.SymbolFunction)

            entry = FunctionRodataEntry.getEntryForFuncFromSection(func, rodataSection)

            # Preserve the order of rodata symbols
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
