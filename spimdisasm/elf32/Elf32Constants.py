#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import enum


# a.k.a. EI
class Elf32HeaderIdentifier:
    @enum.unique
    class FileClass(enum.Enum):
        # EI_CLASS    4        /* File class byte index */
        CLASSNONE   = 0 # Invalid class
        CLASS32     = 1 # 32-bit objects
        CLASS64     = 2 # 64-bit objects
        CLASSNUM    = 3

    @enum.unique
    class DataEncoding(enum.Enum):
        # EI_DATA        5        /* Data encoding byte index */
        DATANONE    = 0 # Invalid data encoding
        DATA2LSB    = 1 # 2's complement, little endian
        DATA2MSB    = 2 # 2's complement, big endian
        DATANUM     = 3 #

    class OsAbi(enum.Enum):
        # EI_OSABI    7        /* OS ABI identification */
        NONE        =   0 # UNIX System V ABI
        SYSV        =   0 # Alias.
        HPUX        =   1 # HP-UX
        NETBSD      =   2 # NetBSD.
        GNU         =   3 # Object uses GNU ELF extensions.
        LINUX       = GNU # Compatibility alias.
        SOLARIS     =   6 # Sun Solaris.
        AIX         =   7 # IBM AIX.
        IRIX        =   8 # SGI Irix.
        FREEBSD     =   9 # FreeBSD.
        TRU64       =  10 # Compaq TRU64 UNIX.
        MODESTO     =  11 # Novell Modesto.
        OPENBSD     =  12 # OpenBSD.
        ARM_AEABI   =  64 # ARM EABI
        ARM         =  97 # ARM
        STANDALONE  = 255 # Standalone (embedded) application


# ET (object file type)
class Elf32ObjectFileType(enum.Enum):
    NONE            = 0 # No file type
    REL             = 1 # Relocatable file
    EXEC            = 2 # Executable file
    DYN             = 3 # Shared object file
    CORE            = 4 # Core file
    NUM             = 5 # Number of defined types
    # LOOS		0xfe00		/* OS-specific range start */
    # HIOS		0xfeff		/* OS-specific range end */
    # LOPROC	0xff00		/* Processor-specific range start */
    # HIPROC	0xffff		/* Processor-specific range end */


# Legal values for e_flags field of Elf32_Ehdr
class Elf32HeaderFlag(enum.Enum):
    NOREORDER       = 0x00000001 # A .noreorder directive was used.
    PIC             = 0x00000002 # Contains PIC code.
    CPIC            = 0x00000004 # Uses PIC calling sequence.
    XGOT            = 0x00000008
    F_64BIT_WHIRL   = 0x00000010
    ABI2            = 0x00000020
    ABI_ON32        = 0x00000040

    _32BITSMODE     = 0x00000100
    FP64            = 0x00000200 # Uses FP64 (12 callee-saved).
    NAN2008         = 0x00000400 # Uses IEEE 754-2008 NaN encoding.

    ARCH            = 0xF0000000 # MIPS architecture level.

    # Legal values for MIPS architecture level
    ARCH_1          = 0x00000000 # -mips1 code.
    ARCH_2          = 0x10000000 # -mips2 code.
    ARCH_3          = 0x20000000 # -mips3 code.
    ARCH_4          = 0x30000000 # -mips4 code.
    ARCH_5          = 0x40000000 # -mips5 code.
    ARCH_32         = 0x50000000 # MIPS32 code.
    ARCH_64         = 0x60000000 # MIPS64 code.
    ARCH_32R2       = 0x70000000 # MIPS32r2 code.
    ARCH_64R2       = 0x80000000 # MIPS64r2 code.


    @staticmethod
    def parseFlags(rawFlags: int) -> tuple[list[Elf32HeaderFlag], int]:
        flagsToCheck = {
            Elf32HeaderFlag.NOREORDER, Elf32HeaderFlag.PIC, Elf32HeaderFlag.CPIC,
            Elf32HeaderFlag.XGOT, Elf32HeaderFlag.F_64BIT_WHIRL, Elf32HeaderFlag.ABI2,
            Elf32HeaderFlag.ABI_ON32,
            Elf32HeaderFlag._32BITSMODE, Elf32HeaderFlag.FP64, Elf32HeaderFlag.NAN2008}
        parsedFlags: list[Elf32HeaderFlag] = list()

        for flagEnum in flagsToCheck:
            if rawFlags & flagEnum.value:
                parsedFlags.append(flagEnum)
                rawFlags &= ~flagEnum.value

        archLevel = rawFlags & Elf32HeaderFlag.ARCH.value
        rawFlags &= ~Elf32HeaderFlag.ARCH.value
        if archLevel == Elf32HeaderFlag.ARCH_1.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_1)
        elif archLevel == Elf32HeaderFlag.ARCH_2.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_2)
        elif archLevel == Elf32HeaderFlag.ARCH_3.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_3)
        elif archLevel == Elf32HeaderFlag.ARCH_4.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_4)
        elif archLevel == Elf32HeaderFlag.ARCH_5.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_5)
        elif archLevel == Elf32HeaderFlag.ARCH_32.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_32)
        elif archLevel == Elf32HeaderFlag.ARCH_64.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_64)
        elif archLevel == Elf32HeaderFlag.ARCH_32R2.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_32R2)
        elif archLevel == Elf32HeaderFlag.ARCH_64R2.value:
            parsedFlags.append(Elf32HeaderFlag.ARCH_64R2)
        else:
            rawFlags |= archLevel

        return parsedFlags, rawFlags


