#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import common
from .. import elf32
from .. import mips
from .. import frontendCommon as fec

from .. import __version__

PROGNAME = "elfObjDisasm"


def getArgsParser() -> argparse.ArgumentParser:
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    parser.add_argument("--file-splits", help="Path to a file splits csv")

    parser.add_argument("--split-functions", help="Enables the function and rodata splitter. Expects a path to place the splited functions", metavar="PATH")


    readelfOptions = parser.add_argument_group("readelf-like flags")

    readelfOptions.add_argument("-a", "--all", help="Equivalent to --file-header --syms --relocs --display-got", action="store_true")
    readelfOptions.add_argument("--file-header", help="Display the ELF file header", action="store_true")
    readelfOptions.add_argument("-s", "--syms", help="Display the symbol table", action="store_true")
    readelfOptions.add_argument("-r", "--relocs", help="Display the relocations (if present)", action="store_true")
    readelfOptions.add_argument("--display-got", help="Shows Global offset table information", action="store_true")

    readelfOptions.add_argument("--readelf-only", help="Exit after processing the readelf-like flags, without performing any disassembly", action="store_true")


    common.Context.addParametersToArgParse(parser)

    common.GlobalConfig.addParametersToArgParse(parser)

    mips.InstructionConfig.addParametersToArgParse(parser)

    return parser

def applyArgs(args: argparse.Namespace) -> None:
    if args.libultra_syms is None:
        args.libultra_syms = False
    if args.hardware_regs is None:
        args.hardware_regs = False
    mips.InstructionConfig.parseArgs(args)
    common.GlobalConfig.parseArgs(args)

def applyGlobalConfigurations() -> None:
    common.GlobalConfig.REMOVE_POINTERS = False
    common.GlobalConfig.IGNORE_BRANCHES = False
    common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES = False

    common.GlobalConfig.ALLOW_UNKSEGMENT = False


def applyReadelfLikeFlags(elfFile: elf32.Elf32File, args: argparse.Namespace) -> None:
    if args.all:
        elfFile.readelf_fileHeader()
        elfFile.readelf_syms()
        elfFile.readelf_relocs()
        elfFile.readelf_displayGot()
    else:
        if args.file_header:
            elfFile.readelf_fileHeader()

        if args.syms:
            elfFile.readelf_syms()

        if args.relocs:
            elfFile.readelf_relocs()

        if args.display_got:
            elfFile.readelf_displayGot()

    if args.readelf_only:
        exit(0)


def getOutputPath(inputPath: Path, textOutput: Path, dataOutput: Path, sectionType: common.FileSectionType) -> Path:
    outputPath = dataOutput
    if sectionType == common.FileSectionType.Text:
        outputPath = textOutput

    outputFilePath = outputPath
    if str(outputPath) != "-":
        outputFilePath = outputFilePath / inputPath.stem

    return outputFilePath

def getProcessedSections(context: common.Context, elfFile: elf32.Elf32File, array_of_bytes: bytearray, inputPath: Path, textOutput: Path, dataOutput: Path) -> tuple[dict[common.FileSectionType, list[mips.sections.SectionBase]], dict[common.FileSectionType, list[Path]]]:
    processedSegments: dict[common.FileSectionType, list[mips.sections.SectionBase]] = dict()
    segmentPaths: dict[common.FileSectionType, list[Path]] = dict()

    for sectionType, sectionEntry in elfFile.progbits.items():
        outputFilePath = getOutputPath(inputPath, textOutput, dataOutput, sectionType)

        vromStart = sectionEntry.offset
        vromEnd = vromStart + sectionEntry.size
        vramStart = sectionEntry.addr
        if sectionEntry.addr == 0:
            vramStart = sectionEntry.offset

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
        processedSegments[sectionType] = [mipsSection]
        segmentPaths[sectionType] = [outputFilePath]

    if elfFile.nobits is not None:
        outputFilePath = getOutputPath(inputPath, textOutput, dataOutput, common.FileSectionType.Bss)

        vromStart = elfFile.nobits.offset
        vromEnd = vromStart + elfFile.nobits.size
        bssStart = elfFile.nobits.addr
        if elfFile.nobits.addr == 0:
            bssStart = elfFile.nobits.offset
        bssEnd = bssStart + elfFile.nobits.size

        mipsSection = mips.sections.SectionBss(context, vromStart, vromEnd, bssStart, bssEnd, inputPath.stem, 0, None)
        mipsSection.setCommentOffset(vromStart)
        processedSegments[common.FileSectionType.Bss] = [mipsSection]
        segmentPaths[common.FileSectionType.Bss] = [outputFilePath]

    return processedSegments, segmentPaths

