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

    parser.add_argument("input", help="Hex words to be disassembled. Leading '0x' must be omitted", nargs='+')

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

def getWordFromStr(inputStr: str) -> int:
    arr = bytearray()
    for index in range(0, len(inputStr), 2):
        byteStr = inputStr[index:index+2]
        temp = 0
        for char in byteStr:
            temp *= 16
            temp += int(char, 16)
        arr.append(temp)
    return common.Utils.bytesToWords(arr)[0]

def wordGeneratorFromStrList(inputlist: list|None) -> Generator[int, None, None]:
    if inputlist is None:
        return

    wordStr = ""
    for inputStr in inputlist:
        for character in inputStr:
            if character not in "0123456789abcdefABCDEF":
                continue
            wordStr += character
            if len(wordStr) == 8:
                yield getWordFromStr(wordStr)
                wordStr = ""

    if len(wordStr) > 0:
        yield getWordFromStr(wordStr)

def getWordListFromStdin():
    if sys.stdin.isatty():
        return

    lines = ""
    try:
        for line in sys.stdin:
            lines += line
    except KeyboardInterrupt:
        pass
    for word in wordGeneratorFromStrList(lines.split(" ")):
        yield word


def disasmdisMain():
    args = getArgsParser().parse_args()

    applyArgs(args)

    category = getInstrCategoryFromStr(args.category)

    for word in getWordListFromStdin():
        instr = rabbitizer.Instruction(word, category=category)
        print(instr.disassemble())

    for word in wordGeneratorFromStrList(args.input):
        instr = rabbitizer.Instruction(word, category=category)
        print(instr.disassemble())
