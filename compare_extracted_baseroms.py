#!/usr/bin/python3

from __future__ import annotations

import argparse
import os

from mips.Utils import *
from mips.GlobalConfig import GlobalConfig
from mips.MipsSection import Section
from mips.MipsFileOverlay import FileOverlay
from mips.MipsFileCode import FileCode
from mips.MipsFileBoot import FileBoot
from mips.MipsContext import Context
from mips.MipsSplitEntry import readSplitsFromCsv

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

    context_one = Context()
    context_two = Context()
    context_one.readFunctionMap(args.version1)
    context_two.readFunctionMap(args.version2)

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
            file_one = FileOverlay(file_one_data, filename, args.version1, context_one)
            file_two = FileOverlay(file_two_data, filename, args.version2, context_two)
        elif filename == "code":
            textSplits = readSplitsFromCsv("csvsplits/code_text.csv") if os.path.exists("csvsplits/code_text.csv") else {args.version1: dict(), args.version2: dict()}
            dataSplits = readSplitsFromCsv("csvsplits/code_data.csv") if os.path.exists("csvsplits/code_data.csv") else {args.version1: dict(), args.version2: dict()}
            rodataSplits = readSplitsFromCsv("csvsplits/code_rodata.csv") if os.path.exists("csvsplits/code_rodata.csv") else {args.version1: dict(), args.version2: dict()}
            bssSplits = readSplitsFromCsv("csvsplits/code_bss.csv") if os.path.exists("csvsplits/code_bss.csv") else {args.version1: dict(), args.version2: dict()}
            file_one = FileCode(file_one_data, args.version1, context_one, textSplits[args.version1], dataSplits[args.version1], rodataSplits[args.version1], bssSplits[args.version1])
            file_two = FileCode(file_two_data, args.version2, context_two, textSplits[args.version2], dataSplits[args.version2], rodataSplits[args.version2], bssSplits[args.version2])
        elif filename == "boot":
            textSplits = readSplitsFromCsv("csvsplits/boot_text.csv") if os.path.exists("csvsplits/boot_text.csv") else {args.version1: dict(), args.version2: dict()}
            dataSplits = readSplitsFromCsv("csvsplits/boot_data.csv") if os.path.exists("csvsplits/boot_data.csv") else {args.version1: dict(), args.version2: dict()}
            rodataSplits = readSplitsFromCsv("csvsplits/boot_rodata.csv") if os.path.exists("csvsplits/boot_rodata.csv") else {args.version1: dict(), args.version2: dict()}
            bssSplits = readSplitsFromCsv("csvsplits/boot_bss.csv") if os.path.exists("csvsplits/boot_bss.csv") else {args.version1: dict(), args.version2: dict()}
            file_one = FileBoot(file_one_data, args.version1, context_one, textSplits[args.version1], dataSplits[args.version1], rodataSplits[args.version1], bssSplits[args.version1])
            file_two = FileBoot(file_two_data, args.version2, context_two, textSplits[args.version2], dataSplits[args.version2], rodataSplits[args.version2], bssSplits[args.version2])
        else:
            file_one = Section(file_one_data, filename, args.version1, context_one)
            file_two = Section(file_two_data, filename, args.version2, context_two)

        file_one.analyze()
        file_two.analyze()

        if GlobalConfig.REMOVE_POINTERS:
            both_updated = file_one.blankOutDifferences(file_two)
            one_updated = file_one.removePointers()
            two_updated = file_two.removePointers()
            if both_updated or one_updated:
                file_one.updateBytes()
            if both_updated or two_updated:
                file_two.updateBytes()

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

                if "filesections" in comparison:
                    for section_name in comparison["filesections"]:
                        section = comparison["filesections"][section_name]

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

def print_section_as_csv(args, index: int, filename: str, section_name: str, section: dict):
    equal = section["equal"]

    if equal and args.print not in ("all", "equals"):
        return
    if not equal and args.print not in ("all", "diffs"):
        return

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
        if args.split_files:
            if "text" in section:
                print(f',{section["text"]["diff_opcode"]},{section["text"]["same_opcode_same_args"]}', end="")
            else:
                print(",,", end="")
        print()

