#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import common
from .. import elf32
from .. import mips


def getArgsParser() -> argparse.ArgumentParser:
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    parser.add_argument("--save-context", help="Saves the context to a file", metavar="FILENAME")

    common.GlobalConfig.addParametersToArgParse(parser)

    mips.InstructionConfig.addParametersToArgParse(parser)

    return parser

def applyArgs(args: argparse.Namespace) -> None:
    mips.InstructionConfig.parseArgs(args)
    common.GlobalConfig.parseArgs(args)

def applyGlobalConfigurations() -> None:
    common.GlobalConfig.REMOVE_POINTERS = False
    common.GlobalConfig.IGNORE_BRANCHES = False
    common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES = False

    common.GlobalConfig.ALLOW_UNKSEGMENT = False


def getOutputPath(inputPath: Path, textOutput: Path, dataOutput: Path, sectionType: common.FileSectionType) -> Path:
    outputPath = dataOutput
    if sectionType == common.FileSectionType.Text:
        outputPath = textOutput

    outputFilePath = outputPath
    if outputPath != "-":
        outputFilePath = outputFilePath / inputPath.stem

    return outputFilePath

def getProcessedSections(context: common.Context, elfFile: elf32.Elf32File, array_of_bytes: bytearray, inputPath: Path, textOutput: Path, dataOutput: Path) -> tuple[dict[common.FileSectionType, mips.sections.SectionBase], dict[common.FileSectionType, Path]]:
    processedSegments: dict[common.FileSectionType, mips.sections.SectionBase] = dict()
    segmentPaths: dict[common.FileSectionType, Path] = dict()

    for sectionType, sectionEntry in elfFile.progbits.items():
        outputFilePath = getOutputPath(inputPath, textOutput, dataOutput, sectionType)

        vromStart = sectionEntry.offset
        vromEnd = vromStart + sectionEntry.size
        vramStart = sectionEntry.addr

        mipsSection: mips.sections.SectionBase
        if sectionType == common.FileSectionType.Text:
            mipsSection = mips.sections.SectionText(context, vromStart, vromEnd, vramStart, inputPath.stem, array_of_bytes, 0, None)
        elif sectionType == common.FileSectionType.Data:
            mipsSection = mips.sections.SectionData(context, vromStart, vromEnd, vramStart, inputPath.stem, array_of_bytes, 0, None)
        elif sectionType == common.FileSectionType.Rodata:
            mipsSection = mips.sections.SectionRodata(context, vromStart, vromEnd, vramStart, inputPath.stem, array_of_bytes, 0, None)
        else:
            common.Utils.eprint(f"Error! Invalid section type '{sectionType}'")
            exit(-1)
        mipsSection.setCommentOffset(vromStart)
        processedSegments[sectionType] = mipsSection
        segmentPaths[sectionType] = outputFilePath

    if elfFile.nobits is not None:
        outputFilePath = getOutputPath(inputPath, textOutput, dataOutput, common.FileSectionType.Bss)

        vromStart = elfFile.nobits.offset
        vromEnd = vromStart + elfFile.nobits.size
        bssStart = elfFile.nobits.addr
        bssEnd = bssStart + elfFile.nobits.size

        mipsSection = mips.sections.SectionBss(context, vromStart, vromEnd, bssStart, bssEnd, inputPath.stem, 0, None)
        mipsSection.setCommentOffset(vromStart)
        processedSegments[common.FileSectionType.Bss] = mipsSection
        segmentPaths[common.FileSectionType.Bss] = outputFilePath

    return processedSegments, segmentPaths

def changeGlobalSegmentRanges(context: common.Context, processedSegments: dict[common.FileSectionType, mips.sections.SectionBase]) -> None:
    lowestVromStart = None
    highestVromEnd = None
    lowestVramStart = None
    highestVramEnd = None

    for subSegment in processedSegments.values():
        if lowestVromStart is None or subSegment.vromStart < lowestVromStart:
            lowestVromStart = subSegment.vromStart
        if highestVromEnd is None or subSegment.vromEnd > highestVromEnd:
            highestVromEnd = subSegment.vromEnd
        if lowestVramStart is None or subSegment.vram < lowestVramStart:
            lowestVramStart = subSegment.vram
        if highestVramEnd is None or subSegment.vramEnd > highestVramEnd:
            highestVramEnd = subSegment.vramEnd

    if lowestVromStart is None:
        lowestVromStart = 0x0
    if highestVromEnd is None:
        highestVromEnd = 0xFFFFFFFF
    if lowestVramStart is None:
        lowestVramStart = 0x0
    if highestVramEnd is None:
        highestVramEnd = 0xFFFFFFFF
    context.globalSegment.changeRanges(lowestVromStart, highestVromEnd, lowestVramStart, highestVramEnd)
    return


def addRelocatedSymbol(context: common.Context, symEntry: elf32.Elf32SymEntry, symName: str|None):
    if symEntry.value == 0:
        return

    if symEntry.stType == elf32.Elf32SymbolTableType.FUNC.value:
        contextSym = context.globalSegment.addFunction(symEntry.value)
    elif symEntry.stType == elf32.Elf32SymbolTableType.OBJECT.value:
        contextSym = context.globalSegment.addSymbol(symEntry.value)
    elif symEntry.stType == elf32.Elf32SymbolTableType.SECTION.value:
        # print(symEntry)
        return
    elif symEntry.stType == elf32.Elf32SymbolTableType.NOTYPE.value:
        # Is ok to just ignore this?
        return
    else:
        common.Utils.eprint(f"Warning: symbol '{symName}' has an unhandled stType: '{symEntry.stType}'")
        contextSym = context.globalSegment.addSymbol(symEntry.value)
    if symName is not None:
        contextSym.name = symName
    contextSym.isUserDeclared = True
    contextSym.setSizeIfUnset(symEntry.size)

