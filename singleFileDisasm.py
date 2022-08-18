#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import os
from pathlib import Path

import spimdisasm


def disassemblerMain():
    description = "General purpose N64-mips disassembler"
    parser = argparse.ArgumentParser(description=description)

    if parser.prog == "simpleDisasm.py":
        spimdisasm.common.Utils.eprint(f"Deprecation warning")
        spimdisasm.common.Utils.eprint(f"\t The simpleDisasm.py script is deprecated")
        spimdisasm.common.Utils.eprint(f"\t Use singleFileDisasm.py instead")
        spimdisasm.common.Utils.eprint()

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

    parser.add_argument("--nuke-pointers", help="Use every technique available to remove pointers", action=spimdisasm.common.Utils.BooleanOptionalAction)
    parser.add_argument("--ignore-words", help="A space separated list of hex numbers. Any word differences which starts in any of the provided arguments will be ignored. Max value: FF. Only works when --nuke-pointers is passed", action="extend", nargs="+")

    parser.add_argument("--write-binary", help=f"Produce a binary from the processed file. Defaults to {spimdisasm.common.GlobalConfig.WRITE_BINARY}", action=spimdisasm.common.Utils.BooleanOptionalAction)


    spimdisasm.common.Context.addParametersToArgParse(parser)

    spimdisasm.common.GlobalConfig.addParametersToArgParse(parser)

    spimdisasm.mips.InstructionConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    spimdisasm.mips.InstructionConfig.parseArgs(args)

    spimdisasm.common.GlobalConfig.parseArgs(args)

    spimdisasm.common.GlobalConfig.REMOVE_POINTERS = args.nuke_pointers
    spimdisasm.common.GlobalConfig.IGNORE_BRANCHES = args.nuke_pointers
    if args.nuke_pointers:
        spimdisasm.common.GlobalConfig.IGNORE_WORD_LIST.add(0x80)
        if args.ignore_words:
            for upperByte in args.ignore_words:
                spimdisasm.common.GlobalConfig.IGNORE_WORD_LIST.add(int(upperByte, 16))
    if args.write_binary is not None:
        spimdisasm.common.GlobalConfig.WRITE_BINARY = args.write_binary

    spimdisasm.common.GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True
    spimdisasm.common.GlobalConfig.TRUST_USER_FUNCTIONS = True


    context = spimdisasm.common.Context()
    context.parseArgs(args)

    array_of_bytes = spimdisasm.common.Utils.readFileAsBytearray(args.binary)
    input_name = os.path.splitext(os.path.split(args.binary)[1])[0]

    processedFiles = {
        spimdisasm.common.FileSectionType.Text: [],
        spimdisasm.common.FileSectionType.Data: [],
        spimdisasm.common.FileSectionType.Rodata: [],
        spimdisasm.common.FileSectionType.Bss: [],
    }
    processedFilesOutputPaths = {k: [] for k in processedFiles}
    lenLastLine = 80

    splits = spimdisasm.common.FileSplitFormat()
    if args.file_splits is not None:
        splits.readCsvFile(args.file_splits)

    if len(splits) == 0:
        if args.file_splits is not None:
            spimdisasm.common.Utils.eprint("Warning: Tried to use file split mode, but passed csv splits file was empty")
            spimdisasm.common.Utils.eprint("\t Using single-file mode instead")

        start = int(args.start, 16)
        end = int(args.end, 16)
        fileVram = int(args.vram, 16)
        endVram = fileVram + end - start

        splitEntry = spimdisasm.common.FileSplitEntry(start, fileVram, "", spimdisasm.common.FileSectionType.Text, end, False, args.disasm_rsp)
        splits.append(splitEntry)

        splits.appendEndSection(end, endVram)

    splitsCount = len(splits)

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    highestVromEnd = len(array_of_bytes)
    lowestVramStart = 0x80000000
    highestVramEnd = lowestVramStart + highestVromEnd
    fileVram = int(args.vram, 16)
    if fileVram != 0:
        lowestVramStart = fileVram
        highestVramEnd = (fileVram & 0xF0000000) + highestVromEnd

    for row in splits:
        if row.section == spimdisasm.common.FileSectionType.Text:
            outputPath = textOutput
        elif row.section == spimdisasm.common.FileSectionType.Data:
            outputPath = dataOutput
        elif row.section == spimdisasm.common.FileSectionType.Rodata:
            outputPath = dataOutput
        elif row.section == spimdisasm.common.FileSectionType.Bss:
            outputPath = dataOutput
        else:
            spimdisasm.common.Utils.eprint("Error! Section not set!")
            exit(1)

        outputFilePath = outputPath
        if outputPath != "-":
            fileName = row.fileName
            if row.fileName == "":
                fileName = f"{input_name}_{row.vram:08X}"

            outputFilePath = os.path.join(outputPath, fileName)

        vramEnd = row.vram + row.nextOffset - row.offset
        if vramEnd > highestVramEnd:
            highestVramEnd = vramEnd
        if row.nextOffset > highestVromEnd:
            highestVromEnd = row.nextOffset

        spimdisasm.common.Utils.printVerbose(f"Reading '{row.fileName}'")
        f = spimdisasm.mips.FilesHandlers.createSectionFromSplitEntry(row, array_of_bytes, outputFilePath, context)
        f.setCommentOffset(row.offset)
        processedFiles[row.section].append(f)
        processedFilesOutputPaths[row.section].append(outputFilePath)

    context.globalSegment.changeRanges(0, highestVromEnd, lowestVramStart, highestVramEnd)

    i = 0
    for section, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[section]
        for fileIndex, f in enumerate(filesInSection):
            path = pathLists[fileIndex]
            spimdisasm.common.Utils.printQuietless(lenLastLine*" " + "\r", end="")
            progressStr = f"Analyzing: {i/splitsCount:%}. File: {path}\r"
            lenLastLine = max(len(progressStr), lenLastLine)
            spimdisasm.common.Utils.printQuietless(progressStr, end="", flush=True)
            spimdisasm.common.Utils.printVerbose("")

            f.analyze()
            f.printAnalyzisResults()

            i += 1


    processedFilesCount = 0
    for sect in processedFiles.values():
        processedFilesCount += len(sect)

    if args.nuke_pointers:
        spimdisasm.common.Utils.printVerbose("Nuking pointers...")
        i = 0
        for section, filesInSection in processedFiles.items():
            for path, f in filesInSection:
                spimdisasm.common.Utils.printVerbose(f"Nuking pointers of {path}")
                spimdisasm.common.Utils.printQuietless(lenLastLine*" " + "\r", end="")
                progressStr = f" Nuking pointers: {i/processedFilesCount:%}. File: {path}\r"
                lenLastLine = max(len(progressStr), lenLastLine)
                spimdisasm.common.Utils.printQuietless(progressStr, end="")

                f.removePointers()
                i += 1

    spimdisasm.common.Utils.printVerbose("Writing files...")
    i = 0
    for section, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[section]
        for fileIndex, f in enumerate(filesInSection):
            path = pathLists[fileIndex]
            spimdisasm.common.Utils.printVerbose(f"Writing {path}")
            spimdisasm.common.Utils.printQuietless(lenLastLine*" " + "\r", end="")
            progressStr = f"Writing: {i/processedFilesCount:%}. File: {path}\r"
            lenLastLine = max(len(progressStr), lenLastLine)
            spimdisasm.common.Utils.printQuietless(progressStr, end="")

            if path == "-":
                spimdisasm.common.Utils.printQuietless()

            spimdisasm.mips.FilesHandlers.writeSection(path, f)
            i += 1

    if args.split_functions is not None:
        spimdisasm.common.Utils.printVerbose("\nSpliting functions...")
        funcTotal = sum(len(x.symbolList) for x in processedFiles[spimdisasm.common.FileSectionType.Text])
        i = 0
        for f in processedFiles[spimdisasm.common.FileSectionType.Text]:
            for func in f.symbolList:
                spimdisasm.common.Utils.printVerbose(f"Spliting {func.getName()}", end="")
                spimdisasm.common.Utils.printQuietless(lenLastLine*" " + "\r", end="")
                spimdisasm.common.Utils.printVerbose()
                progressStr = f" Writing: {i/funcTotal:%}. Function: {func.getName()}\r"
                lenLastLine = max(len(progressStr), lenLastLine)
                spimdisasm.common.Utils.printQuietless(progressStr, end="")


                assert isinstance(func, spimdisasm.mips.symbols.SymbolFunction)
                spimdisasm.mips.FilesHandlers.writeSplitedFunction(os.path.join(args.split_functions, f.name), func, processedFiles[spimdisasm.common.FileSectionType.Rodata])

                i += 1
        spimdisasm.mips.FilesHandlers.writeOtherRodata(args.split_functions, processedFiles[spimdisasm.common.FileSectionType.Rodata])

    if args.save_context is not None:
        contextPath = Path(args.save_context)
        contextPath.parent.mkdir(parents=True, exist_ok=True)
        context.saveContextToFile(contextPath)

    spimdisasm.common.Utils.printQuietless(lenLastLine*" " + "\r", end="")
    spimdisasm.common.Utils.printQuietless(f"Done: {args.binary}")

    spimdisasm.common.Utils.printVerbose()
    spimdisasm.common.Utils.printVerbose("Disassembling complete!")
    spimdisasm.common.Utils.printVerbose("Goodbye.")


if __name__ == "__main__":
    disassemblerMain()
