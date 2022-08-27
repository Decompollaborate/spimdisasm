#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import rabbitizer

import spimdisasm


def disasmdisMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("input", help="Hex words to be disassembled. Leading '0x' must be omitted")

    parser.add_argument("--endian", help="Set the endianness of input files. Defaults to 'big'", choices=["big", "little", "middle"])
    parser.add_argument("--category", help="The instruction category to use when disassembling every passed instruction. Defaults to 'cpu'", choices=["cpu", "rsp", "r5900"])

    args = parser.parse_args()

    spimdisasm.common.GlobalConfig.ASM_COMMENT = False
    spimdisasm.common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS = True

    if args.endian == "little":
        spimdisasm.common.GlobalConfig.ENDIAN = spimdisasm.common.InputEndian.LITTLE
    elif args.endian == "middle":
        spimdisasm.common.GlobalConfig.ENDIAN = spimdisasm.common.InputEndian.MIDDLE
    else:
        spimdisasm.common.GlobalConfig.ENDIAN = spimdisasm.common.InputEndian.BIG

    # Count the amount of words and round up to a word boundary
    wordsCount = (len(args.input) - 1) // 8 + 1

    context = spimdisasm.common.Context()
    context.globalSegment.changeRanges(0x0, 0xFFFFFFFF, 0x0, 0xFFFFFFFF)

    instructionList: list[rabbitizer.Instruction] = list()

    category = rabbitizer.InstrCategory.CPU
    if args.category == "rsp":
        category = rabbitizer.InstrCategory.RSP
    elif args.category == "r5900":
        category = rabbitizer.InstrCategory.R5900

    for i in range(wordsCount):
        array_of_bytes = bytearray(4)
        wordStr = args.input[i*8:(i+1)*8].ljust(8, "0")
        for j in range(4):
            array_of_bytes[j] = int(wordStr[j*2:(j+1)*2], 16)

        word = spimdisasm.common.Utils.bytesToBEWords(array_of_bytes)[0]

        instructionList.append(rabbitizer.Instruction(word, category=category))

    for instr in instructionList:
        print(instr.disassemble())


if __name__ == "__main__":
    disasmdisMain()
