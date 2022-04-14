#!/usr/bin/env python3

from __future__ import annotations

import argparse

from backend.common.Utils import *
from backend.common.GlobalConfig import GlobalConfig, printQuietless, printVerbose
from backend.common.Context import Context
from backend.common.FileSplitFormat import FileSplitFormat, FileSectionType, FileSplitEntry

from backend.mips.MipsText import Text
from backend.mips.FilesHandlers import createSectionFromSplitEntry, analyzeSectionFromSplitEntry, writeSection, writeSplitedFunction, writeOtherRodata


def disassemblerMain():
    description = "General purpose N64-mips disassembler"
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    parser_singleFile = parser.add_argument_group("Single file disassembly options")

    parser_singleFile.add_argument("--start", help="", default="0")
    parser_singleFile.add_argument("--end", help="",  default="0xFFFFFF")
    parser_singleFile.add_argument("--vram", help="Set the VRAM address", default="-1")

    parser.add_argument("--file-splits", help="Path to a file splits csv")

    parser.add_argument("--split-functions", help="Enables the function and rodata splitter. Expects a path to place the splited functions", metavar="PATH")

    parser.add_argument("--nuke-pointers", help="Use every technique available to remove pointers", action="store_true")

    Context.addParametersToArgParse(parser)

    GlobalConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    GlobalConfig.parseArgs(args)

    GlobalConfig.REMOVE_POINTERS = args.nuke_pointers
    GlobalConfig.IGNORE_BRANCHES = args.nuke_pointers
    if args.nuke_pointers:
        GlobalConfig.IGNORE_WORD_LIST.add(0x80)

    GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True
    GlobalConfig.TRUST_USER_FUNCTIONS = True


    context = Context()
    context.parseArgs(args)

    array_of_bytes = readFileAsBytearray(args.binary)
    input_name = os.path.splitext(os.path.split(args.binary)[1])[0]

    processedFiles = {
        FileSectionType.Text: [],
        FileSectionType.Data: [],
        FileSectionType.Rodata: [],
        FileSectionType.Bss: [],
    }
    lenLastLine = 80

    splits = FileSplitFormat()
    if args.file_splits is not None:
        splits.readCsvFile(args.file_splits)

    if len(splits) == 0:
        if args.file_splits is not None:
            eprint("Warning: Tried to use file split mode, but passed csv splits file was empty")
            eprint("\t Using single-file mode instead")

        start = int(args.start, 16)
        end = int(args.end, 16)

        fileVram = int(args.vram, 16)

        endVram = fileVram
        if endVram >= 0:
            endVram += end - start

        splitEntry = FileSplitEntry(start, fileVram, "", FileSectionType.Text, end, False, GlobalConfig.DISASSEMBLE_RSP)
        splits.append(splitEntry)

        splits.appendEndSection(end, endVram)

    splitsCount = len(splits)

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    i = 0
    for row in splits:
        if row.section == FileSectionType.Text:
            outputPath = textOutput
        elif row.section == FileSectionType.Data:
            outputPath = dataOutput
        elif row.section == FileSectionType.Rodata:
            outputPath = dataOutput
        elif row.section == FileSectionType.Bss:
            outputPath = dataOutput
        else:
            eprint("Error! Section not set!")
            exit(1)

        outputFilePath = outputPath
        if outputPath != "-":
            fileName = row.fileName
            if row.fileName == "":
                fileName = f"{input_name}_{row.vram:08X}"

            outputFilePath = os.path.join(outputPath, fileName)

        printVerbose(f"Reading '{row.fileName}'")
        f = createSectionFromSplitEntry(row, array_of_bytes, outputFilePath, context)
        analyzeSectionFromSplitEntry(f, row)
        processedFiles[row.section].append((outputFilePath, f))

        printQuietless(lenLastLine*" " + "\r", end="")
        progressStr = f" Analyzing: {i/splitsCount:%}. File: {row.fileName}\r"
        lenLastLine = max(len(progressStr), lenLastLine)
        printQuietless(progressStr, end="", flush=True)

        printVerbose("\n")
        i += 1

    processedFilesCount = 0
    processedFiles
    for sect in processedFiles.values():
        processedFilesCount += len(sect)

    if args.nuke_pointers:
        printVerbose("Nuking pointers...")
        i = 0
        for section, filesInSection in processedFiles.items():
            for path, f in filesInSection:
                printVerbose(f"Nuking pointers of {path}")
                printQuietless(lenLastLine*" " + "\r", end="")
                progressStr = f" Nuking pointers: {i/processedFilesCount:%}. File: {path}\r"
                lenLastLine = max(len(progressStr), lenLastLine)
                printQuietless(progressStr, end="")

                f.removePointers()
                i += 1

    printVerbose("Writing files...")
    i = 0
    for section, filesInSection in processedFiles.items():
        for path, f in filesInSection:
            printVerbose(f"Writing {path}")
            printQuietless(lenLastLine*" " + "\r", end="")
            progressStr = f" Writing: {i/processedFilesCount:%}. File: {path}\r"
            lenLastLine = max(len(progressStr), lenLastLine)
            printQuietless(progressStr, end="")

            if path == "-":
                printQuietless()

            writeSection(path, f)
            i += 1

    if args.split_functions is not None:
        printVerbose("Spliting functions")
        for path, f in processedFiles[FileSectionType.Text]:
            file: Text = f
            for func in file.functions:
                writeSplitedFunction(os.path.join(args.split_functions, file.filename), func, processedFiles[FileSectionType.Rodata], context)
        writeOtherRodata(args.split_functions, processedFiles[FileSectionType.Rodata], context)

    if args.save_context is not None:
        head, tail = os.path.split(args.save_context)
        if head != "":
            os.makedirs(head, exist_ok=True)
        context.saveContextToFile(args.save_context)

    printQuietless(lenLastLine*" " + "\r", end="")
    printQuietless(f"Done: {args.binary}")

    printVerbose()
    printVerbose("Disassembling complete!")
    printVerbose("Goodbye.")


if __name__ == "__main__":
    disassemblerMain()
