#!/usr/bin/env python3

from __future__ import annotations

import argparse

from backend.common.Utils import *
from backend.common.GlobalConfig import GlobalConfig
from backend.common.Context import Context

from backend.mips.Instructions import wordToInstruction, wordToInstructionRsp, InstructionBase
from backend.mips.MipsFunction import Function


def disasmdisMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("input", help="Hex words to be disassembled. Leading '0x' must be omitted")

    parser.add_argument("--raw-instr", help="Print raw instructions without performing analyzis on them", action="store_true")

    args = parser.parse_args()

    GlobalConfig.ASM_COMMENT = False
    GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS = True

    # Count the amount of words and round up to a word boundary
    wordsCount = (len(args.input) - 1) // 8 + 1

    context = Context()

    instructionList: list[InstructionBase] = list()

    for i in range(wordsCount):
        # print(i)
        word = int(args.input[i*8:(i+1)*8], 16)
        instructionList.append(wordToInstruction(word))

    if args.raw_instr:
        for instr in instructionList:
            print(instr.disassemble())
    else:
        func = Function("", instructionList, context, 0)
        func.analyze()
        print(func.disassemble(), end="")


if __name__ == "__main__":
    disasmdisMain()
