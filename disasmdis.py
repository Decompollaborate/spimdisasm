#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse

import backend as disasmBack


def disasmdisMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("input", help="Hex words to be disassembled. Leading '0x' must be omitted")

    parser.add_argument("--raw-instr", help="Print raw instructions without performing analyzis on them", action="store_true")

    args = parser.parse_args()

    disasmBack.GlobalConfig.ASM_COMMENT = False
    disasmBack.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS = True

    # Count the amount of words and round up to a word boundary
    wordsCount = (len(args.input) - 1) // 8 + 1

    context = disasmBack.Context()

    instructionList: list[disasmBack.mips.Instructions.InstructionBase] = list()

    for i in range(wordsCount):
        # print(i)
        word = int(args.input[i*8:(i+1)*8], 16)
        instructionList.append(disasmBack.mips.Instructions.wordToInstruction(word))

    if args.raw_instr:
        for instr in instructionList:
            print(instr.disassemble())
    else:
        func = disasmBack.mips.Symbols.SymbolFunction(context, 0, None, "", instructionList)
        func.analyze()
        print(func.disassemble(), end="")


if __name__ == "__main__":
    disasmdisMain()
