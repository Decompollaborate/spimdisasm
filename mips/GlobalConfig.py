#!/usr/bin/python3

from __future__ import annotations

class GlobalConfig:
    REMOVE_POINTERS: bool = False
    IGNORE_BRANCHES: bool = False # Ignores the address of every branch, jump and jal
    IGNORE_80: bool = False # Ignores words that starts in 0x80
    IGNORE_06: bool = False # Ignores words that starts in 0x06
    IGNORE_04: bool = False # Ignores words that starts in 0x04

    WRITE_BINARY: bool = False # write to files splitted binaries

    ASM_COMMENT: bool = True
    FUNCTION_ASM_COUNT: bool = True

    ADD_NEW_SYMBOLS: bool = True
    PRODUCE_SYMBOLS_PLUS_OFFSET: bool = False