def changeGlobalSegmentRanges(context: common.Context, processedSegments: dict[common.FileSectionType, list[mips.sections.SectionBase]]) -> None:
    lowestVromStart = None
    highestVromEnd = None
    lowestVramStart = None
    highestVramEnd = None

    for subSegment in processedSegments.values():
        for section in subSegment:
            if lowestVromStart is None or section.vromStart < lowestVromStart:
                lowestVromStart = section.vromStart
            if highestVromEnd is None or section.vromEnd > highestVromEnd:
                highestVromEnd = section.vromEnd
            if lowestVramStart is None or section.vram < lowestVramStart:
                lowestVramStart = section.vram
            if highestVramEnd is None or section.vramEnd > highestVramEnd:
                highestVramEnd = section.vramEnd

    if lowestVromStart is None:
        lowestVromStart = 0x0
    if highestVromEnd is None:
        highestVromEnd = 0xFFFFFFFF
    if lowestVramStart is None:
        lowestVramStart = 0x0
    if highestVramEnd is None:
        highestVramEnd = 0xFFFFFFFF
    context.changeGlobalSegmentRanges(lowestVromStart, highestVromEnd, lowestVramStart, highestVramEnd)
    return


def addContextSymFromSymEntry(context: common.Context, symEntry: elf32.Elf32SymEntry, symAddress: int, symName: str|None=None, symVrom: int|None=None) -> common.ContextSymbol|None:
    if symAddress == 0:
        return None

    segment = context.globalSegment
    if not segment.isVramInRange(symAddress):
        segment = context.unknownSegment

    if symEntry.stType == elf32.Elf32SymbolTableType.FUNC.value:
        segment = context.globalSegment
        contextSym = segment.addFunction(symAddress, vromAddress=symVrom)
    elif symEntry.stType == elf32.Elf32SymbolTableType.OBJECT.value:
        contextSym = segment.addSymbol(symAddress, vromAddress=symVrom)
    elif symEntry.stType == elf32.Elf32SymbolTableType.SECTION.value:
        # print(symEntry)
        return None
    elif symEntry.stType == elf32.Elf32SymbolTableType.NOTYPE.value and symEntry.shndx == elf32.Elf32SectionHeaderNumber.ABS.value:
        segment = context.globalSegment
        contextSym = segment.addSymbol(symAddress, vromAddress=symVrom)
        contextSym.isElfNotype = True
    else:
        common.Utils.eprint(f"Warning: symbol '{symName}' has an unhandled stType: '{symEntry.stType}'")
        contextSym = segment.addSymbol(symAddress, vromAddress=symVrom)

    if symName is not None:
        if symName.startswith("."):
            contextSym._isStatic = True
        else:
            contextSym.name = symName
    contextSym.isUserDeclared = True
    contextSym.setSizeIfUnset(symEntry.size)

    return contextSym


def insertSymtabIntoContext(context: common.Context, symbolTable: elf32.Elf32Syms, stringTable: elf32.Elf32StringTable, elfFile: elf32.Elf32File):
    # Use the symbol table to replace symbol names present in disassembled sections
    for i, symEntry in enumerate(symbolTable):
        symName = stringTable[symEntry.name]

        if symEntry.shndx == 0:
            continue

        sectHeaderEntry = elfFile.sectionHeaders[symEntry.shndx]
        if sectHeaderEntry is None:
            continue

        if elfFile.header.type != elf32.Elf32ObjectFileType.REL.value:
            if symEntry.value != 0:
                addContextSymFromSymEntry(context, symEntry, symEntry.value, symName)
            continue

        sectName = elfFile.shstrtab[sectHeaderEntry.name]
        sectType = common.FileSectionType.fromStr(sectName)
        if sectType == common.FileSectionType.Invalid:
            common.Utils.eprint(f"Warning: symbol {i} (name: '{symName}', value: 0x{symEntry.value:X}) is referencing invalid section '{sectName}'")
            continue

        symVrom = symEntry.value + sectHeaderEntry.offset
        symAddress = symVrom

        contextSym = addContextSymFromSymEntry(context, symEntry, symAddress, symName, symVrom)
        if contextSym is not None:
            contextSym.sectionType = sectType

def insertDynsymIntoContext(context: common.Context, symbolTable: elf32.Elf32Syms, stringTable: elf32.Elf32StringTable):
    for symEntry in symbolTable:
        if symEntry.value == 0 or symEntry.shndx == 0:
            continue

        symName = stringTable[symEntry.name]
        addContextSymFromSymEntry(context, symEntry, symEntry.value, symName)

def insertGotIntoContext(context: common.Context, got: elf32.Elf32GlobalOffsetTable, stringTable: elf32.Elf32StringTable):
    lazyResolver = got.localsTable[0]
    contextSym: common.ContextSymbol|None
    contextSym = context.globalSegment.addSymbol(lazyResolver)
    contextSym.name = f"$$.LazyResolver"
    contextSym.isUserDeclared = True
    contextSym.isGotLocal = True

    gotIndex = len(got.localsTable)

    for gotEntry in got.globalsTable:
        symName = stringTable[gotEntry.symEntry.name]

        contextSym = addContextSymFromSymEntry(context, gotEntry.symEntry, gotEntry.getAddress(), symName)
        if contextSym is not None:
            contextSym.gotIndex = gotIndex
        gotIndex += 1


