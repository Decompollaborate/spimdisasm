#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from pathlib import Path
import rabbitizer
import struct
import subprocess
import sys

from .GlobalConfig import GlobalConfig, InputEndian


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

def printQuietless(*args, **kwargs):
    if not GlobalConfig.QUIET:
        print(*args, **kwargs)

def epprintQuietless(*args, **kwargs):
    if not GlobalConfig.QUIET:
        print(*args, file=sys.stderr, **kwargs)


def printVerbose(*args, **kwargs):
    if not GlobalConfig.QUIET and GlobalConfig.VERBOSE:
        print(*args, **kwargs)

def eprintVerbose(*args, **kwargs):
    if not GlobalConfig.QUIET and GlobalConfig.VERBOSE:
        print(*args, file=sys.stderr, **kwargs)

# https://stackoverflow.com/questions/1512457/determining-if-stdout-for-a-python-process-is-redirected
def isStdoutRedirected() -> bool:
    return not sys.stdout.isatty()

# Returns the md5 hash of a bytes
def getStrHash(byte_array: bytes) -> str:
    return str(hashlib.md5(byte_array).hexdigest())

def writeBytesToFile(filepath: Path, array_of_bytes: bytes):
    with filepath.open(mode="wb") as f:
        f.write(array_of_bytes)

#! deprecated
writeBytearrayToFile = writeBytesToFile

def readFileAsBytearray(filepath: Path) -> bytearray:
    if not filepath.exists():
        return bytearray(0)
    with filepath.open(mode="rb") as f:
        return bytearray(f.read())

def readFile(filepath: Path) -> list[str]:
    with filepath.open() as f:
        return [x.strip() for x in f.readlines()]

def readJson(filepath: Path):
    with filepath.open() as f:
        return json.load(f)

def removeExtraWhitespace(line: str) -> str:
    return " ".join(line.split())

def endianessBytesToWords(endian: InputEndian, array_of_bytes: bytes, offset: int=0, offsetEnd: int|None=None) -> list[int]:
    totalBytesCount = len(array_of_bytes)
    if totalBytesCount == 0:
        return list()

    bytesCount = totalBytesCount
    offsetEndHex = "None"
    if offsetEnd is not None and offsetEnd > 0:
        bytesCount = offsetEnd
        offsetEndHex = f"{offsetEnd:X}"
    bytesCount -= offset

    assert bytesCount >= 0, f"{offset:X}, {offsetEndHex}, {bytesCount:X}, {totalBytesCount:X}"
    assert offset + bytesCount <= totalBytesCount, f"{offset:X}, {offsetEndHex}, {bytesCount:X}, {totalBytesCount:X}"

    if endian == InputEndian.MIDDLE:
        # Convert middle endian to big endian
        halfwords = bytesCount//2
        little_byte_format = f"<{halfwords}H"
        big_byte_format = f">{halfwords}H"
        tmp = struct.unpack_from(little_byte_format, array_of_bytes, offset)
        newBytes = bytearray(array_of_bytes)
        struct.pack_into(big_byte_format, newBytes, offset, *tmp)
        array_of_bytes = bytes(newBytes)

    words = bytesCount//4
    endian_format = f">{words}I"
    if endian == InputEndian.LITTLE:
        endian_format = f"<{words}I"
    return list(struct.unpack_from(endian_format, array_of_bytes, offset))

def bytesToWords(array_of_bytes: bytes, offset: int=0, offsetEnd: int|None=None) -> list[int]:
    return endianessBytesToWords(GlobalConfig.ENDIAN, array_of_bytes, offset, offsetEnd)

#! deprecated
bytesToBEWords = bytesToWords

def endianessWordsToBytes(endian: InputEndian, words_list: list[int]) -> bytes:
    if endian == InputEndian.MIDDLE:
        raise BufferError("TODO: wordsToBytesEndianess: GlobalConfig.ENDIAN == InputEndian.MIDDLE")

    words = len(words_list)
    endian_format = f">{words}I"
    if endian == InputEndian.LITTLE:
        endian_format = f"<{words}I"
    return struct.pack(endian_format, *words_list)

def wordsToBytes(words_list: list[int]) -> bytes:
    return endianessWordsToBytes(GlobalConfig.ENDIAN, words_list)

#! deprecated
beWordsToBytes = wordsToBytes

def wordToFloat(word: int) -> float:
    b = struct.pack('>I', word)
    return struct.unpack('>f', b)[0]

def qwordToDouble(qword: int) -> float:
    b = struct.pack('>Q', qword)
    return struct.unpack('>d', b)[0]

