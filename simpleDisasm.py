#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import os

import backend as disasmBack


def disassemblerMain():
    description = "General purpose N64-mips disassembler"
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    parser_singleFile = parser.add_argument_group("Single file disassembly options")

    parser_singleFile.add_argument("--start", help="", default="0")
    parser_singleFile.add_argument("--end", help="",  default="0xFFFFFF")
    parser_singleFile.add_argument("--vram", help="Set the VRAM address")

    parser.add_argument("--file-splits", help="Path to a file splits csv")

    parser.add_argument("--split-functions", help="Enables the function and rodata splitter. Expects a path to place the splited functions", metavar="PATH")

    parser.add_argument("--nuke-pointers", help="Use every technique available to remove pointers", action="store_true")

    disasmBack.Context.addParametersToArgParse(parser)

    disasmBack.GlobalConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    disasmBack.GlobalConfig.parseArgs(args)

    disasmBack.GlobalConfig.REMOVE_POINTERS = args.nuke_pointers
    disasmBack.GlobalConfig.IGNORE_BRANCHES = args.nuke_pointers
    if args.nuke_pointers:
        disasmBack.GlobalConfig.IGNORE_WORD_LIST.add(0x80)

    disasmBack.GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True
    disasmBack.GlobalConfig.TRUST_USER_FUNCTIONS = True


    context = disasmBack.Context()
    context.parseArgs(args)

    array_of_bytes = disasmBack.Utils.readFileAsBytearray(args.binary)
    input_name = os.path.splitext(os.path.split(args.binary)[1])[0]

    processedFiles = {
        disasmBack.FileSectionType.Text: [],
        disasmBack.FileSectionType.Data: [],
        disasmBack.FileSectionType.Rodata: [],
        disasmBack.FileSectionType.Bss: [],
    }
    processedFilesOutputPaths = {k: [] for k in processedFiles}
    lenLastLine = 80

    splits = disasmBack.FileSplitFormat()
    if args.file_splits is not None:
        splits.readCsvFile(args.file_splits)

    if len(splits) == 0:
        if args.file_splits is not None:
            disasmBack.Utils.eprint("Warning: Tried to use file split mode, but passed csv splits file was empty")
            disasmBack.Utils.eprint("\t Using single-file mode instead")

        start = int(args.start, 16)
        end = int(args.end, 16)

        fileVram = None
        if args.vram is not None:
            fileVram = int(args.vram, 16)

        endVram = fileVram
        if endVram is not None:
            endVram += end - start

        splitEntry = disasmBack.FileSplitEntry(start, fileVram, "", disasmBack.FileSectionType.Text, end, False, disasmBack.GlobalConfig.DISASSEMBLE_RSP)
        splits.append(splitEntry)

        splits.appendEndSection(end, endVram)

    splitsCount = len(splits)

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    i = 0
    for row in splits:
        if row.section == disasmBack.FileSectionType.Text:
            outputPath = textOutput
        elif row.section == disasmBack.FileSectionType.Data:
            outputPath = dataOutput
        elif row.section == disasmBack.FileSectionType.Rodata:
            outputPath = dataOutput
        elif row.section == disasmBack.FileSectionType.Bss:
            outputPath = dataOutput
        else:
            disasmBack.Utils.eprint("Error! Section not set!")
            exit(1)

        outputFilePath = outputPath
        if outputPath != "-":
            fileName = row.fileName
            if row.fileName == "":
                fileName = f"{input_name}_{row.vram:08X}"

            outputFilePath = os.path.join(outputPath, fileName)

        disasmBack.Utils.printVerbose(f"Reading '{row.fileName}'")
        f = disasmBack.mips.FilesHandlers.createSectionFromSplitEntry(row, array_of_bytes, outputFilePath, context)
        disasmBack.mips.FilesHandlers.analyzeSectionFromSplitEntry(f, row)
        processedFiles[row.section].append(f)
        processedFilesOutputPaths[row.section].append(outputFilePath)

        disasmBack.Utils.printQuietless(lenLastLine*" " + "\r", end="")
        progressStr = f" Analyzing: {i/splitsCount:%}. File: {row.fileName}\r"
        lenLastLine = max(len(progressStr), lenLastLine)
        disasmBack.Utils.printQuietless(progressStr, end="", flush=True)

        disasmBack.Utils.printVerbose("\n")
        i += 1

    processedFilesCount = 0
    for sect in processedFiles.values():
        processedFilesCount += len(sect)

    if args.nuke_pointers:
        disasmBack.Utils.printVerbose("Nuking pointers...")
        i = 0
        for section, filesInSection in processedFiles.items():
            for path, f in filesInSection:
                disasmBack.Utils.printVerbose(f"Nuking pointers of {path}")
                disasmBack.Utils.printQuietless(lenLastLine*" " + "\r", end="")
                progressStr = f" Nuking pointers: {i/processedFilesCount:%}. File: {path}\r"
                lenLastLine = max(len(progressStr), lenLastLine)
                disasmBack.Utils.printQuietless(progressStr, end="")

                f.removePointers()
                i += 1

    disasmBack.Utils.printVerbose("Writing files...")
    i = 0
    for section, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[section]
        for fileIndex, f in enumerate(filesInSection):
            path = pathLists[fileIndex]
            disasmBack.Utils.printVerbose(f"Writing {path}")
            disasmBack.Utils.printQuietless(lenLastLine*" " + "\r", end="")
            progressStr = f" Writing: {i/processedFilesCount:%}. File: {path}\r"
            lenLastLine = max(len(progressStr), lenLastLine)
            disasmBack.Utils.printQuietless(progressStr, end="")

            if path == "-":
                disasmBack.Utils.printQuietless()

            disasmBack.mips.FilesHandlers.writeSection(path, f)
            i += 1

    if args.split_functions is not None:
        disasmBack.Utils.printVerbose("Spliting functions")
        for f in processedFiles[disasmBack.FileSectionType.Text]:
            file: disasmBack.mips.Text = f
            for func in file.symbolList:
                assert isinstance(func, disasmBack.mips.Symbols.SymbolFunction)
                disasmBack.mips.FilesHandlers.writeSplitedFunction(os.path.join(args.split_functions, file.name), func, processedFiles[disasmBack.FileSectionType.Rodata], context)
        disasmBack.mips.FilesHandlers.writeOtherRodata(args.split_functions, processedFiles[disasmBack.FileSectionType.Rodata], context)

    if args.save_context is not None:
        head, tail = os.path.split(args.save_context)
        if head != "":
            os.makedirs(head, exist_ok=True)
        context.saveContextToFile(args.save_context)

    disasmBack.Utils.printQuietless(lenLastLine*" " + "\r", end="")
    disasmBack.Utils.printQuietless(f"Done: {args.binary}")

    disasmBack.Utils.printVerbose()
    disasmBack.Utils.printVerbose("Disassembling complete!")
    disasmBack.Utils.printVerbose("Goodbye.")


if __name__ == "__main__":
    disassemblerMain()
