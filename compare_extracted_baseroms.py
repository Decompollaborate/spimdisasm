#!/usr/bin/python3

from __future__ import annotations

import argparse
import os
from typing import List

from mips.Utils import *
from mips.MipsInstructions import Instruction

script_dir = os.path.dirname(os.path.realpath(__file__))
root_dir = script_dir + "/.."
if not script_dir.endswith("/tools"):
    root_dir = script_dir
baserom_path = root_dir + "/baserom_"


class File:
    def __init__(self, array_of_bytes: bytearray):
        self.bytes: bytearray = array_of_bytes
        self.words: List[int] = bytesToBEWords(self.bytes)

    @property
    def size(self):
        return len(self.bytes)
    @property
    def sizew(self):
        return len(self.words)

    def getHash(self):
        return get_str_hash(self.bytes)

    def compareToFile(self, other_file: File, args):
        hash_one = self.getHash()
        hash_two = other_file.getHash()

        result = {
            "equal": hash_one == hash_two,
            "hash_one": hash_one,
            "hash_two": hash_two,
            "size_one": self.size,
            "size_two": other_file.size,
            "diff_bytes": 0,
            "diff_words": 0,
        }

        if not result["equal"]:
            min_len = min(self.size, other_file.size)
            for i in range(min_len):
                if self.bytes[i] != other_file.bytes[i]:
                    result["diff_bytes"] += 1

            min_len = min(self.sizew, other_file.sizew)
            for i in range(min_len):
                if self.words[i] != other_file.words[i]:
                    result["diff_words"] += 1
                    #if isinstance(self, Text) and isinstance(other_file, Text):
                        #eprint(f"Differing instruction: {self.instructions[i]}")
                        #eprint(f"Differing instruction: {other_file.instructions[i]}")
                        #eprint(f"")
                        #pass

        return result

    def blankOutDifferences(self, other: File, args):
        was_updated = False
        if args.ignore80 or args.ignore06 or args.ignore04:
            min_len = min(self.sizew, other.sizew)
            for i in range(min_len):
                if args.ignore80:
                    if ((self.words[i] >> 24) & 0xFF) == 0x80 and ((other.words[i] >> 24) & 0xFF) == 0x80:
                        self.words[i] = 0x80000000
                        other.words[i] = 0x80000000
                        was_updated = True
                if args.ignore06:
                    if ((self.words[i] >> 24) & 0xFF) == 0x06 and ((other.words[i] >> 24) & 0xFF) == 0x06:
                        self.words[i] = 0x06000000
                        other.words[i] = 0x06000000
                        was_updated = True
                if args.ignore04:
                    if ((self.words[i] >> 24) & 0xFF) == 0x04 and ((other.words[i] >> 24) & 0xFF) == 0x04:
                        self.words[i] = 0x04000000
                        other.words[i] = 0x04000000
                        was_updated = True
        if was_updated:
            self.updateBytes()
            other.updateBytes()

    def updateBytes(self):
        beWordsToBytes(self.words, self.bytes)