def insertSymtabIntoContext(context: common.Context, symbolTable: elf32.Elf32Syms, stringTable: elf32.Elf32StringTable, elfFile: elf32.Elf32File, processedSegments: dict[common.FileSectionType, mips.sections.SectionBase]):
    # Use the symbol table to replace symbol names present in disassembled sections
    for i, symEntry in enumerate(symbolTable):
        symName = stringTable[symEntry.name]

        if symEntry.shndx == 0:
            continue

        sectHeaderEntry = elfFile.sectionHeaders[symEntry.shndx]
        if sectHeaderEntry is None:
            continue

        if elfFile.header.type != elf32.Elf32ObjectFileType.REL.value:
            addRelocatedSymbol(context, symEntry, symName)
            continue

        sectName = elfFile.shstrtab[sectHeaderEntry.name]
        sectType = common.FileSectionType.fromStr(sectName)
        if sectType != common.FileSectionType.Invalid:
            subSegment = processedSegments[sectType]
            symbolOffset = symEntry.value + subSegment.vromStart

            contextOffsetSym = common.ContextOffsetSymbol(symbolOffset, symName, sectType)
            contextOffsetSym.isUserDeclared = True
            context.offsetSymbols[sectType][symbolOffset] = contextOffsetSym
        else:
            common.Utils.eprint(f"Warning: symbol {i} (name: '{symName}', value: 0x{symEntry.value:X}) is referencing invalid section '{sectName}'")

def insertDynsymIntoContext(context: common.Context, symbolTable: elf32.Elf32Syms, stringTable: elf32.Elf32StringTable):
    for symEntry in symbolTable:
        symName = stringTable[symEntry.name]

        addRelocatedSymbol(context, symEntry, symName)


def injectAllElfSymbols(context: common.Context, elfFile: elf32.Elf32File, processedSegments: dict[common.FileSectionType, mips.sections.SectionBase]) -> None:
    if elfFile.symtab is not None and elfFile.strtab is not None:
        # Inject symbols from the reloc table referenced in each section
        if elfFile.header.type == elf32.Elf32ObjectFileType.REL.value:
            for sectType, relocs in elfFile.rel.items():
                # subSection = processedFiles[sectType][1]
                for rel in relocs:
                    symbolEntry = elfFile.symtab[rel.rSym]
                    symbolName = elfFile.strtab[symbolEntry.name]
                    if symbolName == "":
                        continue

                    contextRelocSym = common.ContextRelocSymbol(rel.offset, symbolName, sectType)
                    contextRelocSym.isDefined = True
                    contextRelocSym.relocType = rel.rType
                    context.relocSymbols[sectType][rel.offset] = contextRelocSym

        # Use the symtab to replace symbol names present in disassembled sections
        insertSymtabIntoContext(context, elfFile.symtab, elfFile.strtab, elfFile, processedSegments)

    if elfFile.dynsym is not None and elfFile.dynstr is not None:
        # Use the dynsym to replace symbol names present in disassembled sections
        insertDynsymIntoContext(context, elfFile.dynsym, elfFile.dynstr)
    return

def processGlobalOffsetTable(context: common.Context, elfFile: elf32.Elf32File) -> None:
    if elfFile.reginfo is not None:
        common.GlobalConfig.GP_VALUE = elfFile.reginfo.gpValue

    if elfFile.got is not None:
        context.got.localsTable = elfFile.got.localsTable
        context.got.globalsTable = elfFile.got.globalsTable

        for address in context.got.globalsTable:
            contextSym = context.globalSegment.getSymbol(address)
            if contextSym is not None:
                contextSym.isGotGlobal = True

    if elfFile.dynamic is not None:
        context.got.tableStart = elfFile.dynamic.pltGot
    return


def elfObjDisasmMain():
    args = getArgsParser().parse_args()
    applyArgs(args)

    applyGlobalConfigurations()

    context = common.Context()

    inputPath = Path(args.binary)
    array_of_bytes = common.Utils.readFileAsBytearray(inputPath)
    elfFile = elf32.Elf32File(array_of_bytes)

    if elf32.Elf32HeaderFlag.PIC in elfFile.elfFlags or elf32.Elf32HeaderFlag.CPIC in elfFile.elfFlags:
        common.GlobalConfig.PIC = True

    textOutput = Path(args.output)
    if args.data_output is None:
        dataOutput = textOutput
    else:
        dataOutput = Path(args.data_output)

    processedSegments, segmentPaths = getProcessedSections(context, elfFile, array_of_bytes, inputPath, textOutput, dataOutput)

    changeGlobalSegmentRanges(context, processedSegments)
    injectAllElfSymbols(context, elfFile, processedSegments)
    processGlobalOffsetTable(context, elfFile)

    for subSegment in processedSegments.values():
        subSegment.analyze()

    for sectionType, subSegment in processedSegments.items():
        outputFilePath = segmentPaths[sectionType]
        mips.FilesHandlers.writeSection(outputFilePath, subSegment)

    if args.save_context is not None:
        contextPath = Path(args.save_context)
        contextPath.parent.mkdir(parents=True, exist_ok=True)
        context.saveContextToFile(contextPath)
