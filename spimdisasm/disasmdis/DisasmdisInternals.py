#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
from typing import Generator
import rabbitizer
import sys

from .. import common


def getArgsParser() -> argparse.ArgumentParser:
    description = "CLI tool to disassemble multiples instructions passed as argument"
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("input", help="Hex words to be disassembled. Leading '0x' must be omitted", nargs='?')

    parser.add_argument("--endian", help="Set the endianness of input files. Defaults to 'big'", choices=["big", "little", "middle"], default="big")
    parser.add_argument("--category", help="The instruction category to use when disassembling every passed instruction. Defaults to 'cpu'", choices=["cpu", "rsp", "r5900"])

    return parser

def applyArgs(args: argparse.Namespace) -> None:
    common.GlobalConfig.ENDIAN = common.InputEndian(args.endian)

def getInstrCategoryFromStr(category: str) -> rabbitizer.Enum:
    # TODO: consider moving this logic to rabbitizer
    if category == "rsp":
        return rabbitizer.InstrCategory.RSP
    elif category == "r5900":
        return rabbitizer.InstrCategory.R5900

    return rabbitizer.InstrCategory.CPU

def getWordListFromStrList(inputlist: list|None) -> Generator[int, None, None]:
    if inputlist is None:
        return

    wordStr = ""
    for inputStr in inputlist:
        for character in inputStr:
            if character not in "0123456789abcdefABCDEF":
                continue
            wordStr += character
            if len(wordStr) == 8:
                yield int(wordStr, 16)
                wordStr = ""

    if len(wordStr) > 0:
        yield int(wordStr, 16)

def getWordListFromStdin():
    if sys.stdin.isatty():
        return

    lines = ""
    try:
        for line in sys.stdin:
            lines += line
    except KeyboardInterrupt:
        pass
    for word in getWordListFromStrList(lines.split(" ")):
        yield word


def disasmdisMain():
    args = getArgsParser().parse_args()

    applyArgs(args)

    category = getInstrCategoryFromStr(args.category)

    for word in getWordListFromStdin():
        instr = rabbitizer.Instruction(word, category=category)
        print(instr.disassemble())

    for word in getWordListFromStrList(args.input):
        instr = rabbitizer.Instruction(word, category=category)
        print(instr.disassemble())
