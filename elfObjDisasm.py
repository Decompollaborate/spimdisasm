#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import pathlib

import spimdisasm


def elfObjDisasmMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    spimdisasm.GlobalConfig.addParametersToArgParse(parser)

    spimdisasm.mips.instructions.InstructionConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    spimdisasm.mips.instructions.InstructionConfig.parseArgs(args)

    spimdisasm.GlobalConfig.parseArgs(args)

    spimdisasm.GlobalConfig.REMOVE_POINTERS = False
    spimdisasm.GlobalConfig.IGNORE_BRANCHES = False
    spimdisasm.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES = False

    # GlobalConfig.VERBOSE = True

    inputPath = pathlib.Path(args.binary)

    context = spimdisasm.Context()

    array_of_bytes = spimdisasm.Utils.readFileAsBytearray(args.binary)

    elfFile = spimdisasm.elf32.Elf32File(array_of_bytes)

    processedFiles: dict[spimdisasm.FileSectionType, tuple[pathlib.Path, spimdisasm.mips.sections.SectionBase]] = dict()

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    for sectionType, sectionBytes in elfFile.progbits.items():
        outputPath = dataOutput
        if sectionType == spimdisasm.FileSectionType.Text:
            outputPath = textOutput

        outputFilePath = pathlib.Path(outputPath)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        if sectionType == spimdisasm.FileSectionType.Text:
            processedFiles[sectionType] = (outputFilePath, spimdisasm.mips.sections.SectionText(context, None, inputPath.stem, sectionBytes))
        if sectionType == spimdisasm.FileSectionType.Data:
            processedFiles[sectionType] = (outputFilePath, spimdisasm.mips.sections.SectionData(context, None, inputPath.stem, sectionBytes))
        if sectionType == spimdisasm.FileSectionType.Rodata:
            processedFiles[sectionType] = (outputFilePath, spimdisasm.mips.sections.SectionRodata(context, None, inputPath.stem, sectionBytes))

    if elfFile.nobits is not None:
        outputPath = dataOutput

        outputFilePath = pathlib.Path(dataOutput)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        processedFiles[spimdisasm.FileSectionType.Bss] = (outputFilePath, spimdisasm.mips.sections.SectionBss(context, 0, elfFile.nobits, inputPath.stem))

    if elfFile.symtab is not None and elfFile.strtab is not None:
        # Inject symbols from the reloc table referenced in each section
        for sectType, relocs in elfFile.rel.items():
            subSection = processedFiles[sectType][1]
            for rel in relocs:
                symbolEntry = elfFile.symtab[rel.rSym]
                symbolName = elfFile.strtab[symbolEntry.name]

                contextRelocSym = spimdisasm.ContextRelocSymbol(rel.offset, symbolName, sectType)
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
            sectType = spimdisasm.FileSectionType.fromStr(sectName)
            if sectType != spimdisasm.FileSectionType.Invalid:
                subSection = processedFiles[sectType][1]
                symName = elfFile.strtab[symEntry.name]

                contextOffsetSym = spimdisasm.ContextOffsetSymbol(symEntry.value, symName, sectType)
                contextOffsetSym.isDefined = True
                # contextOffsetSym.size = symEntry.size
                context.offsetSymbols[sectType][symEntry.value] = contextOffsetSym


    for outputFilePath, subFile in processedFiles.values():
        subFile.analyze()

    for outputFilePath, subFile in processedFiles.values():
        spimdisasm.mips.FilesHandlers.writeSection(str(outputFilePath), subFile)


if __name__ == "__main__":
    elfObjDisasmMain()
