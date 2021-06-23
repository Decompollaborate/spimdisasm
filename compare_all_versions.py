#!/usr/bin/python3

from __future__ import annotations

import argparse
import os
from typing import List, Dict
from multiprocessing import Pool, cpu_count
from functools import partial

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsFile import File
from mips.MipsFileGeneric import FileGeneric
from mips.MipsFileOverlay import FileOverlay
from mips.MipsFileCode import FileCode
from mips.MipsFileBoot import FileBoot
from mips.ZeldaTables import OverlayTableEntry, getDmaAddresses, DmaEntry
from mips import ZeldaOffsets


def countUnique(row: list) -> int:
    unique = set(row)
    count = len(unique)
    if "" in unique:
        count -= 1
    return count

def removePointers(args, filedata: bytearray) -> bytearray:
    if args.dont_remove_ptrs:
        return filedata
    if not args.ignore04: # This will probably grow...
        return filedata

    words = bytesToBEWords(filedata)
    for i in range(len(words)):
        w = words[i]
        if args.ignore04:
            if ((w >> 24) & 0xFF) == 0x04:
                words[i] = 0x04000000
    return beWordsToBytes(words, filedata)


def getHashesOfFiles(args, filesPath: List[str]) -> List[str]:
    hashList = []
    for path in filesPath:
        f = readFileAsBytearray(path)
        if len(f) != 0:
            fHash = getStrHash(removePointers(args, f))
            line = fHash + " " + path # To be consistent with runCommandGetOutput("md5sum", md5arglist)
            hashList.append(line)
    return hashList

def compareFileAcrossVersions(filename: str, versionsList: List[str], dmaAddresses: dict, actorOverlayTable: dict, args) -> List[List[str]]:
    md5arglist = list(map(lambda orig_string: "baserom_" + orig_string + "/" + filename, versionsList))
    # os.system( "md5sum " + " ".join(filesPath) )

    # Get hashes.
    # output = runCommandGetOutput("md5sum", filesPath)
    output = getHashesOfFiles(args, md5arglist)

    # Print md5hash
    #print("\n".join(output))
    #print()

    filesHashes = dict() # "NN0": "339614255f179a1e308d954d8f7ffc0a"
    firstFilePerHash = dict() # "339614255f179a1e308d954d8f7ffc0a": "NN0"

    for line in output:
        trimmed = removeExtraWhitespace(line)
        filehash, filepath = trimmed.split(" ")
        abbr = ZeldaOffsets.getVersionAbbr(filepath)

        # Map each abbreviation and its hash.
        filesHashes[abbr] = filehash

        # Find out where in which version this hash appeared for first time.
        if filehash not in firstFilePerHash:
            firstFilePerHash[filehash] = abbr

    row = [filename]
    for ver in versionsList:
        abbr = ZeldaOffsets.versions.get(ver, None)

        if abbr in filesHashes:
            fHash = filesHashes[abbr]
            row.append(firstFilePerHash[fHash])
        else:
            row.append("")
    return [row]