class Text(File):
    def __init__(self, array_of_bytes):
        super().__init__(array_of_bytes)

        self.instructions: List[Instruction] = list()
        for word in self.words:
            self.instructions.append(Instruction(word))

    @property
    def nInstr(self):
        return len(self.instructions)

    def compareToFile(self, other: File, args):
        result = super().compareToFile(other, args)

        if isinstance(other, Text):
            result["text"] = {
                "diff_opcode": self.countDiffOpcodes(other),
                "same_opcode_same_args": self.countSameOpcodeButDifferentArguments(other),
            }

        return result

    def countDiffOpcodes(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            if not self.instructions[i].sameOpcode(other.instructions[i]):
                result += 1
        return result

    def countSameOpcodeButDifferentArguments(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                result += 1
        return result

    def blankOutDifferences(self, other_file: File, args):
        super().blankOutDifferences(other_file, args)
        if not isinstance(other_file, Text):
            return

        was_updated = False

        lui_found = False
        lui_pos = 0
        lui_1_register = 0
        lui_2_register = 0

        for i in range(min(self.nInstr, other_file.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other_file.instructions[i]
            if args.ignore_branches:
                if instr1.isBranch() and instr2.isBranch() and instr1.sameOpcode(instr2):
                    instr1.blankOut()
                    instr2.blankOut()
                    was_updated = True

            #if (instr1.isADDIU() or instr1.isSB() or instr1.isSW() or instr1.isLWCz() 
            #    or instr1.isLBU() or instr1.isLH() or instr1.isLW() or instr1.isSWCz() 
            #    or instr1.isLHU() or instr1.isSH() or instr1.isLB() or instr1.isLUI()
            #    or instr1.isLDCz()):
            #    if instr1.sameOpcode(instr2) and instr1.sameBaseRegister(instr2) and instr1.rt == instr2.rt:
            #        if abs(instr1.immediate - instr2.immediate) == 0x10:
            #            instr1.blankOut()
            #            instr2.blankOut()

            if not lui_found:
                if instr1.isLUI() and instr2.isLUI():
                    lui_found = True
                    lui_pos = i
                    lui_1_register = instr1.rt
                    lui_2_register = instr2.rt
            else:
                if instr1.isADDIU() and instr2.isADDIU():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
                elif instr1.isLW() and instr2.isLW():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
                elif instr1.isLWCz() and instr2.isLWCz():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
                elif instr1.isORI() and instr2.isORI():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
            if i > lui_pos + 5:
                lui_found = False

        if was_updated:
            self.updateWords()
            other_file.updateWords()

    def updateWords(self):
        self.words = []
        for instr in self.instructions:
            self.words.append(instr.instr)
        self.updateBytes()


class RelocEntry:
    def __init__(self, entry: int):
        self.sectionId = entry >> 30
        self.relocType = (entry >> 24) & 0x3F
        self.offset = entry & 0x00FFFFFF

class Reloc(File):
    def __init__(self, array_of_bytes):
        super().__init__(array_of_bytes)

        self.entries: List[RelocEntry] = list()
        for word in self.words:
            self.entries.append(RelocEntry(word))

    @property
    def nRelocs(self):
        return len(self.entries)

    def compareToFile(self, other_file: File, args):
        result = super().compareToFile(other_file, args)
        # TODO
        return result

class Overlay(File):
    def __init__(self, array_of_bytes):
        super().__init__(array_of_bytes)

        seekup = self.words[-1]
        self.headerBPos = self.size - seekup
        self.headerWPos = self.headerBPos//4

        text_size = self.words[self.headerWPos]
        data_size = self.words[self.headerWPos+1]
        rodata_size = self.words[self.headerWPos+2]
        bss_size = self.words[self.headerWPos+3]
        header_size = 4*5
        reloc_size = 4*self.words[self.headerWPos+4]

        start = 0
        end = text_size
        self.text = Text(self.bytes[start:end])

        start += text_size
        end += data_size
        self.data = File(self.bytes[start:end])

        start += data_size
        end += rodata_size
        self.rodata = File(self.bytes[start:end])

        start += rodata_size
        end += bss_size
        self.bss = File(self.bytes[start:end])

        start += bss_size
        end += header_size
        self.header = bytesToBEWords(self.bytes[start:end])

        start += header_size
        end += reloc_size
        self.reloc = Reloc(self.bytes[start:end])

        self.tail = bytesToBEWords(self.bytes[end:])


    def compareToFile(self, other_file: File, args):
        result = super().compareToFile(other_file, args)

        if isinstance(other_file, Overlay):
            result["ovl"] = {
                "text": self.text.compareToFile(other_file.text, args),
                "data": self.data.compareToFile(other_file.data, args),
                "rodata": self.rodata.compareToFile(other_file.rodata, args),
                "bss": self.bss.compareToFile(other_file.bss, args),
                "reloc": self.reloc.compareToFile(other_file.reloc, args),
            }

        return result

    def blankOutDifferences(self, other_file: File, args):
        super().blankOutDifferences(other_file, args)
        if not isinstance(other_file, Overlay):
            return

        self.text.blankOutDifferences(other_file.text, args)

        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words + self.header + self.reloc.words + self.tail
        self.updateBytes()
        
        other_file.words = other_file.text.words + other_file.data.words  + other_file.rodata.words + other_file.bss.words + other_file.header + other_file.reloc.words + other_file.tail
        other_file.updateBytes()

    def relocate(self, allocatedVRamAddress: int, vRamAddress: int):
        allocu32 = allocatedVRamAddress

        sections = [0, 0, 0, 0]
        sections[0] = 0
        sections[1] = allocu32
        sections[2] = allocu32 + self.text.size
        sections[3] = sections[2] + self.data.size

        for reloc in self.reloc.entries:
            relocDataPtr = reloc.sectionId
            # relocData = *relocDataPtr
            if reloc.relocType == 2:
                pass
            elif reloc.relocType == 4:
                pass
            elif reloc.relocType == 5:
                pass
            elif reloc.relocType == 6:
                pass


def print_result_different(comparison, indentation=0):
    if comparison['size_one'] != comparison['size_two']:
        div = round(comparison['size_two']/comparison['size_one'], 3)
        print((indentation * "\t") + f"Size doesn't match: {comparison['size_one']} vs {comparison['size_two']} (x{div}) ({comparison['size_two'] - comparison['size_one']})")
    else:
        print((indentation * "\t") + "Size matches.")
    print((indentation * "\t") + f"There are at least {comparison['diff_bytes']} bytes different, and {comparison['diff_words']} words different.")

def compare_baseroms(args, filelist):
    missing_in_one = set()
    missing_in_two = set()

    equals = 0
    different = 0

    for filename in filelist:
        filepath_one = os.path.join(baserom_path + args.version1, filename)
        filepath_two = os.path.join(baserom_path + args.version2, filename)

        if not os.path.exists(filepath_one):
            missing_in_one.add(filename)
            if args.print in ("all", "missing"):
                print(f"File {filename} does not exists in baserom.")
            continue

        if not os.path.exists(filepath_two):
            missing_in_two.add(filename)
            if args.print in ("all", "missing"):
                print(f"File {filename} does not exists in other_baserom.")
            continue

        file_one_data = read_file_as_bytearray(filepath_one)
        file_two_data = read_file_as_bytearray(filepath_two)

        if filename.startswith("ovl_"):
            file_one = Overlay(file_one_data)
            file_two = Overlay(file_two_data)
        else:
            file_one = File(file_one_data)
            file_two = File(file_two_data)

        file_one.blankOutDifferences(file_two, args)

        comparison = file_one.compareToFile(file_two, args)

        if comparison["equal"]:
            equals += 1
            if args.print in ("all", "equals"):
                print(f"{filename} OK")
        else:
            different += 1
            if args.print in ("all", "diffs"):
                print(f"{filename} not OK")
                print_result_different(comparison, 1)
                
                if "ovl" in comparison:
                    for section_name in comparison["ovl"]:
                        section = comparison["ovl"][section_name]

                        if section["size_one"] == 0:
                            continue        

                        if section["equal"] and args.print in ("all", "equals"):
                            print(f"\t\t{section_name} OK")
                        else:
                            print(f"\t\t{section_name} not OK")
                            print_result_different(section, 3)

    total = len(filelist)
    if total > 0:
        print()
        if args.print in ("all", "equals"):
            print(f"Equals:     {equals}/{total} ({round(100*equals/total, 2)}%)")
        if args.print in ("all", "diffs"):
            print(f"Differents: {different}/{total} ({round(100*different/total, 2)}%)")
        if args.print in ("all", "missing"):
            missing = len(missing_in_one)
            print(f"Missing:    {missing}/{total} ({round(100*missing/total, 2)}%)")
            print(f"Missing 2:  {len(missing_in_two)}")

def compare_to_csv(args, filelist):
    index = -1

    column1 = args.version1 if args.column1 is None else args.column1
    column2 = args.version2 if args.column2 is None else args.column2

    print(f"Index,File,Are equals,Size in {column1},Size in {column2},Size proportion,Size difference,Bytes different,Words different", end="")
    if args.overlays:
        print(",Opcodes difference,Same opcode but different arguments", end="")
    print(flush=True)

    for filename in filelist:
        filepath_one = os.path.join(baserom_path + args.version1, filename)
        filepath_two = os.path.join(baserom_path + args.version2, filename)

        index += 1

        #if args.filetype != "all" and args.filetype != filedata["type"]:
        #    continue

        file_one_data = read_file_as_bytearray(filepath_one)
        file_two_data = read_file_as_bytearray(filepath_two)

        equal = ""
        len_one = ""
        len_two = ""
        div = ""
        size_difference = ""
        diff_bytes = ""
        diff_words = ""
        comparison = dict()

        is_missing_in_one = len(file_one_data) == 0
        is_missing_in_two = len(file_two_data) == 0

        if is_missing_in_one or is_missing_in_two:
            if args.print not in ("all", "missing"):
                continue
            len_one = "" if is_missing_in_one else len(file_one_data)
            len_two = "" if is_missing_in_two else len(file_two_data)

        else:
            if filename.startswith("ovl_"):
                file_one = Overlay(file_one_data)
                file_two = Overlay(file_two_data)
            else:
                file_one = File(file_one_data)
                file_two = File(file_two_data)

            file_one.blankOutDifferences(file_two, args)

            comparison = file_one.compareToFile(file_two, args)
            equal = comparison["equal"]

            if equal and args.print not in ("all", "equals"):
                continue
            if not equal and args.print not in ("all", "diffs"):
                continue
            len_one = comparison["size_one"]
            len_two = comparison["size_two"]
            if len_one > 0:
                div = round(len_two/len_one, 3)
            else:
                div = "Inf"
            size_difference = len_two - len_one
            diff_bytes = comparison["diff_bytes"]
            diff_words = comparison["diff_words"]

        if args.overlays and len(comparison) > 0 and "ovl" in comparison:
            for section_name in comparison["ovl"]:
                section = comparison["ovl"][section_name]
                equal = section["equal"]

                if equal and args.print not in ("all", "equals"):
                    continue
                if not equal and args.print not in ("all", "diffs"):
                    continue

                len_one = section["size_one"]
                len_two = section["size_two"]
                if len_one > 0 or len_two > 0:
                    if len_one > 0:
                        div = round(len_two/len_one, 3)
                    else:
                        div = "Inf"
                    size_difference = len_two - len_one
                    diff_bytes = section["diff_bytes"]
                    diff_words = section["diff_words"]
                    print(f'{index},{filename} {section_name},{equal},{len_one},{len_two},{div},{size_difference},{diff_bytes},{diff_words}', end="")
                    if "text" in section:
                        print(f',{section["text"]["diff_opcode"]},{section["text"]["same_opcode_same_args"]}', end="")
                    else:
                        print(",,", end="")
                    print()
        else:
            print(f'{index},{filename},{equal},{len_one},{len_two},{div},{size_difference},{diff_bytes},{diff_words}', end="")
            if args.overlays:
                print(",,", end="")
            print()


def main():
    description = ""

    epilog = """\
    """
    parser = argparse.ArgumentParser(description=description, epilog=epilog, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument("version1", help="A version of the game to compare. The files will be read from baserom_version1. For example: baserom_pal_mq_dbg")
    parser.add_argument("version2", help="A version of the game to compare. The files will be read from baserom_version2. For example: baserom_pal_mq")
    parser.add_argument("filelist", help="Path to the filelist that will be used.")
    parser.add_argument("--print", help="Select what will be printed for a cleaner output. Default is 'all'.", choices=["all", "equals", "diffs", "missing"], default="all")
    # parser.add_argument("--filetype", help="Filters by filetype. Default: all",  choices=["all", "Unknown", "Overlay", "Object", "Texture", "Room", "Scene", "Other"], default="all")
    parser.add_argument("--overlays", help="Treats each section of the overalays as separate files.", action="store_true")
    parser.add_argument("--csv", help="Print the output in csv format instead.", action="store_true")
    #parser.add_argument("--ignore80", help="Ignores words differences that starts in 0x80XXXXXX", action="store_true")
    parser.add_argument("--ignore80", help="Ignores words differences that starts in 0x80XXXXXX", action="store_true", default=True) # temporal?
    parser.add_argument("--ignore06", help="Ignores words differences that starts in 0x06XXXXXX", action="store_true")
    parser.add_argument("--ignore04", help="Ignores words differences that starts in 0x04XXXXXX", action="store_true")
    parser.add_argument("--ignore-branches", help="Ignores the address of every branch, jump and jal.", action="store_true")
    parser.add_argument("--column1", help="Name for column one (baserom) in the csv.", default=None)
    parser.add_argument("--column2", help="Name for column two (other_baserom) in the csv.", default=None)
    args = parser.parse_args()

    filelist = readFile(args.filelist)
    # filelist = readJson(args.filelist)

    if args.csv:
        compare_to_csv(args, filelist)
    else:
        compare_baseroms(args, filelist)


if __name__ == "__main__":
    main()
