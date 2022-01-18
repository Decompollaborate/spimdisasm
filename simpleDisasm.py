#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig, printQuietless, printVerbose
from mips.FileSplitFormat import FileSplitFormat, FileSectionType
from mips.MipsSection import Section
from mips.MipsText import Text
from mips.MipsData import Data
from mips.MipsRodata import Rodata
from mips.MipsBss import Bss
from mips.MipsFunction import Function
from mips.MipsContext import Context


def simpleDisasmFile(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context, isHandwritten: bool=False, isRsp: bool=False, newStuffSuffix: str=""):
    head, tail = os.path.split(outputPath)

    if offsetEnd >= 0:
        printVerbose(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    if offsetStart >= 0:
        printVerbose(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    f = Text(array_of_bytes, tail, "ver", context)
    f.isHandwritten = isHandwritten
    f.isRsp = isRsp
    f.newStuffSuffix = newStuffSuffix

    if vram >= 0:
        printVerbose(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    printVerbose("Analyzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    printVerbose()
    printVerbose(f"Found {f.nFuncs} functions.")

    nBoundaries = len(f.fileBoundaries)
    if nBoundaries > 0:
        printVerbose(f"Found {nBoundaries} file boundaries.")

        for i in range(len(f.fileBoundaries)-1):
            start = f.fileBoundaries[i]
            end = f.fileBoundaries[i+1]

            functionsInBoundary = 0
            for func in f.functions:
                funcOffset = func.vram - vram
                if start <= funcOffset < end:
                    functionsInBoundary += 1
            fileVram = 0
            if vram > -1:
                fileVram = start + vram
            printVerbose("\t", toHex(start+offsetStart, 6)[2:], toHex(end-start, 4)[2:], toHex(fileVram, 8)[2:], "\t functions:", functionsInBoundary)

        start = f.fileBoundaries[-1]
        end = f.size + f.offset

        functionsInBoundary = 0
        for func in f.functions:
            funcOffset = func.vram - vram
            if start <= funcOffset < end:
                functionsInBoundary += 1
        fileVram = 0
        if vram > -1:
            fileVram = start + vram
        printVerbose("\t", toHex(start+offsetStart, 6)[2:], toHex(end-start, 4)[2:], toHex(fileVram, 8)[2:], "\t functions:", functionsInBoundary)

        printVerbose()

    return f


def simpleDisasmData(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context, isHandwritten: bool=False, isRsp: bool=False, newStuffSuffix: str=""):
    head, tail = os.path.split(outputPath)

    if offsetEnd >= 0:
        printVerbose(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    if offsetStart >= 0:
        printVerbose(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    f = Data(array_of_bytes, tail, "ver", context)
    f.isHandwritten = isHandwritten
    f.isRsp = isRsp
    f.newStuffSuffix = newStuffSuffix

    if vram >= 0:
        printVerbose(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    printVerbose("Analyzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    printVerbose()

    return f


def simpleDisasmRodata(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context, isHandwritten: bool=False, isRsp: bool=False, newStuffSuffix: str=""):
    head, tail = os.path.split(outputPath)

    if offsetEnd >= 0:
        printVerbose(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    if offsetStart >= 0:
        printVerbose(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    f = Rodata(array_of_bytes, tail, "ver", context)
    f.isHandwritten = isHandwritten
    f.isRsp = isRsp
    f.newStuffSuffix = newStuffSuffix

    if vram >= 0:
        printVerbose(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    printVerbose("Analyzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    printVerbose()

    return f


def simpleDisasmBss(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context, isHandwritten: bool=False, isRsp: bool=False, newStuffSuffix: str=""):
    head, tail = os.path.split(outputPath)

    if vram < 0:
        return

    f = Bss(vram, vram + offsetEnd - offsetStart, tail, "ver", context)
    f.isHandwritten = isHandwritten
    f.isRsp = isRsp
    f.newStuffSuffix = newStuffSuffix

    if vram >= 0:
        printVerbose(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    printVerbose("Analyzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    printVerbose()

    return f


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

    parser.add_argument("--split-functions", help="Enables the function and rodata splitter. Expects a path to place the splited functions", metavar="PATH")

    parser.add_argument("--save-context", help="Saves the context to a file. The provided filename will be suffixed with the corresponding version.", metavar="FILENAME")

    parser.add_argument("--functions", help="Path to a functions csv", action="append")
    parser.add_argument("--variables", help="Path to a variables csv", action="append")
    parser.add_argument("--constants", help="Path to a constants csv", action="append")
    parser.add_argument("--file-splits", help="Path to a file splits csv")

    parser.add_argument("--add-filename", help="Adds the filename of the file to the generated function/variable name")

    parser.add_argument("--disasm-unknown", help="Force disassembly of functions with unknown instructions",  action="store_true")
    parser.add_argument("--disasm-rsp", help="Experimental. Enables the disassembly of rsp abi instructions. Warning: In its current state the generated asm may not be assemblable to a matching binary",  action="store_true")

    parser.add_argument("-v", "--verbose", help="Enable verbose mode",  action="store_true")
    parser.add_argument("-q", "--quiet", help="Silence most output",  action="store_true")

    parser.add_argument("--disable-asm-comments", help="Disables the comments in assembly code", action="store_true")
    parser.add_argument("--write-binary", help="Produce a binary of the processed file", action="store_true")
    parser.add_argument("--nuke-pointers", help="Use every technique available to remove pointers", action="store_true")
    parser.add_argument("--ignore-words", help="A space separated list of hex numbers. Word differences will be ignored that starts in any of the provided arguments. Max value: FF. Only works when --nuke-pointers is passed", action="extend", nargs="+")

    parser.add_argument("--non-libultra", help="Don't use built-in libultra symbols", action="store_true")
    parser.add_argument("--non-hardware-regs", help="Don't use built-in hardware registers symbols", action="store_true")

    parser.add_argument("--disable-string-guesser", help="Disables the string guesser feature (does nto affect the strings referenced by .data)", action="store_true")

    parser.add_argument("--debug-func-analysis", help="Enables some debug info printing related to the function analysis)", action="store_true")
    parser.add_argument("--debug-symbol-finder", help="Enables some debug info printing related to the symbol finder system)", action="store_true")

    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = args.nuke_pointers
    GlobalConfig.IGNORE_BRANCHES = args.nuke_pointers
    if args.nuke_pointers:
        GlobalConfig.IGNORE_WORD_LIST.add(0x80)
    if args.ignore_words:
        for upperByte in args.ignore_words:
            GlobalConfig.IGNORE_WORD_LIST.add(int(upperByte, 16))
    GlobalConfig.WRITE_BINARY = args.write_binary
    GlobalConfig.ASM_COMMENT = not args.disable_asm_comments
    GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True
    GlobalConfig.TRUST_USER_FUNCTIONS = True
    GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS = args.disasm_unknown
    GlobalConfig.DISASSEMBLE_RSP = args.disasm_rsp
    # TODO: remove? add flag?
    GlobalConfig.STRING_GUESSER = not args.disable_string_guesser
    GlobalConfig.VERBOSE = args.verbose
    GlobalConfig.QUIET = args.quiet
    GlobalConfig.PRINT_FUNCTION_ANALYSIS_DEBUG_INFO = args.debug_func_analysis
    GlobalConfig.PRINT_SYMBOL_FINDER_DEBUG_INFO = args.debug_symbol_finder

    newStuffSuffix = args.add_filename
    if newStuffSuffix is None:
        newStuffSuffix = ""

    context = Context()
    context.fillDefaultBannedSymbols()
    if not args.non_libultra:
        context.fillLibultraSymbols()
    if not args.non_hardware_regs:
        context.fillHardwareRegs()

    if args.functions is not None:
        for funcsPath in args.functions:
            context.readFunctionsCsv(funcsPath)
    if args.variables is not None:
        for varsPath in args.variables:
            context.readVariablesCsv(varsPath)
    if args.constants is not None:
        for constantsPath in args.constants:
            context.readConstantsCsv(constantsPath)

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
        f =  simpleDisasmFile(array_of_bytes, args.output, int(args.start, 16), int(args.end, 16), int(args.vram, 16), context, False, GlobalConfig.DISASSEMBLE_RSP, newStuffSuffix)
        processedFiles[FileSectionType.Text].append((args.output, f))
    else:
        splits = FileSplitFormat(args.file_splits)

        splitsCount = len(splits)

        textOutput = args.output
        dataOutput = args.data_output
        if dataOutput is None:
            dataOutput = textOutput

        modeCallback = None
        outputPath = args.output
        i = 0
        for row in splits:
            offset, vram, fileName, section, nextOffset, isHandwritten, isRsp = row

            if section == FileSectionType.Text:
                modeCallback = simpleDisasmFile
                outputPath = textOutput
            elif section == FileSectionType.Data:
                modeCallback = simpleDisasmData
                outputPath = dataOutput
            elif section == FileSectionType.Rodata:
                modeCallback = simpleDisasmRodata
                outputPath = dataOutput
            elif section == FileSectionType.Bss:
                modeCallback = simpleDisasmBss
                outputPath = dataOutput

            if fileName == "":
                fileName = f"{input_name}_{vram:08X}"

            if modeCallback is None:
                eprint("Error! Section not set!")
                exit(1)
            printVerbose(f"Reading '{fileName}'")
            f = modeCallback(array_of_bytes, f"{outputPath}/{fileName}", offset, nextOffset, vram, context, isHandwritten, isRsp, newStuffSuffix)
            processedFiles[section].append((f"{outputPath}/{fileName}", f))

            printQuietless(lenLastLine*" " + "\r", end="")
            progressStr = f" Analyzing: {i/splitsCount:%}. File: {fileName}\r"
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

    printVerbose("Spliting functions")
    if args.split_functions is not None:
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
