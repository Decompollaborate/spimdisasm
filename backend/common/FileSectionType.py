#!/usr/bin/python3

from __future__ import annotations

import enum


@enum.unique
class FileSectionType(enum.Enum):
    Invalid = -1

    Text    = enum.auto()
    Data    = enum.auto()
    Rodata  = enum.auto()
    Bss     = enum.auto()
    Reloc   = enum.auto()

    def toStr(self) -> str:
        if self == FileSectionType.Text:
            return ".text"
        if self == FileSectionType.Data:
            return ".data"
        if self == FileSectionType.Rodata:
            return ".rodata"
        if self == FileSectionType.Bss:
            return ".bss"
        if self == FileSectionType.Reloc:
            return ".reloc"
        return ""

    @staticmethod
    def fromStr(x: str) -> FileSectionType:
        if x == ".text":
            return FileSectionType.Text
        if x == ".data":
            return FileSectionType.Data
        if x == ".rodata":
            return FileSectionType.Rodata
        if x == ".bss":
            return FileSectionType.Bss
        if x == ".reloc":
            return FileSectionType.Reloc
        return FileSectionType.Invalid
