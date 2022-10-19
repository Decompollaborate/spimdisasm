#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import rabbitizer
from pathlib import Path

from .. import common
from .. import mips


def getArgsParser() -> argparse.ArgumentParser:
    description = "RSP N64 disassembler"
    parser = argparse.ArgumentParser(description=description)


    parser.add_argument("binary", help="Path to input binary")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--start", help="Raw offset of the input binary file to start disassembling. Expects an hex value", default="0")
    parser.add_argument("--end", help="Offset end of the input binary file to start disassembling. Expects an hex value",  default="0xFFFFFF")
    parser.add_argument("--vram", help="Set the VRAM address. Expects an hex value", default="0x0")

    common.Context.addParametersToArgParse(parser)

    common.GlobalConfig.addParametersToArgParse(parser)

    mips.InstructionConfig.addParametersToArgParse(parser)

    return parser

def applyArgs(args: argparse.Namespace) -> None:
    mips.InstructionConfig.parseArgs(args)
    common.GlobalConfig.parseArgs(args)

def applyGlobalConfigurations() -> None:
    common.GlobalConfig.GLABEL_ASM_COUNT = False
    common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL = True
    common.GlobalConfig.ASM_USE_PRELUDE = False
    common.GlobalConfig.ASM_USE_SYMBOL_LABEL = False
    rabbitizer.config.misc_unknownInstrComment = False

def initializeContext(args: argparse.Namespace, fileSize: int, fileVram: int) -> common.Context:
    context = common.Context()
    context.parseArgs(args)

    highestVromEnd = fileSize
    lowestVramStart = 0x00000000
    highestVramEnd = lowestVramStart + highestVromEnd
    if fileVram != 0:
        lowestVramStart = fileVram
        highestVramEnd = fileVram + highestVromEnd

    context.changeGlobalSegmentRanges(0, highestVromEnd, lowestVramStart, highestVramEnd)
    return context


def rspDisasmMain():
    args = getArgsParser().parse_args()
    applyArgs(args)

    applyGlobalConfigurations()

    binaryPath = Path(args.binary)
    array_of_bytes = common.Utils.readFileAsBytearray(binaryPath)
    inputName = binaryPath.stem

    start = int(args.start, 16)
    end = int(args.end, 16)
    if end == 0xFFFFFF:
        end = len(array_of_bytes)
    fileVram = int(args.vram, 16)

    context = initializeContext(args, len(array_of_bytes), fileVram)

    f = mips.sections.SectionText(context, start, end, fileVram, inputName, array_of_bytes, 0, None)
    f.instrCat = rabbitizer.InstrCategory.RSP

    f.analyze()
    f.printAnalyzisResults()

    mips.FilesHandlers.writeSection(Path(args.output), f)

    if args.save_context is not None:
        contextPath = Path(args.save_context)
        contextPath.parent.mkdir(parents=True, exist_ok=True)
        context.saveContextToFile(contextPath)
