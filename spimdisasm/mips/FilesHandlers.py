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

        # We only care for rodata that's used once
        if rodataSym.contextSym.referenceCounter != 1:
            if common.GlobalConfig.COMPILER == common.Compiler.IDO:
                continue
            elif len(rodataSym.contextSym.referenceFunctions) != 1:
                continue

        # A const variable should not be placed with a function
        if rodataSym.contextSym.isMaybeConstVariable():
            if common.GlobalConfig.COMPILER != common.Compiler.SN64:
                continue

        if rodataSym.contextSym.isLateRodata() and common.GlobalConfig.COMPILER == common.Compiler.IDO:
            lateRodataList.append(rodataSym)
            lateRodataSize += rodataSym.sizew
        else:
            rdataList.append(rodataSym)

    return rdataList, lateRodataList, lateRodataSize

def getRdataAndLateRodataForFunction(func: symbols.SymbolFunction, rodataFileList: list[sections.SectionRodata]) -> tuple[list[symbols.SymbolBase], list[symbols.SymbolBase], int]:
    rdataList: list[symbols.SymbolBase] = []
    lateRodataList: list[symbols.SymbolBase] = []
    lateRodataSize = 0

    for rodataSection in rodataFileList:
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
            f.write(sym.disassemble())
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
            f.write(sym.disassemble())
            f.write(common.GlobalConfig.LINE_ENDS)

    if len(rdataList) > 0 or len(lateRodataList) > 0:
        f.write(common.GlobalConfig.LINE_ENDS + ".section .text" + common.GlobalConfig.LINE_ENDS)

def writeSplitedFunction(path: Path, func: symbols.SymbolFunction, rodataFileList: list[sections.SectionRodata]):
    path.mkdir(parents=True, exist_ok=True)

    funcPath = path / (func.getName()+ ".s")
    with funcPath.open("w") as f:
        rdataList, lateRodataList, lateRodataSize = getRdataAndLateRodataForFunction(func, rodataFileList)
        writeFunctionRodataToFile(f, func, rdataList, lateRodataList, lateRodataSize)

        # Write the function itself
        f.write(func.disassemble())

def writeOtherRodata(path: Path, rodataFileList: list[sections.SectionRodata]):
    for rodataSection in rodataFileList:
        rodataPath = path / rodataSection.name
        rodataPath.mkdir(parents=True, exist_ok=True)

        for rodataSym in rodataSection.symbolList:
            if not rodataSym.isRdata():
                continue

            rodataSymbolPath = rodataPath / (rodataSym.getName() + ".s")
            with rodataSymbolPath.open("w") as f:
                f.write(".section .rdata" + common.GlobalConfig.LINE_ENDS)
                f.write(rodataSym.disassemble())
