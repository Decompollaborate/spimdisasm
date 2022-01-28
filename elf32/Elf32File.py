#!/usr/bin/python3

from __future__ import annotations


from .Elf32Constants import Elf32SectionHeaderType
from .Elf32Header import Elf32Header
from .Elf32SectionHeaders import Elf32SectionHeaders
from .Elf32StringTable import Elf32StringTable
from .Elf32Syms import Elf32Syms


class Elf32File:
    def __init__(self, array_of_bytes: bytearray):
        self.header = Elf32Header.fromBytearray(array_of_bytes)
        print(self.header)

        self.strtab: Elf32StringTable | None = None
        self.symtab: Elf32Syms | None = None

        # for i in range(header.shnum):
        #     sectionHeaderEntry = Elf32SectionHeaderEntry.fromBytearray(array_of_bytes, header.shoff + i * 0x28)
        #     print(sectionHeaderEntry)

        self.sectionHeaders = Elf32SectionHeaders(array_of_bytes, self.header.shoff, self.header.shnum)

        shstrtabSectionEntry = self.sectionHeaders.sections[self.header.shstrndx]
        self.shstrtab = Elf32StringTable(array_of_bytes, shstrtabSectionEntry.offset, shstrtabSectionEntry.size)

        for entry in self.sectionHeaders.sections:
            sectionEntryName = self.shstrtab[entry.name]
            print(sectionEntryName, end="\t ")
            print(entry)
            if entry.type == Elf32SectionHeaderType.NULL.value:
                continue
            elif entry.type == Elf32SectionHeaderType.PROGBITS.value:
                if sectionEntryName == ".text":
                    # TODO
                    pass
                elif sectionEntryName == ".data":
                    # TODO
                    pass
                elif sectionEntryName == ".rodata":
                    # TODO
                    pass
                else:
                    # TODO: eprint
                    print("Unknown PROGBITS found: ", sectionEntryName, entry)
            elif entry.type == Elf32SectionHeaderType.SYMTAB.value:
                if sectionEntryName == ".symtab":
                    self.symtab = Elf32Syms(array_of_bytes, entry.offset, entry.size)
                    print()
                    print("SYMTAB:")
                    for i, x in enumerate(self.symtab.symbols):
                        print(i, x)
                    print()
                else:
                    # TODO: eprint
                    print("Unknown SYMTAB found: ", sectionEntryName, entry)
            elif entry.type == Elf32SectionHeaderType.STRTAB.value:
                if sectionEntryName == ".strtab":
                    self.strtab = Elf32StringTable(array_of_bytes, entry.offset, entry.size)
                    print()
                    print("STRTAB:")
                    for i, x in enumerate(self.strtab):
                        print(i, x)
                    print()
                elif sectionEntryName == ".shstrtab":
                    pass
                else:
                    # TODO: eprint
                    print("Unknown STRTAB found: ", sectionEntryName, entry)
            # elif entry.type == Elf32SectionHeaderType.RELA.value:
            #     pass
            # elif entry.type == Elf32SectionHeaderType.HASH.value:
            #     pass
            # elif entry.type == Elf32SectionHeaderType.DYNAMIC.value:
            #     pass
            # elif entry.type == Elf32SectionHeaderType.NOTE.value:
            #     pass
            elif entry.type == Elf32SectionHeaderType.NOBITS.value:
                if sectionEntryName == ".bss":
                    # TODO
                    pass
                else:
                    # TODO: eprint
                    print("Unknown NOBITS found: ", sectionEntryName, entry)
            elif entry.type == Elf32SectionHeaderType.REL.value:
                # TODO
                pass
            elif entry.type == Elf32SectionHeaderType.MIPS_DEBUG.value:
                # ?
                pass
            elif entry.type == Elf32SectionHeaderType.MIPS_REGINFO.value:
                # ?
                pass
            elif entry.type == Elf32SectionHeaderType.MIPS_OPTIONS.value:
                # ?
                pass
            else:
                # TODO: eprint
                print("Unknown section header type found: ", entry)
