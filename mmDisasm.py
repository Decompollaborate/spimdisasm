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
    GlobalConfig.ADD_NEW_SYMBOLS = False
    GlobalConfig.PRODUCE_SYMBOLS_PLUS_OFFSET = True

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
        subfolder = "overlays"
    elif segment.type == "code":
        print("code detected. Parsing...")
        print("TODO. ABORTING")
        exit(-1)
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

        #text_data = array_of_bytes
        #if textend >= 0:
        #    print(f"Parsing until offset {toHex(textend, 2)}")
        #    text_data = array_of_bytes[:textend]

        #f = Text(text_data, filename, version, context)

    f.analyze()

    print()
    print(f"Found {f.nFuncs} functions.")

    new_file_path = os.path.join("asm", subfolder, filename)
    print(f"Writing files to {new_file_path}")
    os.makedirs(new_file_path, exist_ok=True)
    for name, section in f.textList.items():
        section.saveToFile(os.path.join(new_file_path, name))

    new_file_path = os.path.join("asm", "non_matchings", subfolder, filename)
    print(f"Writing files to {new_file_path}")
    os.makedirs(new_file_path, exist_ok=True)
    for name, section in f.textList.items():
        for func in section.functions:
            #print(func.name)
            with open(os.path.join(new_file_path, func.name) + ".s", "w") as file:
                #wasRodataSectionNameWrote = False
                #hasRodata = False
                rodata_stuff = []
                rodataLen = 0
                firstRodata = None
                if name in f.rodataList:
                    rodata = f.rodataList[name]
                    intersection = func.referencedVRams & rodata.symbolsVRams
                    if intersection:
                        sortedSymbolVRams = sorted(rodata.symbolsVRams)
                        #print(func.name, intersection)

                        for vram in sorted(intersection):
                            #print(hex(vram))
                            nextVramIndex = sortedSymbolVRams.index(vram) + 1
                            nextVram = float("inf") if nextVramIndex >= len(sortedSymbolVRams) else sortedSymbolVRams[nextVramIndex]

                            #if not wasRodataSectionNameWrote:
                            #    file.write(".late_rodata\n")
                            #    wasRodataSectionNameWrote = True

                            #file.write(f"glabel {context.getGenericSymbol(vram, tryPlusOffset=False)}\n")
                            for i in range(len(rodata.words)):
                                rodataVram = rodata.getVramOffset(i*4)
                                if rodataVram < vram:
                                    continue
                                if rodataVram >= nextVram:
                                    break

                                if firstRodata is None:
                                    firstRodata = rodata.vRamStart

                                #file.write(rodata.getNthWord(i))
                                #file.write("\n")
                                rodata_stuff.append(rodata.getNthWord(i))
                                rodata_stuff.append("\n")
                                rodataLen += 1

                            #file.write("\n")
                            rodata_stuff.append("\n")

                #print(func.referencedVRams)

                #if wasRodataSectionNameWrote:
                #    file.write(".text\n")
                if len(rodata_stuff) > 0:
                    file.write(".late_rodata\n")
                    if rodataLen / len(func.instructions) > 1/3:
                        align = 4
                        if firstRodata is not None:
                            if firstRodata % 8 == 0:
                                align = 8
                        file.write(".late_rodata_alignment {align}\n")
                    for x in rodata_stuff:
                        file.write(x)
                file.write(func.disassemble())
        #section.saveToFile(os.path.join(new_file_path, name))

    new_file_path = os.path.join("data", filename)
    print(f"Writing files to {new_file_path}")
    os.makedirs(new_file_path, exist_ok=True)
    for name, section in f.dataList.items():
        section.saveToFile(os.path.join(new_file_path, name))
    for name, section in f.rodataList.items():
        section.saveToFile(os.path.join(new_file_path, name))
    for name, section in f.bssList.items():
        section.saveToFile(os.path.join(new_file_path, name))
    if isinstance(f, FileOverlay):
        f.reloc.saveToFile(os.path.join(new_file_path, f.reloc.filename))

if __name__ == "__main__":
    mmDisasmMain()
