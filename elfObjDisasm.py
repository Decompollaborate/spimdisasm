#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import pathlib

import backend as disasmBack


def elfObjDisasmMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    disasmBack.GlobalConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    disasmBack.GlobalConfig.parseArgs(args)

    disasmBack.GlobalConfig.REMOVE_POINTERS = False
    disasmBack.GlobalConfig.IGNORE_BRANCHES = False
    disasmBack.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES = False

    # GlobalConfig.VERBOSE = True

    inputPath = pathlib.Path(args.binary)

    context = disasmBack.Context()

    array_of_bytes = disasmBack.Utils.readFileAsBytearray(args.binary)

    elfFile = disasmBack.elf32.Elf32File(array_of_bytes)

    processedFiles: dict[disasmBack.FileSectionType, tuple[pathlib.Path, disasmBack.mips.sections.SectionBase]] = dict()

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    for sectionType, sectionBytes in elfFile.progbits.items():
        outputPath = dataOutput
        if sectionType == disasmBack.FileSectionType.Text:
            outputPath = textOutput

        outputFilePath = pathlib.Path(outputPath)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        if sectionType == disasmBack.FileSectionType.Text:
            processedFiles[sectionType] = (outputFilePath, disasmBack.mips.sections.SectionText(context, None, inputPath.stem, sectionBytes))
        if sectionType == disasmBack.FileSectionType.Data:
            processedFiles[sectionType] = (outputFilePath, disasmBack.mips.sections.SectionData(context, None, inputPath.stem, sectionBytes))
        if sectionType == disasmBack.FileSectionType.Rodata:
            processedFiles[sectionType] = (outputFilePath, disasmBack.mips.sections.SectionRodata(context, None, inputPath.stem, sectionBytes))

    if elfFile.nobits is not None:
        outputPath = dataOutput

        outputFilePath = pathlib.Path(dataOutput)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        processedFiles[disasmBack.FileSectionType.Bss] = (outputFilePath, disasmBack.mips.sections.SectionBss(context, 0, elfFile.nobits, inputPath.stem))

    if elfFile.symtab is not None and elfFile.strtab is not None:
        # Inject symbols from the reloc table referenced in each section
        for sectType, relocs in elfFile.rel.items():
            subSection = processedFiles[sectType][1]
            for rel in relocs:
                symbolEntry = elfFile.symtab[rel.rSym]
                symbolName = elfFile.strtab[symbolEntry.name]

                contextRelocSym = disasmBack.ContextRelocSymbol(rel.offset, symbolName, sectType)
                contextRelocSym.isDefined = True
                contextRelocSym.relocType = rel.rType
                context.relocSymbols[sectType][rel.offset] = contextRelocSym

        # Use the symtab to replace symbol names present in disassembled sections
        for symEntry in elfFile.symtab:
            if symEntry.shndx == 0:
                continue

            sectHeaderEntry = elfFile.sectionHeaders[symEntry.shndx]
            if sectHeaderEntry is None:
                continue
            sectName = elfFile.shstrtab[sectHeaderEntry.name]
            sectType = disasmBack.FileSectionType.fromStr(sectName)
            if sectType != disasmBack.FileSectionType.Invalid:
                subSection = processedFiles[sectType][1]
                symName = elfFile.strtab[symEntry.name]

                contextOffsetSym = disasmBack.ContextOffsetSymbol(symEntry.value, symName, sectType)
                contextOffsetSym.isDefined = True
                # contextOffsetSym.size = symEntry.size
                context.offsetSymbols[sectType][symEntry.value] = contextOffsetSym


    for outputFilePath, subFile in processedFiles.values():
        subFile.analyze()

    for outputFilePath, subFile in processedFiles.values():
        disasmBack.mips.FilesHandlers.writeSection(str(outputFilePath), subFile)


if __name__ == "__main__":
    elfObjDisasmMain()
