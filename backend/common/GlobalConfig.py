#!/usr/bin/python3

from __future__ import annotations

import sys
import argparse

class GlobalConfig:
    REMOVE_POINTERS: bool = False
    IGNORE_BRANCHES: bool = False # Ignores the address of every branch, jump and jal
    IGNORE_WORD_LIST: set = set() # Ignores words that starts in 0xXX

    WRITE_BINARY: bool = False # write to files splitted binaries

    ASM_COMMENT: bool = True
    FUNCTION_ASM_COUNT: bool = True

    ADD_NEW_SYMBOLS: bool = True
    PRODUCE_SYMBOLS_PLUS_OFFSET: bool = False
    SYMBOL_FINDER_FILTER_LOW_ADDRESSES: bool = True

    TRUST_USER_FUNCTIONS: bool = True
    DISASSEMBLE_UNKNOWN_INSTRUCTIONS: bool = False
    DISASSEMBLE_RSP: bool = False

    STRING_GUESSER: bool = False

    QUIET: bool = False
    VERBOSE: bool = False
    PRINT_FUNCTION_ANALYSIS_DEBUG_INFO: bool = False
    PRINT_SYMBOL_FINDER_DEBUG_INFO: bool = False


    @staticmethod
    def addParametersToArgParse(parser: argparse.ArgumentParser):
        backendConfig = parser.add_argument_group("Disassembler backend configuration")

        backendConfig.add_argument("--disasm-unknown", help="Force disassembly of functions with unknown instructions",  action="store_true")
        backendConfig.add_argument("--disasm-rsp", help="Experimental. Enables the disassembly of rsp abi instructions. Warning: In its current state the generated asm may not be assemblable to a matching binary",  action="store_true")

        backendConfig.add_argument("--ignore-words", help="A space separated list of hex numbers. Word differences will be ignored that starts in any of the provided arguments. Max value: FF. Only works when --nuke-pointers is passed", action="extend", nargs="+")

        backendConfig.add_argument("--disable-string-guesser", help="Disables the string guesser feature (does nto affect the strings referenced by .data)", action="store_true")

        backendConfig.add_argument("--not-filter-low-addressses", help="Treat low addresses (lower than 0x40000000) as real pointers.", action="store_true")


        miscConfig = parser.add_argument_group("Disassembler misc options")

        miscConfig.add_argument("--disable-asm-comments", help="Disables the comments in assembly code", action="store_true")
        miscConfig.add_argument("--write-binary", help="Produce a binary of the processed file", action="store_true")


        verbosityConfig = parser.add_argument_group("Verbosity options")

        verbosityConfig.add_argument("-v", "--verbose", help="Enable verbose mode",  action="store_true")
        verbosityConfig.add_argument("-q", "--quiet", help="Silence most output",  action="store_true")


        debugging = parser.add_argument_group("Disassembler debugging options")

        debugging.add_argument("--debug-func-analysis", help="Enables some debug info printing related to the function analysis)", action="store_true")
        debugging.add_argument("--debug-symbol-finder", help="Enables some debug info printing related to the symbol finder system)", action="store_true")


    @classmethod
    def parseArgs(cls, args: argparse.Namespace):
        GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS = args.disasm_unknown
        GlobalConfig.DISASSEMBLE_RSP = args.disasm_rsp

        if args.ignore_words:
            for upperByte in args.ignore_words:
                GlobalConfig.IGNORE_WORD_LIST.add(int(upperByte, 16))

        GlobalConfig.STRING_GUESSER = not args.disable_string_guesser
        GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES = not args.not_filter_low_addressses

        GlobalConfig.WRITE_BINARY = args.write_binary
        GlobalConfig.ASM_COMMENT = not args.disable_asm_comments

        GlobalConfig.VERBOSE = args.verbose
        GlobalConfig.QUIET = args.quiet

        GlobalConfig.PRINT_FUNCTION_ANALYSIS_DEBUG_INFO = args.debug_func_analysis
        GlobalConfig.PRINT_SYMBOL_FINDER_DEBUG_INFO = args.debug_symbol_finder


def printQuietless(*args, **kwargs):
    if not GlobalConfig.QUIET:
        print(*args, **kwargs)

def epprintQuietless(*args, **kwargs):
    if not GlobalConfig.QUIET:
        print(*args, file=sys.stderr, **kwargs)


def printVerbose(*args, **kwargs):
    if not GlobalConfig.QUIET and GlobalConfig.VERBOSE:
        print(*args, **kwargs)

def eprintVerbose(*args, **kwargs):
    if not GlobalConfig.QUIET and GlobalConfig.VERBOSE:
        print(*args, file=sys.stderr, **kwargs)
