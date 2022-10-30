#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import rabbitizer

from .. import common


def getArgsParser() -> argparse.ArgumentParser:
    description = "CLI tool to disassemble multiples instructions passed as argument"
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("input", help="Hex words to be disassembled. Leading '0x' must be omitted")

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

def getWordListFromStr(inputStr: str) -> list[int]:
    wordList: list[int] = []

    wordStr = ""
    for character in inputStr:
        if character not in "0123456789abcdefABCDEF":
            continue
        wordStr += character
        if len(wordStr) == 8:
            wordList.append(int(wordStr, 16))
            wordStr = ""

    if len(wordStr) > 0:
        wordList.append(int(wordStr, 16))

    return wordList


def disasmdisMain():
    args = getArgsParser().parse_args()

    applyArgs(args)

    category = getInstrCategoryFromStr(args.category)

    for word in getWordListFromStr(args.input):
        instr = rabbitizer.Instruction(word, category=category)
        print(instr.disassemble())