# a.k.a. SHT (section header type)
@enum.unique
class Elf32SectionHeaderType(enum.Enum):
    NULL            =  0
    PROGBITS        =  1
    SYMTAB          =  2
    STRTAB          =  3
    RELA            =  4
    HASH            =  5
    DYNAMIC         =  6
    NOTE            =  7
    NOBITS          =  8
    REL             =  9
    DYNSYM          = 11

    MIPS_LIBLIST    = 0x70000000
    MIPS_MSYM       = 0x70000001
    MIPS_GPTAB      = 0x70000003
    MIPS_DEBUG      = 0x70000005
    MIPS_REGINFO    = 0x70000006
    MIPS_OPTIONS    = 0x7000000D
    MIPS_SYMBOL_LIB = 0x70000020
    MIPS_ABIFLAGS   = 0x7000002A


# a.k.a. STT (symbol table type)
@enum.unique
class Elf32SymbolTableType(enum.Enum):
    NOTYPE       =  0
    OBJECT       =  1
    FUNC         =  2
    SECTION      =  3
    FILE         =  4
    COMMON       =  5
    TLS          =  6
    NUM          =  7

# a.k.a. STB (symbol table binding)
@enum.unique
class Elf32SymbolTableBinding(enum.Enum):
    LOCAL       =  0
    GLOBAL      =  1
    WEAK        =  2
    LOOS        = 10
    HIOS        = 12
    LOPROC      = 13
    HIPROC      = 14

    @staticmethod
    def fromValue(value: int) -> Elf32SymbolTableBinding|None:
        try:
            return Elf32SymbolTableBinding(value)
        except ValueError:
            return None


# Symbol visibility specification encoded in the st_other field.
class Elf32SymbolVisibility(enum.Enum):
    DEFAULT         =    0 # Default symbol visibility rules
    INTERNAL        =    1 # Processor specific hidden class
    HIDDEN          =    2 # Sym unavailable in other modules
    PROTECTED       =    3 # Not preemptible, not exported
    PLT             =    8
    SC_ALIGN_UNUSED = 0xFF

    @staticmethod
    def fromValue(value: int) -> Elf32SymbolVisibility|None:
        try:
            return Elf32SymbolVisibility(value)
        except ValueError:
            return None


# a.k.a. SHN (section header number)
@enum.unique
class Elf32SectionHeaderNumber(enum.Enum):
    UNDEF           = 0
    ABS             = 0xFFF1
    COMMON          = 0xFFF2
    MIPS_ACOMMON    = 0xFF00
    MIPS_TEXT       = 0xFF01
    MIPS_DATA       = 0xFF02

    @staticmethod
    def fromValue(value: int) -> Elf32SectionHeaderNumber|None:
        try:
            return Elf32SectionHeaderNumber(value)
        except ValueError:
            return None


# a.k.a. DT
@enum.unique
class Elf32DynamicTable(enum.Enum):
    NULL                = 0
    "Marks end of dynamic section"
    PLTGOT              = 3
    "Processor defined value"

    MIPS_LOCAL_GOTNO    = 0x7000000A
    "Number of local GOT entries"
    MIPS_SYMTABNO       = 0x70000011
    "Number of DYNSYM entries"
    MIPS_GOTSYM         = 0x70000013
    "First GOT entry in DYNSYM"
