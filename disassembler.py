#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsText import Text
from mips.MipsFileOverlay import FileOverlay
from mips.MipsFileCode import FileCode
from mips.MipsFileBoot import FileBoot
from mips.MipsContext import Context
from mips.ZeldaTables import DmaEntry, getDmaAddresses

def disassembleFile(version: str, filename: str, outputfolder: str, context: Context, dmaAddresses: Dict[str, DmaEntry]):
    is_overlay = filename.startswith("ovl_")
    is_code = filename == "code"
    is_boot = filename == "boot"

    path = os.path.join(f"baserom_{version}", filename)

    array_of_bytes = readFileAsBytearray(path)
    if len(array_of_bytes) == 0:
        eprint(f"File '{path}' not found!")
        exit(-1)

    if is_overlay:
        tableEntry = None
        # TODO
        #if filename in dmaAddresses:
        #    dmaEntry = dmaAddresses[filename]
            #for entry in actorOverlayTable:
            #    if entry.vromStart == dmaEntry.vromStart:
            #        tableEntry = entry
            #        break

        print("Overlay detected. Parsing...")
        f = FileOverlay(array_of_bytes, filename, version, context, tableEntry=tableEntry)
    elif is_code:
        print("code detected. Parsing...")
        f = FileCode(array_of_bytes, filename, version, context)
    elif is_boot:
        print("boot detected. Parsing...")
        f = FileBoot(array_of_bytes, filename, version, context)
    else:
        print("Unknown file type. Assuming .text. Parsing...")
        f = Text(array_of_bytes, filename, version, context)
        f.findFunctions()

    print()
    print(f"Found {f.nFuncs} functions.")

    new_file_folder = os.path.join(outputfolder, version, filename)
    os.makedirs(new_file_folder, exist_ok=True)
    new_file_path = os.path.join(new_file_folder, filename)

    print(f"Writing files to {new_file_folder}")
    f.saveToFile(new_file_path)

    print()
    print("Disassembling complete!")
    print("Goodbye.")


def disassemblerMain():
    description = ""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("version", help="Select which baserom folder will be used. Example: ique_cn would look up in folder baserom_ique_cn")
    parser.add_argument("file", help="File to be disassembled from the baserom folder.")
    parser.add_argument("outputfolder", help="Path to output folder.")
    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = False
    GlobalConfig.IGNORE_BRANCHES = False
    GlobalConfig.IGNORE_04 = False
    GlobalConfig.IGNORE_06 = False
    GlobalConfig.IGNORE_80 = False
    GlobalConfig.WRITE_BINARY = False

    context = Context()
    context.readFunctionMap(args.version)
    dmaAddresses: Dict[str, DmaEntry] = getDmaAddresses(args.version)

    disassembleFile(args.version, args.file, args.outputfolder, context, dmaAddresses)


if __name__ == "__main__":
    disassemblerMain()
