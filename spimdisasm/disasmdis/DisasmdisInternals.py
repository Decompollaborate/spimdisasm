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

def getWordListFromStr(input: str) -> list[int]:
    wordList: list[int] = []

    # Count the amount of words and round up to a word boundary
    wordsCount = (len(input) - 1) // 8 + 1

    for i in range(wordsCount):
        array_of_bytes = bytearray(4)
        wordStr = input[i*8:(i+1)*8].ljust(8, "0")
        for j in range(4):
            array_of_bytes[j] = int(wordStr[j*2:(j+1)*2], 16)

        wordList.append(common.Utils.bytesToWords(array_of_bytes)[0])
    return wordList


def disasmdisMain():
    args = getArgsParser().parse_args()

    applyArgs(args)

    category = getInstrCategoryFromStr(args.category)

    for word in getWordListFromStr(args.input):
        instr = rabbitizer.Instruction(word, category=category)
        print(instr.disassemble())
