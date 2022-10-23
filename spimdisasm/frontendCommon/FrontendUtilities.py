#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from pathlib import Path
from typing import Callable

from .. import common
from .. import mips


_sLenLastLine = 80

def migrateFunctions(processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]], functionMigrationPath: Path, progressCallback: Callable[[int, str, int], None]|None=None):
    funcTotal = sum(len(x.symbolList) for x in processedFiles[common.FileSectionType.Text])
    rodataFileList = processedFiles[common.FileSectionType.Rodata]
    i = 0
    for f in processedFiles[common.FileSectionType.Text]:
        for func in f.symbolList:
            if progressCallback is not None:
                progressCallback(i, func.getName(), funcTotal)

            assert isinstance(func, mips.symbols.SymbolFunction)
            functionPath = functionMigrationPath / f.name
            mips.FilesHandlers.writeSplitedFunction(functionPath, func, rodataFileList)

            i += 1
    mips.FilesHandlers.writeOtherRodata(functionMigrationPath, rodataFileList)

def progressCallback_migrateFunctions(i: int, funcName: str, funcTotal: int) -> None:
    global _sLenLastLine

    common.Utils.printVerbose(f"Spliting {funcName}", end="")
    common.Utils.printQuietless(_sLenLastLine*" " + "\r", end="")
    common.Utils.printVerbose()
    progressStr = f" Writing: {i/funcTotal:%}. Function: {funcName}\r"
    _sLenLastLine = max(len(progressStr), _sLenLastLine)
    common.Utils.printQuietless(progressStr, end="")
