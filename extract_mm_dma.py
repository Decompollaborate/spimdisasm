#!/usr/bin/python3

from __future__ import annotations

import argparse
import os
import sys
import struct
from typing import Dict, List
import zlib


# ROM_FILE_NAME = 'baserom.z64'
# ROM_FILE_NAME_V = 'baserom_{}.z64'
FILE_TABLE_OFFSET = {
    "MM USA":     0x1A500,
}

FILE_NAMES: Dict[str, List[str] | None] = {
    "MM USA":     None,
}

romData = None
Edition = "" # "pal_mq"
Version = "" # "PAL MQ"


def readFile(filepath):
    with open(filepath) as f:
        return [x.strip() for x in f.readlines()]

def readFilelists():
    FILE_NAMES["MM USA"] = readFile("filelists/mm/filelist_usa.txt")

def initialize_worker(rom_data):
    global romData
    global globalDmaTable
    romData = rom_data

def read_uint32_be(offset):
    return struct.unpack('>I', romData[offset:offset+4])[0]


def ExtractFunc(i):
    verFilename = FILE_NAMES[Version][i]
    if verFilename == "":
        print(f"Skipping {i} because it doesn't have a name.")
        return
        
    entryOffset = FILE_TABLE_OFFSET[Version] + 16 * i

    virtStart = read_uint32_be(entryOffset + 0)
    virtEnd   = read_uint32_be(entryOffset + 4)
    physStart = read_uint32_be(entryOffset + 8)
    physEnd   = read_uint32_be(entryOffset + 12)

    displayedPhysEnd = physEnd
    
    if physEnd == 0:  # uncompressed
        compressed = False
        size = virtEnd - virtStart
        compressString = ""
        actualPhysEnd = physStart + size
        if showEnd:
            displayedPhysEnd = actualPhysEnd
    else:             # compressed
        compressed = True
        size = physEnd - physStart
        compressString = "compressed"

    print(f"{verFilename},{virtStart:X},{virtEnd:X},{physStart:X},{displayedPhysEnd:X},{compressString}")


#####################################################################

def extract_rom(): #(j):
    readFilelists()

    file_names_table = FILE_NAMES[Version]
    if file_names_table is None:
        print(f"'{Edition}' is not supported yet.")
        sys.exit(2)


    filename = RomFile#.format(Edition)
    # exit()
    # if not os.path.exists(filename):
    #     print(f"{filename} not found. Defaulting to {ROM_FILE_NAME}")
    #     filename = ROM_FILE_NAME

    # read baserom data
    try:
        with open(filename, 'rb') as f:
            rom_data = f.read()
    except IOError:
        print('Failed to read file ' + filename)
        sys.exit(1)

    if True:
        initialize_worker(rom_data)#, dmaTable)
        for i in range(len(file_names_table)):
            ExtractFunc(i)


def main():
    description = "Extracts the dmadata table from the rom. Will try to read the rom 'baserom_version.z64', or 'baserom.z64' if that doesn't exists."

    parser = argparse.ArgumentParser(description=description, formatter_class=argparse.RawTextHelpFormatter)
    choices = [x.lower().replace(" ", "_") for x in FILE_TABLE_OFFSET]
    parser.add_argument("edition", help="Select the version of the game to extract", choices=choices, default="mm_usa", nargs='?')
    parser.add_argument("romFile", help="ROM to use", nargs='?')
    # parser.add_argument("-j", help="Enables multiprocessing.", action="store_true")
    parser.add_argument("--show-end", help="Show physical ROM end addresses for uncompressed files", action="store_true")
    args = parser.parse_args()

    global Edition
    global Version
    global RomFile
    global showEnd

    Edition = args.edition
    Version = Edition.upper().replace("_", " ")
    RomFile = args.romFile
    showEnd = args.show_end

    print("File name,VROM start,VROM end,ROM start,ROM end,Compressed?")
    extract_rom()

if __name__ == "__main__":
    main()
