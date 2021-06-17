#!/usr/bin/python3

from __future__ import annotations

import os
import hashlib
import json
import struct
from typing import List
import sys

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

def get_str_hash(byte_array):
    return str(hashlib.md5(byte_array).hexdigest())

def readFile(filepath):
    with open(filepath) as f:
        return [x.strip() for x in f.readlines()]

def readJson(filepath):
    with open(filepath) as f:
        return json.load(f)

def read_file_as_bytearray(filepath):
    if not os.path.exists(filepath):
        return bytearray(0)
    with open(filepath, mode="rb") as f:
        return bytearray(f.read())

def bytesToBEWords(array_of_bytes: bytearray) -> List[int]:
    words = len(array_of_bytes)//4
    big_endian_format = f">{words}I"
    return list(struct.unpack_from(big_endian_format, array_of_bytes, 0))

def beWordsToBytes(words_list: List[int], buffer: bytearray) -> bytearray:
    words = len(words_list)
    big_endian_format = f">{words}I"
    struct.pack_into(big_endian_format, buffer, 0, *words_list)
    return buffer
