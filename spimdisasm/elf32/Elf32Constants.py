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


# a.k.a. SHN (section header number)
@enum.unique
class Elf32SectionHeaderNumber(enum.Enum):
    UNDEF           = 0
    COMMON          = 0xFFF2
    MIPS_ACOMMON    = 0xFF00
    MIPS_TEXT       = 0xFF01
    MIPS_DATA       = 0xFF02


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
