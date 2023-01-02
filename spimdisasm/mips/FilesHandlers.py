#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from typing import TextIO
from pathlib import Path

import rabbitizer

from .. import common

from . import sections
from . import symbols


def createSectionFromSplitEntry(splitEntry: common.FileSplitEntry, array_of_bytes: bytearray, outputPath: Path, context: common.Context) -> sections.SectionBase:
    offsetStart = splitEntry.offset
    offsetEnd = splitEntry.nextOffset

    if offsetEnd == 0xFFFFFF:
        offsetEnd = len(array_of_bytes)

    if offsetStart >= 0 and offsetEnd >= 0:
        common.Utils.printVerbose(f"Parsing offset range [{offsetStart:02X}, {offsetEnd:02X}]")
    elif offsetEnd >= 0:
        common.Utils.printVerbose(f"Parsing until offset 0x{offsetEnd:02X}")
    elif offsetStart >= 0:
        common.Utils.printVerbose(f"Parsing since offset 0x{offsetStart:02X}")

    common.Utils.printVerbose(f"Using VRAM {splitEntry.vram:08X}")
    vram = splitEntry.vram

    f: sections.SectionBase
    if splitEntry.section == common.FileSectionType.Text:
        f = sections.SectionText(context, offsetStart, offsetEnd, vram, outputPath.stem, array_of_bytes, 0, None)
        if splitEntry.isRsp:
            f.instrCat = rabbitizer.InstrCategory.RSP
    elif splitEntry.section == common.FileSectionType.Data:
        f = sections.SectionData(context, offsetStart, offsetEnd, vram, outputPath.stem, array_of_bytes, 0, None)
    elif splitEntry.section == common.FileSectionType.Rodata:
        f = sections.SectionRodata(context, offsetStart, offsetEnd, vram, outputPath.stem, array_of_bytes, 0, None)
    elif splitEntry.section == common.FileSectionType.Bss:
        f = sections.SectionBss(context, offsetStart, offsetEnd, splitEntry.vram, splitEntry.vram + offsetEnd - offsetStart, outputPath.stem, 0, None)
    else:
        common.Utils.eprint("Error! Section not set!")
        exit(-1)

    f.isHandwritten = splitEntry.isHandwritten

    return f

def writeSection(path: Path, fileSection: sections.SectionBase):
    path.parent.mkdir(parents=True, exist_ok=True)
    fileSection.saveToFile(str(path))
    return path


def getRdataAndLateRodataForFunctionFromSection(func: symbols.SymbolFunction, rodataSection: sections.SectionRodata) -> tuple[list[symbols.SymbolBase], list[symbols.SymbolBase], int]:
    rdataList: list[symbols.SymbolBase] = []
    lateRodataList: list[symbols.SymbolBase] = []
    lateRodataSize = 0

    intersection = func.instrAnalyzer.referencedVrams & rodataSection.symbolsVRams
    for rodataSym in rodataSection.symbolList:
        if rodataSym.vram not in intersection:
            continue

        if not rodataSym.shouldMigrate():
            continue

        if rodataSym.contextSym.isLateRodata():
            lateRodataList.append(rodataSym)
            lateRodataSize += rodataSym.sizew
        else:
            rdataList.append(rodataSym)

    return rdataList, lateRodataList, lateRodataSize

def getRdataAndLateRodataForFunction(func: symbols.SymbolFunction, rodataFileList: list[sections.SectionBase]) -> tuple[list[symbols.SymbolBase], list[symbols.SymbolBase], int]:
    rdataList: list[symbols.SymbolBase] = []
    lateRodataList: list[symbols.SymbolBase] = []
    lateRodataSize = 0

    for rodataSection in rodataFileList:
        assert isinstance(rodataSection, sections.SectionRodata)

        if len(rdataList) > 0 or len(lateRodataList) > 0:
            # We already have the rodata for this function. Stop searching
            break

        # Skip the file if there's nothing in this file refenced by the current function
        intersection = func.instrAnalyzer.referencedVrams & rodataSection.symbolsVRams
        if len(intersection) == 0:
            continue

        rdataList, lateRodataList, lateRodataSize = getRdataAndLateRodataForFunctionFromSection(func, rodataSection)

    return rdataList, lateRodataList, lateRodataSize

