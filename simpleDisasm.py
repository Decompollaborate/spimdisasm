#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig, printQuietless, printVerbose
from mips.FileSplitFormat import FileSplitFormat, FileSectionType, FileSplitEntry
from mips.MipsSection import Section
from mips.MipsText import Text
from mips.MipsData import Data
from mips.MipsRodata import Rodata
from mips.MipsBss import Bss
from mips.MipsFunction import Function
from mips.MipsContext import Context


def simpleDisasm_CreateSection(splitEntry: FileSplitEntry, array_of_bytes: bytearray, outputPath: str, context: Context) -> Section:
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

    if splitEntry.section == FileSectionType.Text:
        f = Text(array_of_bytes, tail, "ver", context)
    elif splitEntry.section == FileSectionType.Data:
        f = Data(array_of_bytes, tail, "ver", context)
    elif splitEntry.section == FileSectionType.Rodata:
        f = Rodata(array_of_bytes, tail, "ver", context)
    elif splitEntry.section == FileSectionType.Bss:
        f = Bss(splitEntry.vram, splitEntry.vram + offsetEnd - offsetStart, tail, "ver", context)
    else:
        eprint("Error! Section not set!")
        exit(-1)

    f.isHandwritten = splitEntry.isHandwritten
    f.isRsp = splitEntry.isRsp

    if splitEntry.vram >= 0:
        printVerbose(f"Using VRAM {splitEntry.vram:08X}")
        f.setVRamStart(splitEntry.vram)

    return f

def simpleDisasm_AnalyzeSection(fileSection: Section, splitEntry: FileSplitEntry):
    offsetStart = splitEntry.offset

    printVerbose("Analyzing")
    fileSection.analyze()
    fileSection.setCommentOffset(offsetStart)

    printVerbose()

    fileSection.printAnalyzisResults()

    return fileSection



def writeSection(x):
    path, f = x

    head, tail = os.path.split(path)

    # Create directories
    if head != "":
        os.makedirs(head, exist_ok=True)

    f.saveToFile(path)

    return path

def writeSplitedFunction(path: str, func: Function, rodataFileList: List[Tuple[str, Rodata]], context: Context):
    os.makedirs(path, exist_ok=True)
    with open(os.path.join(path, func.name) + ".s", "w") as f:
        rdataList = []
        lateRodataList = []
        lateRodataLen = 0
        firstRodata = None
        for _, rodata in rodataFileList:
            if len(rdataList) > 0 or len(lateRodataList) > 0:
                # We already have the rodata for this function. Stop searching
                break

            # Skip the file if there's nothing in this file refenced by the current function
            intersection = func.referencedVRams & rodata.symbolsVRams
            if len(intersection) == 0:
                continue

            sortedSymbolVRams = sorted(rodata.symbolsVRams)

            for vram in sorted(intersection):
                nextVramIndex = sortedSymbolVRams.index(vram) + 1
                nextVram = float("inf") if nextVramIndex >= len(sortedSymbolVRams) else sortedSymbolVRams[nextVramIndex]

                rodataSymbol = context.getGenericSymbol(vram, False)
                assert rodataSymbol is not None
                # We only care for rodata that's used once
                if rodataSymbol.referenceCounter != 1:
                    break

                j = 0
                while j < len(rodata.words):
                    rodataVram = rodata.getVramOffset(j*4)
                    if rodataVram < vram:
                        j += 1
                        continue
                    if rodataVram >= nextVram:
                        break

                    if firstRodata is None:
                        firstRodata = rodata.vRamStart

                    nthRodata, skip = rodata.getNthWord(j)
                    j += skip
                    j += 1
                    if rodataSymbol.isLateRodata:
                        lateRodataList.append(nthRodata)
                        lateRodataList.append("\n")
                        lateRodataLen += 1
                    else:
                        rdataList.append(nthRodata)
                        rdataList.append("\n")

                if rodataSymbol.isLateRodata:
                    lateRodataList.append("\n")
                else:
                    rdataList.append("\n")

        if len(rdataList) > 0:
            # Write the rdata
            f.write(".rdata\n")
            for x in rdataList:
                f.write(x)

            f.write("\n.text\n")

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

def writeOtherRodata(path: str, rodataFileList: List[Tuple[str, Rodata]], context: Context):
    for _, rodata in rodataFileList:
        rodataPath = os.path.join(path, rodata.filename)
        os.makedirs(rodataPath, exist_ok=True)
        sortedSymbolVRams = sorted(rodata.symbolsVRams)

        for vram in sortedSymbolVRams:
            nextVramIndex = sortedSymbolVRams.index(vram) + 1
            nextVram = float("inf") if nextVramIndex >= len(sortedSymbolVRams) else sortedSymbolVRams[nextVramIndex]

            rodataSymbol = context.getGenericSymbol(vram, False)
            assert rodataSymbol is not None
            if rodataSymbol.referenceCounter == 1:
                continue

            rodataSymbolPath = os.path.join(rodataPath, rodataSymbol.name) + ".s"
            # print(rodataSymbolPath, rodataSymbol.referenceCounter)

            with open(rodataSymbolPath, "w") as f:
                f.write(".rdata\n")
                j = 0
                while j < len(rodata.words):
                    rodataVram = rodata.getVramOffset(j*4)
                    if rodataVram < vram:
                        j += 1
                        continue
                    if rodataVram >= nextVram:
                        break

                    nthRodata, skip = rodata.getNthWord(j)
                    f.write(nthRodata)
                    f.write("\n")
                    j += skip
                    j += 1


def disassemblerMain():
    description = "General purpose N64-mips disassembler"
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    parser.add_argument("--start", help="", default="0")
    parser.add_argument("--end", help="",  default="0xFFFFFF")
    parser.add_argument("--vram", help="Set the VRAM address", default="-1")

    parser.add_argument("--file-splits", help="Path to a file splits csv")

    parser.add_argument("--split-functions", help="Enables the function and rodata splitter. Expects a path to place the splited functions", metavar="PATH")

    parser.add_argument("--add-filename", help="Adds the filename of the file to the generated function/variable name")

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
        f = simpleDisasm_CreateSection(splitEntry, array_of_bytes, args.output, context)
        f.newStuffSuffix = newStuffSuffix
        simpleDisasm_AnalyzeSection(f, splitEntry)
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
            f = simpleDisasm_CreateSection(row, array_of_bytes, outputPath, context)
            f.newStuffSuffix = newStuffSuffix
            simpleDisasm_AnalyzeSection(f, row)
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

            writeSection((path, f))
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
