#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from typing import Callable

from .. import common

from .Elf32Constants import Elf32HeaderIdentifier, Elf32SectionHeaderType
from .Elf32Dyns import Elf32Dyns
from .Elf32GlobalOffsetTable import Elf32GlobalOffsetTable
from .Elf32Header import Elf32Header
from .Elf32RegInfo import Elf32RegInfo
from .Elf32SectionHeaders import Elf32SectionHeaders, Elf32SectionHeaderEntry
from .Elf32StringTable import Elf32StringTable
from .Elf32Syms import Elf32Syms
from .Elf32Rels import Elf32Rels


class Elf32File:
    def __init__(self, array_of_bytes: bytearray):
        self.header = Elf32Header.fromBytearray(array_of_bytes)
        # print(self.header)

        dataEncoding = self.header.ident.getDataEncoding()
        if dataEncoding == Elf32HeaderIdentifier.DataEncoding.DATA2MSB:
            common.GlobalConfig.ENDIAN = common.InputEndian.BIG
        elif dataEncoding == Elf32HeaderIdentifier.DataEncoding.DATA2LSB:
            common.GlobalConfig.ENDIAN = common.InputEndian.LITTLE

        self.strtab: Elf32StringTable | None = None
        self.symtab: Elf32Syms | None = None

        self.dynamic: Elf32Dyns | None = None
        self.dynstr: Elf32StringTable | None = None
        self.dynsym: Elf32Syms | None = None

        self.progbits: dict[common.FileSectionType, Elf32SectionHeaderEntry] = dict()
        self.nobits: Elf32SectionHeaderEntry | None = None

        self.rel: dict[common.FileSectionType, Elf32Rels] = dict()

        self.reginfo: Elf32RegInfo | None = None

        self.sectionHeaders = Elf32SectionHeaders(array_of_bytes, self.header.shoff, self.header.shnum)

        shstrtabSectionEntry = self.sectionHeaders.sections[self.header.shstrndx]
        self.shstrtab = Elf32StringTable(array_of_bytes, shstrtabSectionEntry.offset, shstrtabSectionEntry.size)

        self.got: Elf32GlobalOffsetTable | None = None

        for entry in self.sectionHeaders.sections:
            sectionEntryName = self.shstrtab[entry.name]
            # print(sectionEntryName, end="\t ")
            # print(entry)

            callback = self._sectionProcessorCallbacks.get(entry.type)
            if callback is not None:
                callback(self, array_of_bytes, entry, sectionEntryName)
            elif common.GlobalConfig.VERBOSE:
                common.Utils.eprint("Unknown section header type found:", sectionEntryName, entry, "\n")

        if self.got is not None and self.dynamic is not None and self.dynsym is not None:
            self.got.initTables(self.dynamic, self.dynsym)


    def _processSection_NULL(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        pass

    def _processSection_PROGBITS(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        fileSecType = common.FileSectionType.fromStr(sectionEntryName)

        if fileSecType != common.FileSectionType.Invalid:
            self.progbits[fileSecType] = entry
            if fileSecType == common.FileSectionType.Text:
                self.sectionHeaders.mipsText = entry
            elif fileSecType == common.FileSectionType.Data:
                self.sectionHeaders.mipsData = entry
        elif sectionEntryName == ".got":
            self.got = Elf32GlobalOffsetTable(array_of_bytes, entry.offset, entry.size)
        elif sectionEntryName == ".interp":
            # strings with names of dynamic libraries
            common.Utils.printVerbose(f"Unhandled SYMTAB found: '{sectionEntryName}'")
        elif sectionEntryName == ".MIPS.stubs":
            # ?
            common.Utils.printVerbose(f"Unhandled SYMTAB found: '{sectionEntryName}'")
        elif sectionEntryName == ".init":
            # ?
            common.Utils.printVerbose(f"Unhandled SYMTAB found: '{sectionEntryName}'")
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint(f"Unhandled PROGBITS found: '{sectionEntryName}'", entry, "\n")

    def _processSection_SYMTAB(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName == ".symtab":
            self.symtab = Elf32Syms(array_of_bytes, entry.offset, entry.size)
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled SYMTAB found: ", sectionEntryName, entry, "\n")

    def _processSection_STRTAB(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName == ".strtab":
            self.strtab = Elf32StringTable(array_of_bytes, entry.offset, entry.size)
        elif sectionEntryName == ".dynstr":
            self.dynstr = Elf32StringTable(array_of_bytes, entry.offset, entry.size)
        elif sectionEntryName == ".shstrtab":
            pass
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled STRTAB found: ", sectionEntryName, entry, "\n")

    def _processSection_RELA(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_HASH(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_DYNAMIC(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName == ".dynamic":
            self.dynamic = Elf32Dyns(array_of_bytes, entry.offset, entry.size)
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled DYNAMIC found: ", sectionEntryName, entry, "\n")

    def _processSection_NOTE(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_NOBITS(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName == ".bss":
            self.nobits = entry
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled NOBITS found: ", sectionEntryName, entry, "\n")

    def _processSection_REL(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName.startswith(".rel."):
            fileSecType = common.FileSectionType.fromStr(sectionEntryName[4:])
            if fileSecType != common.FileSectionType.Invalid:
                self.rel[fileSecType] = Elf32Rels(array_of_bytes, entry.offset, entry.size)
            elif common.GlobalConfig.VERBOSE:
                common.Utils.eprint("Unhandled REL subsection found: ", sectionEntryName, entry, "\n")
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled REL found: ", sectionEntryName, entry, "\n")

    def _processSection_DYNSYM(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName == ".dynsym":
            self.dynsym = Elf32Syms(array_of_bytes, entry.offset, entry.size)
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled DYNSYM found: ", sectionEntryName, entry, "\n")


    def _processSection_MIPS_LIBLIST(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_MIPS_MSYM(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_MIPS_GPTAB(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_MIPS_DEBUG(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_MIPS_REGINFO(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        if sectionEntryName == ".reginfo":
            self.reginfo = Elf32RegInfo.fromBytearray(array_of_bytes, entry.offset)
        elif common.GlobalConfig.VERBOSE:
            common.Utils.eprint("Unhandled MIPS_REGINFO found: ", sectionEntryName, entry, "\n")

    def _processSection_MIPS_OPTIONS(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_MIPS_SYMBOL_LIB(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass

    def _processSection_MIPS_ABIFLAGS(self, array_of_bytes: bytearray, entry: Elf32SectionHeaderEntry, sectionEntryName: str) -> None:
        # ?
        pass


    _sectionProcessorCallbacks: dict[int, Callable[[Elf32File, bytearray, Elf32SectionHeaderEntry, str], None]] = {
        Elf32SectionHeaderType.NULL.value: _processSection_NULL,
        Elf32SectionHeaderType.PROGBITS.value: _processSection_PROGBITS,
        Elf32SectionHeaderType.SYMTAB.value: _processSection_SYMTAB,
        Elf32SectionHeaderType.STRTAB.value: _processSection_STRTAB,
        # Elf32SectionHeaderType.RELA.value: _processSection_RELA,
        Elf32SectionHeaderType.HASH.value: _processSection_HASH,
        Elf32SectionHeaderType.DYNAMIC.value: _processSection_DYNAMIC,
        # Elf32SectionHeaderType.NOTE.value: _processSection_NOTE,
        Elf32SectionHeaderType.NOBITS.value: _processSection_NOBITS,
        Elf32SectionHeaderType.REL.value: _processSection_REL,
        Elf32SectionHeaderType.DYNSYM.value: _processSection_DYNSYM,

        Elf32SectionHeaderType.MIPS_LIBLIST.value: _processSection_MIPS_LIBLIST,
        Elf32SectionHeaderType.MIPS_MSYM.value: _processSection_MIPS_MSYM,
        Elf32SectionHeaderType.MIPS_GPTAB.value: _processSection_MIPS_GPTAB,
        Elf32SectionHeaderType.MIPS_DEBUG.value: _processSection_MIPS_DEBUG,
        Elf32SectionHeaderType.MIPS_REGINFO.value: _processSection_MIPS_REGINFO,
        Elf32SectionHeaderType.MIPS_OPTIONS.value: _processSection_MIPS_OPTIONS,
        Elf32SectionHeaderType.MIPS_SYMBOL_LIB.value: _processSection_MIPS_SYMBOL_LIB,
        Elf32SectionHeaderType.MIPS_ABIFLAGS.value: _processSection_MIPS_ABIFLAGS,
    }
