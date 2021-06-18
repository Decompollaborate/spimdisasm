#!/usr/bin/python3

from __future__ import annotations

class GlobalConfig:
    REMOVE_POINTERS: bool = False
    IGNORE_BRANCHES: bool = False # Ignores the address of every branch, jump and jal
    IGNORE_80: bool = False # Ignores words that starts in 0x80
    IGNORE_06: bool = False # Ignores words that starts in 0x06
    IGNORE_04: bool = False # Ignores words that starts in 0x04

    TRACK_REGISTERS: int = 8 # Set for how many instructions a register will be tracked.
    DELETE_OPENDISPS: bool = False # Will try to find and delete every function that calls Graph_OpenDisps.

    WRITE_BINARY: bool = True # write to files splitted binaries
