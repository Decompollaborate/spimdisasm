#!/usr/bin/python3

from __future__ import annotations

import argparse
import pathlib

from backend.common.Utils import *
from backend.common.GlobalConfig import GlobalConfig
from backend.common.FileSectionType import FileSectionType

from backend.elf32.Elf32File import Elf32File

from backend.mips.MipsContext import Context
from backend.mips import MipsSection
from backend.mips import MipsText
from backend.mips import MipsData
from backend.mips import MipsRodata
from backend.mips import MipsBss
from backend.mips import FilesHandlers


def elfObjDisasmMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    args = parser.parse_args()

    GlobalConfig.VERBOSE = False

    inputPath = pathlib.Path(args.binary)

    context = Context()

    array_of_bytes = readFileAsBytearray(args.binary)

    elfFile = Elf32File(array_of_bytes)

    processedFiles: dict[FileSectionType, tuple[pathlib.Path, MipsSection.Section]] = dict()

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    for sectionType, sectionBytes in elfFile.progbits.items():
        outputPath = dataOutput
        if sectionType == FileSectionType.Text:
            outputPath = textOutput

        outputFilePath = pathlib.Path(outputPath)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        if sectionType == FileSectionType.Text:
            processedFiles[sectionType] = (outputFilePath, MipsText.Text(sectionBytes, inputPath.stem, context))
        if sectionType == FileSectionType.Data:
            processedFiles[sectionType] = (outputFilePath, MipsData.Data(sectionBytes, inputPath.stem, context))
        if sectionType == FileSectionType.Rodata:
            processedFiles[sectionType] = (outputFilePath, MipsRodata.Rodata(sectionBytes, inputPath.stem, context))

    if elfFile.nobits is not None:
        outputPath = dataOutput

        outputFilePath = pathlib.Path(dataOutput)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        processedFiles[FileSectionType.Bss] = (outputFilePath, MipsBss.Bss(0, elfFile.nobits, inputPath.stem, context))

    # TODO: use the elfFile.rel stuff

    for outputFilePath, subFile in processedFiles.values():
        subFile.analyze()

    for outputFilePath, subFile in processedFiles.values():
        FilesHandlers.writeSection(str(outputFilePath), subFile)


if __name__ == "__main__":
    elfObjDisasmMain()
