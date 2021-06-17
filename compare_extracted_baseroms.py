#!/usr/bin/python3

from __future__ import annotations

import argparse
import os

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsFile import File
from mips.MipsOverlay import Overlay

script_dir = os.path.dirname(os.path.realpath(__file__))
root_dir = script_dir + "/.."
if not script_dir.endswith("/tools"):
    root_dir = script_dir
baserom_path = root_dir + "/baserom_"


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

        file_one_data = readFileAsBytearray(filepath_one)
        file_two_data = readFileAsBytearray(filepath_two)

        if filename.startswith("ovl_"):
            file_one = Overlay(file_one_data, filename, args.version1)
            file_two = Overlay(file_two_data, filename, args.version2)
        else:
            file_one = File(file_one_data, filename, args.version1)
            file_two = File(file_two_data, filename, args.version2)

        file_one.blankOutDifferences(file_two)

        comparison = file_one.compareToFile(file_two)

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

        file_one_data = readFileAsBytearray(filepath_one)
        file_two_data = readFileAsBytearray(filepath_two)

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
                file_one = Overlay(file_one_data, filename, args.version1)
                file_two = Overlay(file_two_data, filename, args.version2)
            else:
                file_one = File(file_one_data, filename, args.version1)
                file_two = File(file_two_data, filename, args.version2)

            file_one.blankOutDifferences(file_two)

            comparison = file_one.compareToFile(file_two)
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
    parser.add_argument("--ignore80", help="Ignores words differences that starts in 0x80XXXXXX", action="store_true")
    parser.add_argument("--ignore06", help="Ignores words differences that starts in 0x06XXXXXX", action="store_true")
    parser.add_argument("--ignore04", help="Ignores words differences that starts in 0x04XXXXXX", action="store_true")
    parser.add_argument("--track-registers", help="Set for how many instructions a register will be tracked.", type=int)
    parser.add_argument("--ignore-branches", help="Ignores the address of every branch, jump and jal.", action="store_true")
    parser.add_argument("--dont-remove-ptrs", help="Disable the pointer removal feature.", action="store_true")
    parser.add_argument("--column1", help="Name for column one (baserom) in the csv.", default=None)
    parser.add_argument("--column2", help="Name for column two (other_baserom) in the csv.", default=None)
    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = not args.dont_remove_ptrs
    GlobalConfig.IGNORE_BRANCHES = args.ignore_branches
    if args.track_registers is not None:
        GlobalConfig.TRACK_REGISTERS = args.track_registers
    GlobalConfig.IGNORE_04 = args.ignore04
    GlobalConfig.IGNORE_06 = args.ignore06
    GlobalConfig.IGNORE_80 = args.ignore80

    filelist = readFile(args.filelist)
    # filelist = readJson(args.filelist)

    if args.csv:
        compare_to_csv(args, filelist)
    else:
        compare_baseroms(args, filelist)


if __name__ == "__main__":
    main()
