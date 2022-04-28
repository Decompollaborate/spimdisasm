#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig, printQuietless, printVerbose
from ..common.Context import Context
from ..common.FileSplitFormat import FileSectionType, FileSplitEntry

from .MipsSection import Section
from .MipsText import Text
from .MipsData import Data
from .MipsRodata import Rodata
from .MipsBss import Bss
from .MipsFunction import Function


def createSectionFromSplitEntry(splitEntry: FileSplitEntry, array_of_bytes: bytearray, outputPath: str, context: Context) -> Section:
    head, tail = os.path.split(outputPath)

    offsetStart = splitEntry.offset
    offsetEnd = splitEntry.nextOffset

    if offsetStart >= 0 and offsetEnd >= 0:
        printVerbose(f"Parsing offset range [{offsetStart:02X}, {offsetEnd:02X}]")
        array_of_bytes = array_of_bytes[offsetStart:offsetEnd]
    elif offsetEnd >= 0:
        printVerbose(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    elif offsetStart >= 0:
        printVerbose(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    vram = None
    if splitEntry.vram is not None:
        printVerbose(f"Using VRAM {splitEntry.vram:08X}")
        vram = splitEntry.vram

    if splitEntry.section == FileSectionType.Text:
        f = Text(context, vram, tail, array_of_bytes)
    elif splitEntry.section == FileSectionType.Data:
        f = Data(context, vram, tail, array_of_bytes)
    elif splitEntry.section == FileSectionType.Rodata:
        f = Rodata(context, vram, tail, array_of_bytes)
    elif splitEntry.section == FileSectionType.Bss:
        assert isinstance(splitEntry.vram, int)
        f = Bss(context, splitEntry.vram, splitEntry.vram + offsetEnd - offsetStart, tail)
    else:
        eprint("Error! Section not set!")
        exit(-1)

    f.isHandwritten = splitEntry.isHandwritten
    f.isRsp = splitEntry.isRsp

    return f

def analyzeSectionFromSplitEntry(fileSection: Section, splitEntry: FileSplitEntry):
    offsetStart = splitEntry.offset

    printVerbose("Analyzing")
    fileSection.analyze()
    fileSection.setCommentOffset(offsetStart)

    printVerbose()

    fileSection.printAnalyzisResults()

    return fileSection


def writeSection(path: str, fileSection: Section):
    head, tail = os.path.split(path)

    # Create directories
    if head != "":
        os.makedirs(head, exist_ok=True)

    fileSection.saveToFile(path)

    return path


def getRdataAndLateRodataForFunction(func: Function, rodataFileList: List[Tuple[str, Rodata]], context: Context):
    rdataList = []
    lateRodataList = []
    lateRodataLen = 0
    firstRodata = None
    for _, rodataSection in rodataFileList:
        if len(rdataList) > 0 or len(lateRodataList) > 0:
            # We already have the rodata for this function. Stop searching
            break

        # Skip the file if there's nothing in this file refenced by the current function
        intersection = func.referencedVRams & rodataSection.symbolsVRams
        if len(intersection) == 0:
            continue

        sortedSymbolVRams = sorted(rodataSection.symbolsVRams)

        for vram in sorted(intersection):
            nextVramIndex = sortedSymbolVRams.index(vram) + 1
            nextVram = float("inf") if nextVramIndex >= len(sortedSymbolVRams) else sortedSymbolVRams[nextVramIndex]

            rodataSymbol = context.getGenericSymbol(vram, False)
            assert rodataSymbol is not None
            # We only care for rodata that's used once
            if rodataSymbol.referenceCounter != 1:
                break

            isConstVariable = True
            if rodataSymbol.type in ("f32", "f64", "Vec3f"):
                isConstVariable = False
            elif vram in context.jumpTables:
                isConstVariable = False
            elif type == "char" or (GlobalConfig.STRING_GUESSER and rodataSymbol.isMaybeString):
                isConstVariable = False

            # A const variable should not be placed with a function
            if isConstVariable:
                break

            for rodataSym in rodataSection.symbolList:
                # TODO: this can be improved a bit
                assert rodataSym.vram is not None
                if rodataSym.vram < vram:
                    continue
                if rodataSym.vram >= nextVram:
                    break

                if firstRodata is None:
                    firstRodata = rodataSection.vram

                dis = rodataSym.disassemble()
                if rodataSymbol.isLateRodata:
                    lateRodataList.append(dis)
                    lateRodataLen += rodataSym.sizew
                else:
                    rdataList.append(dis)


            if rodataSymbol.isLateRodata:
                lateRodataList.append("\n")
            else:
                rdataList.append("\n")

    return rdataList, lateRodataList, lateRodataLen, firstRodata

def writeSplittedFunctionToFile(f: TextIO, func: Function, rodataFileList: List[Tuple[str, Rodata]], context: Context):
    rdataList, lateRodataList, lateRodataLen, firstRodata = getRdataAndLateRodataForFunction(func, rodataFileList, context)

    if len(rdataList) > 0:
        # Write the rdata
        f.write(".rdata\n")
        for x in rdataList:
            f.write(x)

    if len(lateRodataList) > 0:
        # Write the late_rodata
        f.write(".late_rodata\n")
        if lateRodataLen / len(func.instructions) > 1/3:
            align = 4
            if firstRodata is not None:
                if firstRodata % 8 == 0:
                    align = 8
            f.write(f".late_rodata_alignment {align}\n")
        for x in lateRodataList:
            f.write(x)

    f.write("\n.text\n")

    # Write the function
    f.write(func.disassemble())

def writeSplitedFunction(path: str, func: Function, rodataFileList: List[Tuple[str, Rodata]], context: Context):
    os.makedirs(path, exist_ok=True)
    with open(os.path.join(path, func.name) + ".s", "w") as f:
        writeSplittedFunctionToFile(f, func, rodataFileList, context)


def getOtherRodata(vram: int, nextVram: int, rodataSection: Rodata, context: Context) -> tuple[str|None, list[str]]:
    rdataList: list[str] = []

    rodataSymbol = context.getGenericSymbol(vram, False)
    assert rodataSymbol is not None

    isConstVariable = True
    if rodataSymbol.type in ("f32", "f64", "Vec3f"):
        isConstVariable = False
    elif vram in context.jumpTables:
        isConstVariable = False
    elif type == "char" or (GlobalConfig.STRING_GUESSER and rodataSymbol.isMaybeString):
        isConstVariable = False

    # A const variable should not be placed with a function
    if not isConstVariable:
        if rodataSymbol.referenceCounter == 1:
            #continue
            return None, []

    # print(rodataSymbol.name, rodataSymbol.referenceCounter)

    for rodataSym in rodataSection.symbolList:
        # TODO: this can be improved a bit
        assert rodataSym.vram is not None
        if rodataSym.vram < vram:
            continue
        if rodataSym.vram >= nextVram:
            break

        dis = rodataSym.disassemble()
        rdataList.append(dis)

    return rodataSymbol.name, rdataList

def writeOtherRodata(path: str, rodataFileList: List[Tuple[str, Rodata]], context: Context):
    for _, rodata in rodataFileList:
        rodataPath = os.path.join(path, rodata.name)
        os.makedirs(rodataPath, exist_ok=True)
        sortedSymbolVRams = sorted(rodata.symbolsVRams)

        for vram in sortedSymbolVRams:
            nextVramIndex = sortedSymbolVRams.index(vram) + 1
            nextVram = 0xFFFFFFFF if nextVramIndex >= len(sortedSymbolVRams) else sortedSymbolVRams[nextVramIndex]

            rodataSymbolName, rdataList = getOtherRodata(vram, nextVram, rodata, context)
            if rodataSymbolName is None:
                continue

            rodataSymbolPath = os.path.join(rodataPath, rodataSymbolName) + ".s"

            with open(rodataSymbolPath, "w") as f:
                f.write(".rdata\n")
                for rdata in rdataList:
                    f.write(rdata)
