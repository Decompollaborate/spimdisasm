#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from pathlib import Path
from typing import Callable

from .. import common
from .. import mips


ProgressCallbackType = Callable[[int, str, int], None]


_sLenLastLine = 80


def analyzeProcessedFiles(processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]], processedFilesOutputPaths: dict[common.FileSectionType, list[Path]], processedFilesCount: int, progressCallback: ProgressCallbackType|None=None):
    i = 0
    for sectionType, filesInSection in sorted(processedFiles.items()):
        pathLists = processedFilesOutputPaths[sectionType]
        for fileIndex, f in enumerate(filesInSection):
            if progressCallback is not None:
                filePath = pathLists[fileIndex]
                progressCallback(i, str(filePath), processedFilesCount)
            f.analyze()
            f.printAnalyzisResults()

            i += 1
    return

def progressCallback_analyzeProcessedFiles(i: int, filePath: str, processedFilesCount: int) -> None:
    global _sLenLastLine

    common.Utils.printQuietless(_sLenLastLine*" " + "\r", end="")
    progressStr = f"Analyzing: {i/processedFilesCount:%}. File: {filePath}\r"
    _sLenLastLine = max(len(progressStr), _sLenLastLine)
    common.Utils.printQuietless(progressStr, end="", flush=True)
    common.Utils.printVerbose("")


def nukePointers(processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]], processedFilesOutputPaths: dict[common.FileSectionType, list[Path]], processedFilesCount: int, progressCallback: ProgressCallbackType|None=None):
    i = 0
    for sectionType, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[sectionType]
        for fileIndex, f in enumerate(filesInSection):
            if progressCallback is not None:
                filePath = pathLists[fileIndex]
                progressCallback(i, str(filePath), processedFilesCount)
            f.removePointers()
            i += 1
    return

def progressCallback_nukePointers(i: int, filePath: str, processedFilesCount: int) -> None:
    global _sLenLastLine

    common.Utils.printVerbose(f"Nuking pointers of {filePath}")
    common.Utils.printQuietless(_sLenLastLine*" " + "\r", end="")
    progressStr = f" Nuking pointers: {i/processedFilesCount:%}. File: {filePath}\r"
    _sLenLastLine = max(len(progressStr), _sLenLastLine)
    common.Utils.printQuietless(progressStr, end="")


def writeProcessedFiles(processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]], processedFilesOutputPaths: dict[common.FileSectionType, list[Path]], processedFilesCount: int, progressCallback: ProgressCallbackType|None=None):
    common.Utils.printVerbose("Writing files...")
    i = 0
    for section, filesInSection in processedFiles.items():
        pathLists = processedFilesOutputPaths[section]
        for fileIndex, f in enumerate(filesInSection):
            filePath = pathLists[fileIndex]
            if progressCallback is not None:
                progressCallback(i, str(filePath), processedFilesCount)

            mips.FilesHandlers.writeSection(filePath, f)
            i += 1
    return

def progressCallback_writeProcessedFiles(i: int, filePath: str, processedFilesCount: int) -> None:
    global _sLenLastLine

    common.Utils.printVerbose(f"Writing {filePath}")
    common.Utils.printQuietless(_sLenLastLine*" " + "\r", end="")
    progressStr = f"Writing: {i/processedFilesCount:%}. File: {filePath}\r"
    _sLenLastLine = max(len(progressStr), _sLenLastLine)
    common.Utils.printQuietless(progressStr, end="")

    if str(filePath) == "-":
        common.Utils.printQuietless()


def migrateFunctions(processedFiles: dict[common.FileSectionType, list[mips.sections.SectionBase]], functionMigrationPath: Path, progressCallback: ProgressCallbackType|None=None):
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
