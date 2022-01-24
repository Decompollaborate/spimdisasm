#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig, printQuietless, printVerbose
from mips.FileSplitFormat import FileSplitFormat, FileSectionType, FileSplitEntry
from mips.MipsText import Text
from mips.MipsContext import Context
from mips.FilesHandlers import createSectionFromSplitEntry, analyzeSectionFromSplitEntry, writeSection, writeSplitedFunction, writeOtherRodata


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

    parser.add_argument("--add-filename", help="Adds the filename of the file to the generated function/variable name")

    args = parser.parse_args()

    GlobalConfig.parseArgs(args)

    GlobalConfig.REMOVE_POINTERS = args.nuke_pointers
    GlobalConfig.IGNORE_BRANCHES = args.nuke_pointers
    if args.nuke_pointers:
        GlobalConfig.IGNORE_WORD_LIST.add(0x80)

    GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True
    GlobalConfig.TRUST_USER_FUNCTIONS = True


    newStuffSuffix = args.add_filename
    if newStuffSuffix is None:
        newStuffSuffix = ""

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

    if args.file_splits is None:
        splitEntry = FileSplitEntry(int(args.start, 16), int(args.vram, 16), "", FileSectionType.Text, int(args.end, 16), False, GlobalConfig.DISASSEMBLE_RSP)
        f = createSectionFromSplitEntry(splitEntry, array_of_bytes, args.output, context)
        f.newStuffSuffix = newStuffSuffix
        analyzeSectionFromSplitEntry(f, splitEntry)
        processedFiles[FileSectionType.Text].append((args.output, f))
    else:
        splits = FileSplitFormat(args.file_splits)

        splitsCount = len(splits)

        textOutput = args.output
        dataOutput = args.data_output
        if dataOutput is None:
            dataOutput = textOutput

        outputPath = args.output
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

            fileName = row.fileName
            if row.fileName == "":
                fileName = f"{input_name}_{row.vram:08X}"

            outputPath = f"{outputPath}/{fileName}"

            printVerbose(f"Reading '{row.fileName}'")
            f = createSectionFromSplitEntry(row, array_of_bytes, outputPath, context)
            f.newStuffSuffix = newStuffSuffix
            analyzeSectionFromSplitEntry(f, row)
            processedFiles[row.section].append((outputPath, f))

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