def compareOverlayAcrossVersions(filename: str, versionsList: List[str], dmaAddresses: Dict[str, Dict[str, DmaEntry]], actorOverlayTable: Dict[str, List[OverlayTableEntry]], args) -> List[List[str]]:
    column = []
    filesHashes = dict() # "filename": {"NN0": hash}
    firstFilePerHash = dict() # "filename": {hash: "NN0"}

    if filename.startswith("#"):
        return column

    is_overlay = filename.startswith("ovl_")
    is_code = filename == "code"
    is_boot = filename == "boot"

    for version in versionsList:
        path = os.path.join("baserom_" + version, filename)

        array_of_bytes = readFileAsBytearray(path)
        if len(array_of_bytes) == 0:
            continue

        if is_overlay:
            tableEntry = None
            if version in dmaAddresses:
                versionData = dmaAddresses[version]
                if filename in versionData:
                    dmaData = versionData[filename]
                    if version in actorOverlayTable:
                        for entry in actorOverlayTable[version]:
                            if entry.vromStart == dmaData.vromStart:
                                tableEntry = entry
                                break

            f = FileOverlay(array_of_bytes, filename, version, tableEntry=tableEntry)
        elif is_code:
            f = FileCode(array_of_bytes, filename, version)
        elif is_boot:
            f = FileBoot(array_of_bytes, filename, version)
        else:
            f = File(array_of_bytes, filename, version)

        if GlobalConfig.REMOVE_POINTERS:
            was_updated = f.removePointers()
            if was_updated:
                f.updateBytes()

        if args.savetofile:
            new_file_path = os.path.join(args.savetofile, version, filename, filename)
            f.saveToFile(new_file_path)

        abbr = ZeldaOffsets.getVersionAbbr(path)

        if isinstance(f, FileOverlay):
            subfiles = {
                ".text" : f.text,
                ".data" : f.data,
                ".rodata" : f.rodata,
                #".bss" : f.bss,
                #".reloc" : f.reloc,
            }
        elif isinstance(f, FileGeneric):
            subfiles = {
                ".text" : f.text,
                ".data" : f.data,
                ".rodata" : f.rodata,
                #".bss" : f.bss,
            }
        else:
            subfiles = {
                "" : f,
            }

        for section, sub in subfiles.items():
            file_section = filename + section
            if file_section not in filesHashes:
                filesHashes[file_section] = dict()
                firstFilePerHash[file_section] = dict()

            f_hash = sub.getHash()
            # Map each abbreviation to its hash.
            filesHashes[file_section][abbr] = f_hash

            # Find out where in which version this hash appeared for first time.
            if f_hash not in firstFilePerHash[file_section]:
                firstFilePerHash[file_section][f_hash] = abbr

    for file_section in filesHashes:
        row = [file_section]
        for version in versionsList:
            abbr = ZeldaOffsets.versions.get(version)

            if abbr in filesHashes[file_section]:
                fHash = filesHashes[file_section][abbr]
                row.append(firstFilePerHash[file_section][fHash])
            else:
                row.append("")
        column.append(row)

    return column


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("versionlist", help="Path to version list.")
    parser.add_argument("filelist", help="List of filenames of the ROM that will be compared.")
    parser.add_argument("--noheader", help="Disables the csv header.", action="store_true")
    # parser.add_argument("--overlays", help="Treats the files in filelist as overlays.", action="store_true")
    parser.add_argument("--savetofile", help="Specify a folder where each part of an overlay will be written.", metavar="FOLDER")
    parser.add_argument("--ignore04", help="Ignores words starting with 0x04.", action="store_true")
    parser.add_argument("--track-registers", help="Set for how many instructions a register will be tracked.", type=int)
    parser.add_argument("--ignore-branches", help="Ignores the address of every branch, jump and jal.", action="store_true")
    parser.add_argument("--dont-remove-ptrs", help="Disable the pointer removal feature.", action="store_true")
    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = not args.dont_remove_ptrs
    GlobalConfig.IGNORE_BRANCHES = args.ignore_branches
    if args.track_registers is not None:
        GlobalConfig.TRACK_REGISTERS = args.track_registers
    GlobalConfig.IGNORE_04 = args.ignore04

    # Read filelist
    versionsList = []
    with open(args.versionlist) as f:
        for version in f:
            if version.startswith("#"):
                continue
            versionsList.append(version.strip())
    filesList = readFile(args.filelist)

    if args.savetofile is not None:
        for ver in versionsList:
            for filename in filesList:
                if filename.startswith("#"):
                    continue
                os.makedirs(os.path.join(args.savetofile, ver, filename), exist_ok=True)

    dmaAddresses: Dict[str, Dict[str, DmaEntry]] = dict()
    actorOverlayTable: Dict[str, List[OverlayTableEntry]] = dict()
    for version in versionsList:
        dmaAddresses[version] = getDmaAddresses(version)

        codePath = os.path.join(f"baserom_{version}", "code")

        if os.path.exists(codePath) and version in ZeldaOffsets.offset_ActorOverlayTable:
            tableOffset = ZeldaOffsets.offset_ActorOverlayTable[version]
            if tableOffset != 0x0:
                codeData = readFileAsBytearray(codePath)
                i = 0
                table = list()
                while i < ZeldaOffsets.ACTOR_ID_MAX:
                    entry = OverlayTableEntry(codeData[tableOffset + i*0x20 : tableOffset + (i+1)*0x20])
                    table.append(entry)
                    i += 1
                actorOverlayTable[version] = table

    if not args.noheader:
        # Print csv header
        print("File name", end="")
        for ver in versionsList:
            print("," + ver, end="")
        print(",Different versions", end="")
        print()

    # compareFunction = compareFileAcrossVersions
    # if args.overlays:
    #     compareFunction = compareOverlayAcrossVersions
    compareFunction = compareOverlayAcrossVersions

    numCores = cpu_count()
    p = Pool(numCores)
    for column in p.imap(partial(compareFunction, versionsList=versionsList, dmaAddresses=dmaAddresses, actorOverlayTable=actorOverlayTable, args=args), filesList):
        for row in column:
            # Print csv row
            for cell in row:
                print(cell + ",", end="")
            print(countUnique(row)-1)

if __name__ == "__main__":
    main()
