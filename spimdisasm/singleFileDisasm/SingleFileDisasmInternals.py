#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from pathlib import Path

from .. import common
from .. import mips


sLenLastLine = 80

def getArgsParser() -> argparse.ArgumentParser:
    description = "General purpose N64-mips disassembler"
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    parser_singleFile = parser.add_argument_group("Single file disassembly options")

    parser_singleFile.add_argument("--start", help="Raw offset of the input binary file to start disassembling. Expects an hex value", default="0")
    parser_singleFile.add_argument("--end", help="Offset end of the input binary file to start disassembling. Expects an hex value",  default="0xFFFFFF")
    parser_singleFile.add_argument("--vram", help="Set the VRAM address. Expects an hex value", default="0x0")

    parser_singleFile.add_argument("--disasm-rsp", help=f"Experimental. Disassemble this file using rsp ABI instructions. Warning: In its current state the generated asm may not be assemblable to a matching binary. Defaults to False", action="store_true")

    parser.add_argument("--file-splits", help="Path to a file splits csv")

    parser.add_argument("--split-functions", help="Enables the function and rodata splitter. Expects a path to place the splited functions", metavar="PATH")

    parser.add_argument("--nuke-pointers", help="Use every technique available to remove pointers", action=common.Utils.BooleanOptionalAction)
    parser.add_argument("--ignore-words", help="A space separated list of hex numbers. Any word differences which starts in any of the provided arguments will be ignored. Max value: FF. Only works when --nuke-pointers is passed", action="extend", nargs="+")

    parser.add_argument("--write-binary", help=f"Produce a binary from the processed file. Defaults to {common.GlobalConfig.WRITE_BINARY}", action=common.Utils.BooleanOptionalAction)


    common.Context.addParametersToArgParse(parser)

    common.GlobalConfig.addParametersToArgParse(parser)

    mips.InstructionConfig.addParametersToArgParse(parser)

    return parser

def applyArgs(args: argparse.Namespace) -> None:
    mips.InstructionConfig.parseArgs(args)

    common.GlobalConfig.parseArgs(args)

    common.GlobalConfig.REMOVE_POINTERS = args.nuke_pointers
    common.GlobalConfig.IGNORE_BRANCHES = args.nuke_pointers
    if args.nuke_pointers:
        common.GlobalConfig.IGNORE_WORD_LIST.add(0x80)
        if args.ignore_words:
            for upperByte in args.ignore_words:
                common.GlobalConfig.IGNORE_WORD_LIST.add(int(upperByte, 16))
    if args.write_binary is not None:
        common.GlobalConfig.WRITE_BINARY = args.write_binary

def applyGlobalConfigurations() -> None:
    common.GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True
    common.GlobalConfig.TRUST_USER_FUNCTIONS = True

def getSplits(fileSplitsPath: Path|None, vromStart: int, vromEnd: int, fileVram: int, disasmRsp: bool) -> common.FileSplitFormat:
    splits = common.FileSplitFormat()
    if fileSplitsPath is not None:
        splits.readCsvFile(fileSplitsPath)

    if len(splits) == 0:
        if fileSplitsPath is not None:
            common.Utils.eprint("Warning: Tried to use file split mode, but passed csv splits file was empty")
            common.Utils.eprint("\t Using single-file mode instead")

        endVram = fileVram + vromEnd - vromStart

        splitEntry = common.FileSplitEntry(vromStart, fileVram, "", common.FileSectionType.Text, vromEnd, False, disasmRsp)
        splits.append(splitEntry)

        splits.appendEndSection(vromEnd, endVram)

    return splits

def getProcessedSections(context: common.Context, splits: common.FileSplitFormat, array_of_bytes: bytearray, inputPath: Path, textOutput: Path, dataOutput: Path):
    processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]] = {
        common.FileSectionType.Text: [],
        common.FileSectionType.Data: [],
        common.FileSectionType.Rodata: [],
        common.FileSectionType.Bss: [],
    }
    processedFilesOutputPaths: dict[common.FileSectionType, list[Path]] = {k: [] for k in processedFiles}

    for row in splits:
        if row.section == common.FileSectionType.Text:
            outputPath = textOutput
        elif row.section == common.FileSectionType.Data:
            outputPath = dataOutput
        elif row.section == common.FileSectionType.Rodata:
            outputPath = dataOutput
        elif row.section == common.FileSectionType.Bss:
            outputPath = dataOutput
        else:
            common.Utils.eprint("Error! Section not set!")
            exit(1)

        outputFilePath = outputPath
        if str(outputPath) != "-":
            fileName = row.fileName
            if row.fileName == "":
                fileName = f"{inputPath.stem}_{row.vram:08X}"

            outputFilePath = outputPath / fileName

        common.Utils.printVerbose(f"Reading '{row.fileName}'")
        f = mips.FilesHandlers.createSectionFromSplitEntry(row, array_of_bytes, outputFilePath, context)
        f.setCommentOffset(row.offset)
        processedFiles[row.section].append(f)
        processedFilesOutputPaths[row.section].append(outputFilePath)

    return processedFiles, processedFilesOutputPaths

def changeGlobalSegmentRanges(context: common.Context, processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]], fileSize: int, fileVram: int) -> None:
    highestVromEnd = fileSize
    lowestVramStart = 0x80000000
    highestVramEnd = lowestVramStart + highestVromEnd
    if fileVram != 0:
        lowestVramStart = fileVram
        highestVramEnd = (fileVram & 0xF0000000) + highestVromEnd

    for filesInSection in processedFiles.values():
        for mipsSection in filesInSection:
            if lowestVramStart is None or mipsSection.vram < lowestVramStart:
                lowestVramStart = mipsSection.vram
            if highestVramEnd is None or mipsSection.vramEnd > highestVramEnd:
                highestVramEnd = mipsSection.vramEnd

    if lowestVramStart is None:
        lowestVramStart = 0x0
    if highestVramEnd is None:
        highestVramEnd = 0xFFFFFFFF
    context.globalSegment.changeRanges(0, highestVromEnd, lowestVramStart, highestVramEnd)
    return