def compare_to_csv(args, filelist):
    index = -1

    column1 = args.version1 if args.column1 is None else args.column1
    column2 = args.version2 if args.column2 is None else args.column2

    context_one = Context()
    context_two = Context()
    context_one.readFunctionMap(args.version1)
    context_two.readFunctionMap(args.version2)

    print(f"Index,File,Are equals,Size in {column1},Size in {column2},Size proportion,Size difference,Bytes different,Words different", end="")
    if args.split_files:
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

            print(f'{index},{filename},{equal},{len_one},{len_two},{div},{size_difference},{diff_bytes},{diff_words}', end="")
            if args.split_files:
                print(",,", end="")
            print()

        else:
            if args.split_files and filename.startswith("ovl_"):
                file_one = FileOverlay(file_one_data, filename, args.version1, context_one)
                file_two = FileOverlay(file_two_data, filename, args.version2, context_two)
            elif args.split_files and filename == "code":
                textSplits = readSplitsFromCsv("csvsplits/code_text.csv") if os.path.exists("csvsplits/code_text.csv") else {args.version1: dict(), args.version2: dict()}
                dataSplits = readSplitsFromCsv("csvsplits/code_data.csv") if os.path.exists("csvsplits/code_data.csv") else {args.version1: dict(), args.version2: dict()}
                rodataSplits = readSplitsFromCsv("csvsplits/code_rodata.csv") if os.path.exists("csvsplits/code_rodata.csv") else {args.version1: dict(), args.version2: dict()}
                bssSplits = readSplitsFromCsv("csvsplits/code_bss.csv") if os.path.exists("csvsplits/code_bss.csv") else {args.version1: dict(), args.version2: dict()}
                file_one = FileCode(file_one_data, args.version1, context_one, textSplits[args.version1], dataSplits[args.version1], rodataSplits[args.version1], bssSplits[args.version1])
                file_two = FileCode(file_two_data, args.version2, context_two, textSplits[args.version2], dataSplits[args.version2], rodataSplits[args.version2], bssSplits[args.version2])
            elif filename == "boot":
                textSplits = readSplitsFromCsv("csvsplits/boot_text.csv") if os.path.exists("csvsplits/boot_text.csv") else {args.version1: dict(), args.version2: dict()}
                dataSplits = readSplitsFromCsv("csvsplits/boot_data.csv") if os.path.exists("csvsplits/boot_data.csv") else {args.version1: dict(), args.version2: dict()}
                rodataSplits = readSplitsFromCsv("csvsplits/boot_rodata.csv") if os.path.exists("csvsplits/boot_rodata.csv") else {args.version1: dict(), args.version2: dict()}
                bssSplits = readSplitsFromCsv("csvsplits/boot_bss.csv") if os.path.exists("csvsplits/boot_bss.csv") else {args.version1: dict(), args.version2: dict()}
                file_one = FileBoot(file_one_data, args.version1, context_one, textSplits[args.version1], dataSplits[args.version1], rodataSplits[args.version1], bssSplits[args.version1])
                file_two = FileBoot(file_two_data, args.version2, context_two, textSplits[args.version2], dataSplits[args.version2], rodataSplits[args.version2], bssSplits[args.version2])
            else:
                file_one = Section(file_one_data, filename, args.version1, context_one)
                file_two = Section(file_two_data, filename, args.version2, context_two)

            file_one.analyze()
            file_two.analyze()

            if GlobalConfig.REMOVE_POINTERS:
                both_updated = file_one.blankOutDifferences(file_two)
                one_updated = file_one.removePointers()
                two_updated = file_two.removePointers()
                if both_updated or one_updated:
                    file_one.updateBytes()
                if both_updated or two_updated:
                    file_two.updateBytes()

            comparison = file_one.compareToFile(file_two)
            if "filesections" in comparison:
                for section_name in comparison["filesections"]:
                    section = comparison["filesections"][section_name]
                    for n, sub in section.items():
                        aux_section_name = section_name
                        if n != filename:
                            aux_section_name = f"{n} {section_name}"
                        print_section_as_csv(args, index, filename, aux_section_name, sub)
            else:
                print_section_as_csv(args, index, filename, "", comparison)


def main():
    description = ""

    epilog = """\
    """
    parser = argparse.ArgumentParser(description=description, epilog=epilog, formatter_class=argparse.RawTextHelpFormatter)
    parser.add_argument("version1", help="A version of the game to compare. The files will be read from baserom_version1. For example: baserom_pal_mq_dbg")
    parser.add_argument("version2", help="A version of the game to compare. The files will be read from baserom_version2. For example: baserom_pal_mq")
    parser.add_argument("filelist", help="Path to the filelist that will be used.")
    parser.add_argument("--print", help="Select what will be printed for a cleaner output. Default is 'all'.", choices=["all", "equals", "diffs", "missing"], default="all")
    parser.add_argument("--split-files", help="Treats each section of a a file as separate files.", action="store_true")
    parser.add_argument("--no-csv", help="Don't print the output in csv format.", action="store_true")
    parser.add_argument("--ignore80", help="Ignores words differences that starts in 0x80XXXXXX", action="store_true")
    parser.add_argument("--ignore06", help="Ignores words differences that starts in 0x06XXXXXX", action="store_true")
    parser.add_argument("--ignore04", help="Ignores words differences that starts in 0x04XXXXXX", action="store_true")
    parser.add_argument("--ignore-branches", help="Ignores the address of every branch, jump and jal.", action="store_true")
    parser.add_argument("--dont-remove-ptrs", help="Disable the pointer removal feature.", action="store_true")
    parser.add_argument("--column1", help="Name for column one (baserom) in the csv.", default=None)
    parser.add_argument("--column2", help="Name for column two (other_baserom) in the csv.", default=None)
    args = parser.parse_args()

    GlobalConfig.REMOVE_POINTERS = not args.dont_remove_ptrs
    GlobalConfig.IGNORE_BRANCHES = args.ignore_branches
    GlobalConfig.IGNORE_04 = args.ignore04
    GlobalConfig.IGNORE_06 = args.ignore06
    GlobalConfig.IGNORE_80 = args.ignore80

    filelist = readFile(args.filelist)
    # filelist = readJson(args.filelist)

    if not args.no_csv:
        compare_to_csv(args, filelist)
    else:
        compare_baseroms(args, filelist)


if __name__ == "__main__":
    main()