def wordToCurrenEndian(word: int) -> int:
    if GlobalConfig.ENDIAN == InputEndian.BIG:
        return word

    if GlobalConfig.ENDIAN == InputEndian.LITTLE:
        b = struct.pack('>I', word)
        return struct.unpack('<I', b)[0]

    # MIDDLE
    b = struct.pack('<2H', word >> 16, word & 0xFFFF)
    first, second = struct.unpack('>2H', b)
    return (first << 16) | second

def qwordToCurrenEndian(word: int) -> int:
    if GlobalConfig.ENDIAN == InputEndian.BIG:
        return word

    if GlobalConfig.ENDIAN == InputEndian.LITTLE:
        b = struct.pack('>Q', word)
        return struct.unpack('<Q', b)[0]

    # MIDDLE
    b = struct.pack('<4H', (word >> 48) & 0xFFFF, (word >> 32) & 0xFFFF, (word >> 16) & 0xFFFF, word & 0xFFFF)
    first, second, third, fourth = struct.unpack('>4H', b)
    return (first << 48) | (second << 32) | (third << 16) | fourth

def runCommandGetOutput(command: str, args: list[str]) -> list[str] | None:
    try:
        output = subprocess.check_output([command, *args]).decode("utf-8")
        return output.strip().split("\n")
    except:
        return None

def readCsv(filepath: Path) -> list[list[str]]:
    data: list[list[str]] = []

    with filepath.open() as f:
        lines = f.readlines()
        processedLines = [x.strip().split("#")[0] for x in lines]
        csvReader = csv.reader(processedLines)
        for row in csvReader:
            data.append(list(row))

    return data

def parseColonSeparatedPairLine(line: str) -> dict[str, str]:
    pairs: dict[str, str] = dict()

    # Allow // and # comments
    line = line.split("//")[0].split("#")[0].strip()

    for info in line.split(" "):
        if ":" not in info:
            continue

        key, *vals = info.split(":")
        pairs[key] = ":".join(vals)

    return pairs

def getMaybeIntFromMaybeStr(number: str|None, base: int=0) -> int|None:
    if number is None:
        return None

    return int(number, base)


TRUEY_VALS = ["true", "on", "yes", "y"]
FALSEY_VALS = ["false", "off", "no", "n"]

def getMaybeBooleyFromMaybeStr(booley: str|None) -> bool|None:
    if booley is None:
        return None

    if booley in TRUEY_VALS:
        return True
    if booley in FALSEY_VALS:
        return False
    return None

# Escape characters that are unlikely to be used
bannedEscapeCharacters = {
    0x00, # '\0'
    0x01,
    0x02,
    0x03,
    0x04,
    0x05,
    0x06,
    0x07, # '\a'
    0x08, # '\b'
    # 0x09, # '\t'
    # 0x0A, # '\n'
    0x0B, # '\v'
    # 0x0C, # '\f'
    # 0x0D, # '\r'
    0x0E,
    0x0F,
    0x10,
    0x11,
    0x12,
    0x13,
    0x14,
    0x15,
    0x16,
    0x17,
    0x18,
    0x19,
    # 0x1A, # Some games use this char as an escape sequence
    # 0x1B, # VT escape sequences
    0x1C,
    0x1D,
    0x1E,
    0x1F,
}

escapeCharactersSpecialCases = {
    0x1A,
    0x1B,
    0x8C,
    0x8D,
}

def decodeBytesToStrings(buf: bytes, offset: int, stringEncoding: str, terminator: int=0) -> tuple[list[str], int]:
    result = []

    dst = bytearray()
    i = 0
    while offset + i < len(buf) and buf[offset + i] != terminator:
        char = buf[offset + i]
        if char in bannedEscapeCharacters:
            return [], -1
        elif char in escapeCharactersSpecialCases:
            if dst:
                try:
                    decoded = rabbitizer.Utils.escapeString(dst.decode(stringEncoding))
                except UnicodeDecodeError:
                    return [], -1
                result.append(decoded)
                dst.clear()
            result.append(f"\\x{char:02X}")
        else:
            dst.append(char)
        i += 1

    if offset + i >= len(buf):
        # Reached the end of the buffer without finding an 0
        return [], -1

    if dst:
        try:
            decoded = rabbitizer.Utils.escapeString(dst.decode(stringEncoding))
        except UnicodeDecodeError:
            return [], -1
        result.append(decoded)

    # To be a valid aligned string, the next word-aligned bytes needs to be zero
    checkStartOffset = offset + i
    checkEndOffset = min((checkStartOffset & ~3) + 4, len(buf))
    while checkStartOffset < checkEndOffset:
        if buf[checkStartOffset] != terminator:
            return [], -1
        checkStartOffset += 1

    return result, i