def analyzeProcessedFiles(processedFiles, processedFilesOutputPaths, processedFilesCount: int):
    global sLenLastLine

    i = 0
    for section, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[section]
        for fileIndex, f in enumerate(filesInSection):
            path = pathLists[fileIndex]
            common.Utils.printQuietless(sLenLastLine*" " + "\r", end="")
            progressStr = f"Analyzing: {i/processedFilesCount:%}. File: {path}\r"
            sLenLastLine = max(len(progressStr), sLenLastLine)
            common.Utils.printQuietless(progressStr, end="", flush=True)
            common.Utils.printVerbose("")

            f.analyze()
            f.printAnalyzisResults()

            i += 1
    return

def nukePointers(processedFiles, processedFilesCount: int):
    global sLenLastLine

    common.Utils.printVerbose("Nuking pointers...")
    i = 0
    for section, filesInSection in processedFiles.items():
        for path, f in filesInSection:
            common.Utils.printVerbose(f"Nuking pointers of {path}")
            common.Utils.printQuietless(sLenLastLine*" " + "\r", end="")
            progressStr = f" Nuking pointers: {i/processedFilesCount:%}. File: {path}\r"
            sLenLastLine = max(len(progressStr), sLenLastLine)
            common.Utils.printQuietless(progressStr, end="")

            f.removePointers()
            i += 1
    return

def writeProcessedFiles(processedFiles, processedFilesOutputPaths, processedFilesCount: int):
    global sLenLastLine

    common.Utils.printVerbose("Writing files...")
    i = 0
    for section, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[section]
        for fileIndex, f in enumerate(filesInSection):
            path = pathLists[fileIndex]
            common.Utils.printVerbose(f"Writing {path}")
            common.Utils.printQuietless(sLenLastLine*" " + "\r", end="")
            progressStr = f"Writing: {i/processedFilesCount:%}. File: {path}\r"
            sLenLastLine = max(len(progressStr), sLenLastLine)
            common.Utils.printQuietless(progressStr, end="")

            if path == "-":
                common.Utils.printQuietless()

            mips.FilesHandlers.writeSection(Path(path), f)
            i += 1
    return

def migrateFunctions(processedFiles, functionMigrationPath: Path):
    global sLenLastLine

    common.Utils.printVerbose("\nSpliting functions...")
    funcTotal = sum(len(x.symbolList) for x in processedFiles[common.FileSectionType.Text])
    i = 0
    for f in processedFiles[common.FileSectionType.Text]:
        for func in f.symbolList:
            common.Utils.printVerbose(f"Spliting {func.getName()}", end="")
            common.Utils.printQuietless(sLenLastLine*" " + "\r", end="")
            common.Utils.printVerbose()
            progressStr = f" Writing: {i/funcTotal:%}. Function: {func.getName()}\r"
            sLenLastLine = max(len(progressStr), sLenLastLine)
            common.Utils.printQuietless(progressStr, end="")

            assert isinstance(func, mips.symbols.SymbolFunction)
            functionPath = functionMigrationPath / f.name
            mips.FilesHandlers.writeSplitedFunction(functionPath, func, processedFiles[common.FileSectionType.Rodata])

            i += 1
    mips.FilesHandlers.writeOtherRodata(functionMigrationPath, processedFiles[common.FileSectionType.Rodata])


def disassemblerMain():
    args = getArgsParser().parse_args()
    applyArgs(args)

    applyGlobalConfigurations()

    context = common.Context()
    context.parseArgs(args)

    inputPath = Path(args.binary)
    array_of_bytes = common.Utils.readFileAsBytearray(inputPath)

    fileSplitsPath = None
    if args.file_splits is not None:
        fileSplitsPath = Path(args.file_splits)
    vromStart = int(args.start, 16)
    vromEnd = int(args.end, 16)
    if vromEnd == 0xFFFFFF:
        vromEnd = len(array_of_bytes)
    fileVram = int(args.vram, 16)
    splits = getSplits(fileSplitsPath, vromStart, vromEnd, fileVram, args.disasm_rsp)

    textOutput = Path(args.output)
    if args.data_output is None:
        dataOutput = textOutput
    else:
        dataOutput = Path(args.data_output)

    processedFiles, processedFilesOutputPaths = getProcessedSections(context, splits, array_of_bytes, inputPath, textOutput, dataOutput)
    changeGlobalSegmentRanges(context, processedFiles, len(array_of_bytes), int(args.vram, 16))

    processedFilesCount = 0
    for sect in processedFiles.values():
        processedFilesCount += len(sect)

    analyzeProcessedFiles(processedFiles, processedFilesOutputPaths, processedFilesCount)

    if args.nuke_pointers:
        nukePointers(processedFiles, processedFilesCount)

    writeProcessedFiles(processedFiles, processedFilesOutputPaths, processedFilesCount)

    if args.split_functions is not None:
        migrateFunctions(processedFiles, Path(args.split_functions))

    if args.save_context is not None:
        contextPath = Path(args.save_context)
        contextPath.parent.mkdir(parents=True, exist_ok=True)
        context.saveContextToFile(contextPath)

    common.Utils.printQuietless(sLenLastLine*" " + "\r", end="")
    common.Utils.printQuietless(f"Done: {args.binary}")

    common.Utils.printVerbose()
    common.Utils.printVerbose("Disassembling complete!")
    common.Utils.printVerbose("Goodbye.")