def writeFunctionRodataToFile(f: TextIO, func: symbols.SymbolFunction, rdataList: list[symbols.SymbolBase], lateRodataList: list[symbols.SymbolBase], lateRodataSize: int):
    if len(rdataList) > 0:
        # Write the rdata
        sectionName = ".rodata"
        f.write(f".section {sectionName}" + common.GlobalConfig.LINE_ENDS)
        for sym in rdataList:
            f.write(sym.disassemble(migrate=True, useGlobalLabel=True))
            f.write(common.GlobalConfig.LINE_ENDS)

    if len(lateRodataList) > 0:
        # Write the late_rodata
        f.write(".section .late_rodata" + common.GlobalConfig.LINE_ENDS)
        if lateRodataSize / len(func.instructions) > 1/3:
            align = 4
            firstLateRodataVram = lateRodataList[0].vram
            if firstLateRodataVram is not None and firstLateRodataVram % 8 == 0:
                align = 8
            f.write(f".late_rodata_alignment {align}" + common.GlobalConfig.LINE_ENDS)
        for sym in lateRodataList:
            f.write(sym.disassemble(migrate=True, useGlobalLabel=True))
            f.write(common.GlobalConfig.LINE_ENDS)

    if len(rdataList) > 0 or len(lateRodataList) > 0:
        f.write(common.GlobalConfig.LINE_ENDS + ".section .text" + common.GlobalConfig.LINE_ENDS)

def writeSplitedFunction(path: Path, func: symbols.SymbolFunction, rodataFileList: list[sections.SectionBase]):
    path.mkdir(parents=True, exist_ok=True)

    funcPath = path / (func.getName()+ ".s")
    with funcPath.open("w") as f:
        rdataList, lateRodataList, lateRodataSize = getRdataAndLateRodataForFunction(func, rodataFileList)
        writeFunctionRodataToFile(f, func, rdataList, lateRodataList, lateRodataSize)

        # Write the function itself
        f.write(func.disassemble(migrate=True))

def writeOtherRodata(path: Path, rodataFileList: list[sections.SectionBase]):
    for rodataSection in rodataFileList:
        assert isinstance(rodataSection, sections.SectionRodata)

        rodataPath = path / rodataSection.name
        rodataPath.mkdir(parents=True, exist_ok=True)

        for rodataSym in rodataSection.symbolList:
            if rodataSym.shouldMigrate():
                continue

            rodataSymbolPath = rodataPath / (rodataSym.getName() + ".s")
            with rodataSymbolPath.open("w") as f:
                f.write(".section .rodata" + common.GlobalConfig.LINE_ENDS)
                f.write(rodataSym.disassemble(migrate=True))


def writeMigratedFunctionsList(processedSegments: dict[common.FileSectionType, list[sections.SectionBase]], functionMigrationPath: Path, name: str) -> None:
    funcAndRodataOrderPath = functionMigrationPath / f"{name}_migrated_functions.txt"

    rodataSymbols: list[tuple[symbols.SymbolBase, symbols.SymbolFunction|None]] = []
    for section in processedSegments.get(common.FileSectionType.Rodata, []):
        for sym in section.symbolList:
            rodataSymbols.append((sym, None))
    rodataSymbolsVrams = {sym.vram for sym, _ in rodataSymbols}

    funcs: list[symbols.SymbolFunction] = []
    for section in processedSegments.get(common.FileSectionType.Text, []):
        for func in section.symbolList:
            assert isinstance(func, symbols.SymbolFunction)
            funcs.append(func)

            referencedRodata = rodataSymbolsVrams & func.instrAnalyzer.referencedVrams
            for i in range(len(rodataSymbols)):
                if len(referencedRodata) == 0:
                    break

                rodataSym, funcReferencingThisSym = rodataSymbols[i]

                if rodataSym.vram not in referencedRodata:
                    continue

                referencedRodata.remove(rodataSym.vram)

                if funcReferencingThisSym is not None:
                    # This rodata sym already has a corresponding function associated
                    continue

                rodataSymbols[i] = (rodataSym, func)

    resultingList: list[symbols.SymbolBase] = []
    alreadyAddedFuncs: set[symbols.SymbolFunction] = set()

    lastFunc = None
    for rodataSym, funcReferencingThisSym in rodataSymbols:
        if funcReferencingThisSym is None:
            resultingList.append(rodataSym)
        elif funcReferencingThisSym not in alreadyAddedFuncs:
            alreadyAddedFuncs.add(funcReferencingThisSym)
            lastFunc = funcReferencingThisSym

            for func in funcs:
                if func.vram >= funcReferencingThisSym.vram:
                    break
                if func in alreadyAddedFuncs:
                    continue

                alreadyAddedFuncs.add(func)
                resultingList.append(func)
            resultingList.append(funcReferencingThisSym)

    if lastFunc is None:
        for func in funcs:
            resultingList.append(func)
    else:
        for func in funcs:
            if func.vram <= lastFunc.vram:
                continue
            resultingList.append(func)

    with funcAndRodataOrderPath.open("w") as f:
        for sym in resultingList:
            f.write(sym.getName() + "\n")
