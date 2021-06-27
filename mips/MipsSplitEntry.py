#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class SplitEntry:
    def __init__(self, version: str, filename: str, offset: int, size: int = -1, vram: int = -1):
        self.version: str = version
        self.filename: str = filename
        self.offset: int = offset
        self.size: int = size
        self.vram: int = vram

def readSplitsFromCsv(csvfilename: str) -> Dict[str, Dict[str, SplitEntry]]:
    code_splits_file = readCsv(csvfilename)

    header = code_splits_file[0][3::3]
    splits: Dict[str, Dict[str, SplitEntry]] = { h: dict() for h in header }

    for i in range(2, len(code_splits_file)):
        row = code_splits_file[i]
        filename1, filename2, _, *data = row
        name = filename1 or filename2
        if name == "":
            continue

        for i in range(len(header)):
            h = header[i]
            if h == "":
                continue
            offset, vram, size = data[i*3:(i+1)*3]
            try:
                offset = int(offset, 16)
            except:
                continue

            try:
                size = int(size, 16)
            except:
                size = -1

            try:
                vram = int(vram, 16)
            except:
                vram = -1

            splits[h][name] = SplitEntry(h, name, offset, size, vram)
    return splits

def getFileStartsFromEntries(splits: Dict[str, SplitEntry], fileEndOffset: int) -> List[Tuple[int, int, str]]:
    starts = list()
    for filename, entry in splits.items():
        starts.append((entry.offset, entry.size, filename))
    starts.append((fileEndOffset, 0, "end"))

    starts.sort()

    i = 0
    while i < len(starts) - 1:
        start, size, filename = starts[i]
        nextStart, _, _ = starts[i+1]

        end = start + size
        if size < 0:
            end = nextStart
            starts[i] = (start, nextStart-start, filename)

        if end < nextStart:
            starts.insert(i+1, (end, -1, f"file_{toHex(end, 6)}"))

        i += 1

    return starts