def decodeBytesToPascalStrings(buf: bytes, offset: int, stringEncoding: str, terminator: int=0x20) -> tuple[list[str], int]:
    result = []

    dst = bytearray()
    i = 0
    while offset + i < len(buf):
        if buf[offset + i] == terminator:
            if offset + i + 1 < len(buf) and buf[offset + i + 1] == terminator:
                # Require at least 2 terminators next to each other to actually consider the string has ended
                break
        char = buf[offset + i]
        if char in bannedEscapeCharacters:
            return [], -1
        elif char in escapeCharactersSpecialCases:
            if dst:
                try:
                    decoded = rabbitizer.Utils.escapeString(dst.decode(stringEncoding))
                except UnicodeDecodeError:
                    return [], -1
                result.append(decoded)
                dst.clear()
            result.append(f"\\x{char:02X}")
        else:
            dst.append(char)
        i += 1

    if offset + i >= len(buf):
        # Reached the end of the buffer without finding an 0
        return [], -1

    # To be a valid aligned string, the next word-aligned bytes needs to be the terminator value
    checkStartOffset = offset + i
    checkEndOffset = min((checkStartOffset & ~3) + 4, len(buf))
    while checkStartOffset < checkEndOffset:
        if buf[checkStartOffset] != terminator:
            return [], -1
        dst.append(buf[checkStartOffset])
        checkStartOffset += 1
        i += 1

    while offset + i < len(buf):
        # Check in chunks of 4 bytes for the terminator value
        j = 0
        onlyTerminator = True
        while j < 4 and offset + i + j < len(buf):
            char = buf[offset + i + j]
            if char != terminator:
                onlyTerminator = False
                break
            j += 1

        if not onlyTerminator:
            break
        dst.extend([terminator] * 4)
        i += 4

    if dst:
        try:
            decoded = rabbitizer.Utils.escapeString(dst.decode(stringEncoding))
        except UnicodeDecodeError:
            return [], -1
        result.append(decoded)

    return result, i


#! @deprecated
def decodeString(buf: bytes, offset: int, stringEncoding: str) -> tuple[list[str], int]:
    result = []

    dst = bytearray()
    i = 0
    while offset + i < len(buf) and buf[offset + i] != 0:
        char = buf[offset + i]
        if char in bannedEscapeCharacters:
            raise RuntimeError()
        elif char in escapeCharactersSpecialCases:
            if dst:
                decoded = rabbitizer.Utils.escapeString(dst.decode(stringEncoding))
                result.append(decoded)
                dst.clear()
            result.append(f"\\x{char:02X}")
        else:
            dst.append(char)
        i += 1

    if offset + i > len(buf):
        raise RuntimeError("Reached the end of the buffer without finding an 0")

    if dst:
        decoded = rabbitizer.Utils.escapeString(dst.decode(stringEncoding))
        result.append(decoded)
    return result, i


# Copied from argparse.py to be able to use it on Python versions < 3.9
class BooleanOptionalAction(argparse.Action):
    def __init__(self,
                 option_strings,
                 dest,
                 default=None,
                 type=None,
                 choices=None,
                 required=False,
                 help=None,
                 metavar=None):

        _option_strings = []
        for option_string in option_strings:
            _option_strings.append(option_string)

            if option_string.startswith('--'):
                option_string = '--no-' + option_string[2:]
                _option_strings.append(option_string)

        if help is not None and default is not None:
            help += " (default: %(default)s)"

        super().__init__(
            option_strings=_option_strings,
            dest=dest,
            nargs=0,
            default=default,
            type=type,
            choices=choices,
            required=required,
            help=help,
            metavar=metavar)

    def __call__(self, parser, namespace, values, option_string: str|None=None):
        if option_string is not None and option_string in self.option_strings:
            setattr(namespace, self.dest, not option_string.startswith('--no-'))

    def format_usage(self):
        return ' | '.join(self.option_strings)

# https://stackoverflow.com/a/35925919/6292472
class PreserveWhiteSpaceWrapRawTextHelpFormatter(argparse.RawDescriptionHelpFormatter):
    def __add_whitespace(self, idx, iWSpace, text):
        if idx == 0:
            return text
        return (" " * iWSpace) + text

    def _split_lines(self, text, width):
        import textwrap
        import re
        textRows = text.splitlines()
        for idx, line in enumerate(textRows):
            search = re.search('\s*[0-9\-]{0,}\.?\s*', line)
            if line.strip() == "":
                textRows[idx] = " "
            elif search:
                lWSpace = search.end()
                lines = [self.__add_whitespace(i,lWSpace,x) for i,x in enumerate(textwrap.wrap(line, width))]
                textRows[idx] = lines

        return [item for sublist in textRows for item in sublist]