def injectAllElfSymbols(context: common.Context, elfFile: elf32.Elf32File, processedSegments: dict[common.FileSectionType, list[mips.sections.SectionBase]]) -> None:
    if elfFile.symtab is not None and elfFile.strtab is not None:
        # Inject symbols from the reloc table referenced in each section
        if elfFile.header.type == elf32.Elf32ObjectFileType.REL.value:
            for sectType, relocs in elfFile.rel.items():
                for rel in relocs:
                    symbolEntry = elfFile.symtab[rel.rSym]
                    symbolName = elfFile.strtab[symbolEntry.name]
                    if symbolName == "":
                        continue

                    subSegment = processedSegments[sectType][0]

                    relocVrom = subSegment.vromStart + rel.offset
                    relocInfo = common.RelocationInfo(common.RelocType(rel.rType), symbolName)
                    if symbolEntry.stType == elf32.Elf32SymbolTableType.SECTION.value:
                        sectionType = common.FileSectionType.fromStr(symbolName)
                        sectionVram = processedSegments[sectionType][0].vram
                        relocInfo.staticReference = common.RelocationStaticReference(sectionType, sectionVram)

                    context.globalRelocationOverrides[relocVrom] = relocInfo

        # Use the symtab to replace symbol names present in disassembled sections
        insertSymtabIntoContext(context, elfFile.symtab, elfFile.strtab, elfFile)

    if elfFile.dynsym is not None and elfFile.dynstr is not None:
        # Use the dynsym to replace symbol names present in disassembled sections
        insertDynsymIntoContext(context, elfFile.dynsym, elfFile.dynstr)

    if elfFile.got is not None and elfFile.dynstr is not None:
        insertGotIntoContext(context, elfFile.got, elfFile.dynstr)
    return

def processGlobalOffsetTable(context: common.Context, elfFile: elf32.Elf32File) -> None:
    if elfFile.dynamic is not None and elfFile.got is not None:
        common.GlobalConfig.GP_VALUE = elfFile.dynamic.getGpValue()

        globalsTable = [gotEntry.getAddress() for gotEntry in elfFile.got.globalsTable]

        assert elfFile.dynamic.pltGot is not None
        context.initGotTable(elfFile.dynamic.pltGot, elfFile.got.localsTable, globalsTable)
    return


def elfObjDisasmMain():
    args = getArgsParser().parse_args()
    applyArgs(args)

    applyGlobalConfigurations()

    context = common.Context()
    context.parseArgs(args)

    common.Utils.printQuietless(f"{PROGNAME} (spimdisasm {__version__})")

    inputPath = Path(args.binary)
    array_of_bytes = common.Utils.readFileAsBytearray(inputPath)
    elfFile = elf32.Elf32File(array_of_bytes)

    elfFile.handleHeaderIdent()
    elfFile.handleFlags()

    applyReadelfLikeFlags(elfFile, args)

    textOutput = Path(args.output)
    if args.data_output is None:
        dataOutput = textOutput
    else:
        dataOutput = Path(args.data_output)

    common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Reading segments...")
    if args.file_splits is not None:
        splits = common.FileSplitFormat()
        splits.readCsvFile(Path(args.file_splits))
        processedSegments, segmentPaths = fec.FrontendUtilities.getSplittedSections(context, splits, array_of_bytes, inputPath, textOutput, dataOutput)
    else:
        processedSegments, segmentPaths = getProcessedSections(context, elfFile, array_of_bytes, inputPath, textOutput, dataOutput)

    changeGlobalSegmentRanges(context, processedSegments)

    common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Processing global offset table...")
    processGlobalOffsetTable(context, elfFile)

    common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Injecting elf symbols...")
    injectAllElfSymbols(context, elfFile, processedSegments)

    processedFilesCount = 0
    for sect in processedSegments.values():
        processedFilesCount += len(sect)

    common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Analyzing sections...")
    fec.FrontendUtilities.analyzeProcessedFiles(processedSegments, segmentPaths, processedFilesCount)

    common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Writing files...")
    fec.FrontendUtilities.writeProcessedFiles(processedSegments, segmentPaths, processedFilesCount)

    if args.split_functions is not None:
        common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Migrating functions and rodata...")
        functionMigrationPath = Path(args.split_functions)
        fec.FrontendUtilities.migrateFunctions(processedSegments, functionMigrationPath)

        common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Generating functions list...")
        mips.FilesHandlers.writeMigratedFunctionsList(processedSegments, functionMigrationPath, inputPath.stem)

    if args.save_context is not None:
        common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Writing context...")
        contextPath = Path(args.save_context)
        contextPath.parent.mkdir(parents=True, exist_ok=True)
        context.saveContextToFile(contextPath)

    common.Utils.printQuietless(f"{PROGNAME} {inputPath}: Done!")
