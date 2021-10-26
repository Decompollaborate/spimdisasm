#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsText import Text
from mips.MipsFileGeneric import FileGeneric
from mips.MipsFileOverlay import FileOverlay
from mips.MipsFileCode import FileCode
from mips.MipsFileBoot import FileBoot
from mips.MipsContext import Context
from mips.MipsSplitEntry import readSplitsFromCsv
from mips.ZeldaTables import DmaEntry, getDmaAddresses, OverlayTableEntry
from mips import ZeldaOffsets


def mmDisasmMain():
    description = ""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("filepath", help="File to be disassembled from the baserom folder.")
    #parser.add_argument("outputfolder", help="Path to output folder.")
    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = False
    GlobalConfig.IGNORE_BRANCHES = False
    GlobalConfig.IGNORE_04 = False
    GlobalConfig.IGNORE_06 = False
    GlobalConfig.IGNORE_80 = False
    GlobalConfig.WRITE_BINARY = False
    #GlobalConfig.ASM_COMMENT = not args.disable_asm_comments

    context = Context()
    context.readMMAddressMaps("../mm/tools/disasm/files.txt", "../mm/tools/disasm/functions.txt", "../mm/tools/disasm/variables.txt")

    path = args.filepath
    filename = os.path.split(path)[-1]
    version = "mm_ntsc_usa"
    textend = -1

    array_of_bytes = readFileAsBytearray(path)
    if len(array_of_bytes) == 0:
        eprint(f"File '{path}' not found!")
        exit(-1)

    segment = context.segments[filename]

    if segment.type == "overlay":
        print("Overlay detected. Parsing...")

        f = FileOverlay(array_of_bytes, filename, version, context)
    elif segment.type == "code":
        print("code detected. Parsing...")
        f = FileCode(array_of_bytes, version, context)
    elif segment.type == "boot":
        print("boot detected. Parsing...")
        print("TODO. ABORTING")
        exit(-1)
        f = FileBoot(array_of_bytes, version, context)
    else:
        print("Unknown file type. Assuming .text. Parsing...")
        print("TODO. ABORTING")
        exit(-1)

        text_data = array_of_bytes
        if textend >= 0:
            print(f"Parsing until offset {toHex(textend, 2)}")
            text_data = array_of_bytes[:textend]

        f = Text(text_data, filename, version, context)

    f.analyze()

    print()
    print(f"Found {f.nFuncs} functions.")

    new_file_folder = "asm/mm_ntsc_usa"
    print(f"Writing files to {new_file_folder}")
    new_file_path = f"{new_file_folder}/{filename}/"
    f.saveToFile(new_file_path)

if __name__ == "__main__":
    mmDisasmMain()
