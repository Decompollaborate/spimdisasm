#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import pathlib

import spimdisasm


def insertSymbolsIntoContext(context: spimdisasm.common.Context, symbolTable: spimdisasm.elf32.Elf32Syms, stringTable: spimdisasm.elf32.Elf32StringTable, elfFile: spimdisasm.elf32.Elf32File, isDynamic: bool):
    # Use the symbol table to replace symbol names present in disassembled sections
    for symEntry in symbolTable:
        symName = stringTable[symEntry.name]

        if isDynamic:
            if symEntry.value == 0:
                continue

            if symEntry.stType == spimdisasm.elf32.Elf32SymbolTableType.FUNC.value:
                contextSym = context.globalSegment.addFunction(symEntry.value)
                contextSym.name = symName
                contextSym.isUserDeclared = True
                # contextSym.setSizeIfUnset(symEntry.size)
            elif symEntry.stType == spimdisasm.elf32.Elf32SymbolTableType.OBJECT.value:
                contextSym = context.globalSegment.addSymbol(symEntry.value)
                contextSym.name = symName
                contextSym.isUserDeclared = True
                # contextSym.setSizeIfUnset(symEntry.size)
            elif symEntry.stType == spimdisasm.elf32.Elf32SymbolTableType.SECTION.value:
                # print(symEntry)
                pass
            else:
                spimdisasm.common.Utils.eprint(f"Warning: symbol '{symName}' has an unhandled stType: '{symEntry.stType}'")
                contextSym = context.globalSegment.addSymbol(symEntry.value)
                contextSym.name = symName
                contextSym.isUserDeclared = True
                # contextSym.setSizeIfUnset(symEntry.size)

            continue

        if symEntry.shndx == 0:
            continue

        sectHeaderEntry = elfFile.sectionHeaders[symEntry.shndx]
        if sectHeaderEntry is None:
            continue
        sectName = elfFile.shstrtab[sectHeaderEntry.name]
        sectType = spimdisasm.common.FileSectionType.fromStr(sectName)
        if sectType != spimdisasm.common.FileSectionType.Invalid:
            # subSection = processedFiles[sectType][1]

            contextOffsetSym = spimdisasm.common.ContextOffsetSymbol(symEntry.value, symName, sectType)
            contextOffsetSym.isUserDeclared = True
            context.offsetSymbols[sectType][symEntry.value] = contextOffsetSym
        else:
            spimdisasm.common.Utils.eprint(f"symbol referencing invalid section '{sectName}'")



def elfObjDisasmMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    parser.add_argument("--data-output", help="Path to output the data and rodata disassembly")

    spimdisasm.common.GlobalConfig.addParametersToArgParse(parser)

    spimdisasm.mips.InstructionConfig.addParametersToArgParse(parser)

    args = parser.parse_args()

    spimdisasm.mips.InstructionConfig.parseArgs(args)

    spimdisasm.common.GlobalConfig.parseArgs(args)

    spimdisasm.common.GlobalConfig.REMOVE_POINTERS = False
    spimdisasm.common.GlobalConfig.IGNORE_BRANCHES = False
    spimdisasm.common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES = False

    # GlobalConfig.VERBOSE = True

    inputPath = pathlib.Path(args.binary)

    context = spimdisasm.common.Context()
    context.globalSegment.changeRanges(0x0, 0xFFFFFFFF, 0x0, 0xFFFFFFFF)

    array_of_bytes = spimdisasm.common.Utils.readFileAsBytearray(args.binary)

    elfFile = spimdisasm.elf32.Elf32File(array_of_bytes)

    processedFiles: dict[spimdisasm.common.FileSectionType, tuple[pathlib.Path, spimdisasm.mips.sections.SectionBase]] = dict()

    textOutput = args.output
    dataOutput = args.data_output
    if dataOutput is None:
        dataOutput = textOutput

    for sectionType, sectionEntry in elfFile.progbits.items():
        outputPath = dataOutput
        if sectionType == spimdisasm.common.FileSectionType.Text:
            outputPath = textOutput

        outputFilePath = pathlib.Path(outputPath)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        vromStart = sectionEntry.offset
        vromEnd = vromStart + sectionEntry.size
        addr = sectionEntry.addr

        mipsSection: spimdisasm.mips.sections.SectionBase
        if sectionType == spimdisasm.common.FileSectionType.Text:
            mipsSection = spimdisasm.mips.sections.SectionText(context, vromStart, vromEnd, addr, inputPath.stem, array_of_bytes, 0, None)
        elif sectionType == spimdisasm.common.FileSectionType.Data:
            mipsSection = spimdisasm.mips.sections.SectionData(context, vromStart, vromEnd, addr, inputPath.stem, array_of_bytes, 0, None)
        elif sectionType == spimdisasm.common.FileSectionType.Rodata:
            mipsSection = spimdisasm.mips.sections.SectionRodata(context, vromStart, vromEnd, addr, inputPath.stem, array_of_bytes, 0, None)
        else:
            spimdisasm.common.Utils.eprint(f"Error! Invalid section type '{sectionType}'")
            exit(-1)
        mipsSection.setCommentOffset(vromStart)
        processedFiles[sectionType] = (outputFilePath, mipsSection)

    if elfFile.nobits is not None:
        outputPath = dataOutput

        outputFilePath = pathlib.Path(dataOutput)
        if outputPath != "-":
            outputFilePath /= inputPath.stem

        vromStart = elfFile.nobits.offset
        vromEnd = vromStart + elfFile.nobits.size
        bssStart = elfFile.nobits.addr
        bssEnd = bssStart + elfFile.nobits.size
        mipsSection = spimdisasm.mips.sections.SectionBss(context, vromStart, vromEnd, bssStart, bssEnd, inputPath.stem, 0, None)
        mipsSection.setCommentOffset(vromStart)
        processedFiles[spimdisasm.common.FileSectionType.Bss] = (outputFilePath, mipsSection)

    if elfFile.symtab is not None and elfFile.strtab is not None:
        # Inject symbols from the reloc table referenced in each section
        for sectType, relocs in elfFile.rel.items():
            # subSection = processedFiles[sectType][1]
            for rel in relocs:
                symbolEntry = elfFile.symtab[rel.rSym]
                symbolName = elfFile.strtab[symbolEntry.name]

                contextRelocSym = spimdisasm.common.ContextRelocSymbol(rel.offset, symbolName, sectType)
                contextRelocSym.isDefined = True
                contextRelocSym.relocType = rel.rType
                context.relocSymbols[sectType][rel.offset] = contextRelocSym

        # Use the symtab to replace symbol names present in disassembled sections
        insertSymbolsIntoContext(context, elfFile.symtab, elfFile.strtab, elfFile, False)

    if elfFile.dynsym is not None and elfFile.dynstr is not None:
        # Use the dynsym to replace symbol names present in disassembled sections
        insertSymbolsIntoContext(context, elfFile.dynsym, elfFile.dynstr, elfFile, True)

    if elfFile.reginfo is not None:
        spimdisasm.common.GlobalConfig.GP_VALUE = elfFile.reginfo.gpValue

    context.gotGlobalsTable = elfFile.gotGlobalsTable
    context.gotLocalsTable = elfFile.gotLocalsTable
    if elfFile.dynamic is not None:
        context.gotStart = elfFile.dynamic.pltGot

    for outputFilePath, subFile in processedFiles.values():
        subFile.analyze()

    for outputFilePath, subFile in processedFiles.values():
        spimdisasm.mips.FilesHandlers.writeSection(str(outputFilePath), subFile)


if __name__ == "__main__":
    elfObjDisasmMain()
