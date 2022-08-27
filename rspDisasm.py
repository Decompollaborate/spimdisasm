#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import os
from pathlib import Path

import spimdisasm
import rabbitizer


def rspDisasmMain():
    description = "RSP N64 disassembler"
    parser = argparse.ArgumentParser(description=description)


    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--start", help="Raw offset of the input binary file to start disassembling. Expects an hex value", default="0")
    parser.add_argument("--end", help="Offset end of the input binary file to start disassembling. Expects an hex value",  default="0xFFFFFF")
    parser.add_argument("--vram", help="Set the VRAM address. Expects an hex value", default="0x0")


    spimdisasm.common.GlobalConfig.GLABEL_ASM_COUNT = False
    spimdisasm.common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL = True
    spimdisasm.common.GlobalConfig.ASM_USE_PRELUDE = False
    spimdisasm.common.GlobalConfig.ASM_USE_SYMBOL_LABEL = False
    rabbitizer.config.misc_unknownInstrComment = False


    spimdisasm.common.Context.addParametersToArgParse(parser)

    spimdisasm.common.GlobalConfig.addParametersToArgParse(parser)

    spimdisasm.mips.InstructionConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    spimdisasm.mips.InstructionConfig.parseArgs(args)

    spimdisasm.common.GlobalConfig.parseArgs(args)


    context = spimdisasm.common.Context()
    context.parseArgs(args)

    array_of_bytes = spimdisasm.common.Utils.readFileAsBytearray(args.binary)
    input_name = os.path.splitext(os.path.split(args.binary)[1])[0]


    start = int(args.start, 16)
    end = int(args.end, 16)
    fileVram = int(args.vram, 16)

    f = spimdisasm.mips.sections.SectionText(context, start, end, fileVram, input_name, array_of_bytes, 0, None)
    f.isRsp = True

    highestVromEnd = len(array_of_bytes)
    lowestVramStart = 0x00000000
    highestVramEnd = lowestVramStart + highestVromEnd
    fileVram = int(args.vram, 16)
    if fileVram != 0:
        lowestVramStart = fileVram
        highestVramEnd = fileVram + highestVromEnd

    context.globalSegment.changeRanges(0, highestVromEnd, lowestVramStart, highestVramEnd)

    f.analyze()
    f.printAnalyzisResults()


    spimdisasm.mips.FilesHandlers.writeSection(args.output, f)


    if args.save_context is not None:
        contextPath = Path(args.save_context)
        contextPath.parent.mkdir(parents=True, exist_ok=True)
        context.saveContextToFile(contextPath)


if __name__ == "__main__":
    rspDisasmMain()
