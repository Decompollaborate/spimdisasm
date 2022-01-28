#!/usr/bin/python3

from __future__ import annotations

import argparse

from mips.Utils import *

from elf32.Elf32File import Elf32File


def elfObjDisasmMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")

    args = parser.parse_args()


    array_of_bytes = readFileAsBytearray(args.binary)

    elfFile = Elf32File(array_of_bytes)


if __name__ == "__main__":
    elfObjDisasmMain()
