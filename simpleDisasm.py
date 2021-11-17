#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsText import Text
from mips.MipsData import Data
from mips.MipsRodata import Rodata
from mips.MipsBss import Bss
from mips.MipsContext import Context


def simpleDisasmFile(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context):
    head, tail = os.path.split(outputPath)

    if offsetEnd >= 0:
        print(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    if offsetStart >= 0:
        print(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    f = Text(array_of_bytes, tail, "ver", context)

    if vram >= 0:
        print(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    print("Analzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    print()
    print(f"Found {f.nFuncs} functions.")

    nBoundaries = len(f.fileBoundaries)
    if nBoundaries > 0:
        print(f"Found {nBoundaries} file boundaries.")

        for i in range(len(f.fileBoundaries)-1):
            start = f.fileBoundaries[i]
            end = f.fileBoundaries[i+1]

            functionsInBoundary = 0
            for func in f.functions:
                funcOffset = func.vram - vram
                if start <= funcOffset < end:
                    functionsInBoundary += 1
            print("\t", toHex(start, 6)[2:], toHex(end-start, 3)[2:], "\t functions:", functionsInBoundary)


        start = f.fileBoundaries[-1]
        end = f.size + f.offset

        functionsInBoundary = 0
        for func in f.functions:
            funcOffset = func.vram - vram
            if start <= funcOffset < end:
                functionsInBoundary += 1
        print("\t", toHex(start, 6)[2:], toHex(end-start, 3)[2:], "\t functions:", functionsInBoundary)

        print()

    return f


def simpleDisasmData(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context):
    head, tail = os.path.split(outputPath)

    if offsetEnd >= 0:
        print(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    if offsetStart >= 0:
        print(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    f = Data(array_of_bytes, tail, "ver", context)

    if vram >= 0:
        print(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    print("Analzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    print()

    return f


def simpleDisasmRodata(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context):
    head, tail = os.path.split(outputPath)

    if offsetEnd >= 0:
        print(f"Parsing until offset {toHex(offsetEnd, 2)}")
        array_of_bytes = array_of_bytes[:offsetEnd]
    if offsetStart >= 0:
        print(f"Parsing since offset {toHex(offsetStart, 2)}")
        array_of_bytes = array_of_bytes[offsetStart:]

    f = Rodata(array_of_bytes, tail, "ver", context)

    if vram >= 0:
        print(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    print("Analzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    print()

    return f


def simpleDisasmBss(array_of_bytes: bytearray, outputPath: str, offsetStart: int, offsetEnd: int, vram: int, context: Context):
    head, tail = os.path.split(outputPath)

    if vram < 0:
        return

    f = Bss(vram, vram + offsetEnd - offsetStart, tail, "ver", context)

    if vram >= 0:
        print(f"Using VRAM {toHex(vram, 2)}")
        f.setVRamStart(vram)

    print("Analzing")
    f.analyze()
    f.setCommentOffset(offsetStart)

    print()

    return f


def disassemblerMain():
    description = ""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output")
    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")
    parser.add_argument("--start", help="", default="0")
    parser.add_argument("--end", help="",  default="0xFFFFFF")
    parser.add_argument("--vram", help="Set the VRAM address", default="-1")
    parser.add_argument("--save-context", help="Saves the context to a file. The provided filename will be suffixed with the corresponding version.", metavar="FILENAME")
    parser.add_argument("--functions", help="Path to a functions csv")
    parser.add_argument("--variables", help="Path to a variables csv")
    parser.add_argument("--file-splits", help="Path to a file splits csv")
    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = False
    GlobalConfig.IGNORE_BRANCHES = False
    GlobalConfig.IGNORE_04 = False
    GlobalConfig.IGNORE_06 = False
    GlobalConfig.IGNORE_80 = False
    GlobalConfig.WRITE_BINARY = False
    GlobalConfig.ASM_COMMENT = True
    GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True

    context = Context()
    if args.functions is not None:
        context.readFunctionsCsv(args.functions)
    if args.variables is not None:
        context.readVariablesCsv(args.variables)

    array_of_bytes = readFileAsBytearray(args.binary)
    input_name = os.path.splitext(os.path.split(args.binary)[1])[0]

    processedFiles = []

    if args.file_splits is None:
        f =  simpleDisasmFile(array_of_bytes, args.output, int(args.start, 16), int(args.end, 16), int(args.vram, 16), context)
        processedFiles.append((args.output, f))
    else:
        splits = readCsv(args.file_splits)

        splits = [x for x in splits if len(x) > 0]

        modeCallback = None
        outputPath = args.output
        for i, row in enumerate(splits):
            offset, vram, fileName = row
            if fileName == ".text":
                modeCallback = simpleDisasmFile
                outputPath = args.output
                continue
            elif fileName == ".data":
                modeCallback = simpleDisasmData
                outputPath = args.data_output
                continue
            elif fileName == ".rodata":
                modeCallback = simpleDisasmRodata
                outputPath = args.data_output
                continue
            elif fileName == ".bss":
                modeCallback = simpleDisasmBss
                outputPath = args.data_output
                continue
            elif fileName == ".end":
                break

            vram = int(vram, 16)
            offset = int(offset, 16)
            nextOffset = 0xFFFFFF
            if i + 1 < len(splits):
                if splits[i+1][2] == ".end":
                    nextOffset = int(splits[i+1][0], 16)
                elif splits[i+1][2].startswith("."):
                    nextOffset = int(splits[i+2][0], 16)
                else:
                    nextOffset = int(splits[i+1][0], 16)

            if fileName == "":
                fileName = f"{input_name}_{vram:08X}"

            if modeCallback is None:
                eprint("Error! Section not set!")
                exit(1)
            f = modeCallback(array_of_bytes, f"{outputPath}/{fileName}", offset, nextOffset, vram, context)
            processedFiles.append((f"{outputPath}/{fileName}", f))
            print()

    print("Writing files...")
    for path, f in processedFiles:
        head, tail = os.path.split(path)

        # Create directories
        os.makedirs(head, exist_ok=True)

        print(f"Writing {path}")
        f.saveToFile(path)

    if args.save_context is not None:
        head, tail = os.path.split(args.save_context)
        os.makedirs(head, exist_ok=True)
        name = tail
        extension = ""
        if "." in tail:
            *aux, extension = tail.split(".")
            name = ".".join(aux)
            extension = "." + extension
        name = os.path.join(head, name)
        context.saveContextToFile(f"{name}_{extension}")

    print()
    print("Disassembling complete!")
    print("Goodbye.")


if __name__ == "__main__":
    disassemblerMain()
