#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsText import Text, readMipsText
from mips.MipsFileOverlay import FileOverlay
from mips.MipsFileCode import FileCode
from mips.ZeldaTables import DmaEntry, getDmaAddresses

from mips.ZeldaOffsets import codeVramStart, codeDataStart, codeRodataStart

from mips.codeFunctionMapPalMqDbg import codeFunctionMapPalMqDbg
from mips.ZeldaCodeSplits import codeFilesStarts

GlobalConfig.REMOVE_POINTERS = False
GlobalConfig.IGNORE_BRANCHES = False
GlobalConfig.IGNORE_04 = False
GlobalConfig.IGNORE_06 = False
GlobalConfig.IGNORE_80 = False
GlobalConfig.WRITE_BINARY = False


parser = argparse.ArgumentParser()
parser.add_argument("version", help="Select which baserom folder will be used. Example: ique_cn would look up in folder baserom_ique_cn")
args = parser.parse_args()


CODE = "code"
VERSION = args.version

palMqDbg_Code_array = readVersionedFileAsBytearrray(CODE, VERSION)
# remove data
palMqDbg_Code_array = palMqDbg_Code_array[:codeDataStart[VERSION]]

palMqDbg_filesStarts = list()
palMqDbg_filesStarts.append(("start", 0))
for codeFilename, versionsSplits in codeFilesStarts.items():
    start = versionsSplits[VERSION]
    if start < 0:
        continue
    palMqDbg_filesStarts.append((codeFilename, start))
palMqDbg_filesStarts.append(("end", codeDataStart[VERSION]))

palMqDbg_texts: List[Text] = []
i = 0
while i < len(palMqDbg_filesStarts) - 1:
    filename, start = palMqDbg_filesStarts[i]
    _, end = palMqDbg_filesStarts[i+1]

    text = Text(palMqDbg_Code_array[start:end], filename, CODE)
    text.offset = start
    text.vRamStart = codeVramStart[VERSION]

    text.findFunctions()

    palMqDbg_texts.append(text)

    i += 1

for text in palMqDbg_texts:
    print(text.filename)
    print("boundaries:", len(text.fileBoundaries))
    for i in range(len(text.fileBoundaries)-1):
        a = text.fileBoundaries[i]
        b = text.fileBoundaries[i+1]
        #if b-a <= 0x100:
        #    print("\t", toHex(a, 6), toHex(b-a, 4))
        print("\t", toHex(a, 6), toHex(b-a, 4))
    if len(text.fileBoundaries) > 0:
        print("\t", toHex(text.fileBoundaries[-1], 6))


    print("functions:", len(text.functions))
    print()

OUTPUT_FOLDER = "splits"

print(f"Writing files to {OUTPUT_FOLDER}/")
for text in palMqDbg_texts:
    new_file_folder = os.path.join(OUTPUT_FOLDER, VERSION, CODE)
    os.makedirs(new_file_folder, exist_ok=True)
    new_file_path = os.path.join(new_file_folder, text.filename)

    print(f"Writing file {new_file_path}")
    text.saveToFile(new_file_path)
