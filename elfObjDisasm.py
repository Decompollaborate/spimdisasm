#!/usr/bin/python3

from __future__ import annotations

import argparse

from backend.common.Utils import *

from backend.elf32.Elf32File import Elf32File


def elfObjDisasmMain():
    # TODO
    description = ""
    parser = argparse.ArgumentParser(description=description)

    parser.add_argument("binary", help="Path to input elf binary file")
    # parser.add_argument("output", help="Path to output. Use '-' to print to stdout instead")

    args = parser.parse_args()


    array_of_bytes = readFileAsBytearray(args.binary)

    elfFile = Elf32File(array_of_bytes)


if __name__ == "__main__":
    elfObjDisasmMain()
