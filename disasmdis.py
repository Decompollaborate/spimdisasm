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

    parser.add_argument("--raw-instr", help="Print raw instructions without performing analyzis on them", action=argparse.BooleanOptionalAction)

    parser.add_argument("--endian", help="Set the endianness of input files. Defaults to 'big'", choices=["big", "little", "middle"])

    args = parser.parse_args()

    disasmBack.GlobalConfig.ASM_COMMENT = False
    disasmBack.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS = True

    if args.endian == "little":
        disasmBack.GlobalConfig.ENDIAN = disasmBack.InputEndian.LITTLE
    elif args.endian == "middle":
        disasmBack.GlobalConfig.ENDIAN = disasmBack.InputEndian.MIDDLE
    else:
        disasmBack.GlobalConfig.ENDIAN = disasmBack.InputEndian.BIG

    # Count the amount of words and round up to a word boundary
    wordsCount = (len(args.input) - 1) // 8 + 1

    context = disasmBack.Context()

    instructionList: list[disasmBack.mips.instructions.InstructionBase] = list()

    for i in range(wordsCount):
        array_of_bytes = bytearray(4)
        wordStr = args.input[i*8:(i+1)*8].ljust(8, "0")
        for j in range(4):
            array_of_bytes[j] = int(wordStr[j*2:(j+1)*2], 16)

        word = disasmBack.Utils.bytesToBEWords(array_of_bytes)[0]
        instructionList.append(disasmBack.mips.instructions.wordToInstruction(word))

    if args.raw_instr:
        for instr in instructionList:
            print(instr.disassemble())
    else:
        func = disasmBack.mips.symbols.SymbolFunction(context, 0, None, "", instructionList)
        func.analyze()
        print(func.disassemble(), end="")


if __name__ == "__main__":
    disasmdisMain()
