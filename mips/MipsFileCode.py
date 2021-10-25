#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig

from .MipsText import Text
from .MipsData import Data
from .MipsRodata import Rodata
from .MipsBss import Bss
from .MipsFileGeneric import FileGeneric
from .MipsContext import Context
from .MipsSplitEntry import SplitEntry, getFileStartsFromEntries

from .ZeldaOffsets import codeVramStart, codeDataStart, codeRodataStart


class FileCode(FileGeneric):
    def __init__(self, array_of_bytes: bytearray, version: str, context: Context, textSplits: Dict[str, SplitEntry] = {}, dataSplits: Dict[str, SplitEntry] = {}, rodataSplits: Dict[str, SplitEntry] = {}, bssSplits: Dict[str, SplitEntry] = {}):
        super().__init__(array_of_bytes, "code", version, context)

        self.vRamStart = codeVramStart.get(version, -1)

        text_start = 0
        data_start = codeDataStart.get(version, -1)
        rodata_start = codeRodataStart.get(version, -1)
        # bss_start = codeBssStart.get(version, -1)
        bss_start = self.size

        vramSegmentEnd = 0x80FFFFFF

        # TODO: remove
        textStarts = getFileStartsFromEntries(textSplits, data_start)
        dataStarts = getFileStartsFromEntries(dataSplits, rodata_start)
        rodataStarts = getFileStartsFromEntries(rodataSplits, bss_start)
        bssStarts = getFileStartsFromEntries(bssSplits, self.size)

        ## MM stuff
        if "code" in context.segments:
            for start, end, subsectionName, _ in context.segments["code"].subsections:
                #print(hex(start), hex(end), subsectionName)
                if self.vRamStart == -1:
                    self.vRamStart = start
                    vramSegmentEnd = end

                if subsectionName == "text":
                    text_start = start - self.vRamStart
                if subsectionName == "data":
                    data_start = start - self.vRamStart
                if subsectionName == "rodata":
                    rodata_start = start - self.vRamStart
                if subsectionName == "bss":
                    bss_start = start - self.vRamStart

                vramSegmentEnd = max(vramSegmentEnd, end)

        if self.vRamStart != -1:
            sortedFiles = sorted(context.files.items(), key=lambda x: x[1].vram)
            for i, x in enumerate(sortedFiles):
                subfileName, subfileData = x
                if subfileData.vram < self.vRamStart:
                    continue
                if subfileData.vram >= vramSegmentEnd:
                    break

                start = subfileData.vram - self.vRamStart
                size = vramSegmentEnd - subfileData.vram
                if i+1 < len(sortedFiles):
                    size = sortedFiles[i+1][1].vram - subfileData.vram
                filename = subfileName

                data = (start, size, filename)
                #print(hex(start), hex(size), subfileName)

                if text_start <= start < data_start:
                    textStarts.append(data)
                elif data_start <= start < rodata_start:
                    dataStarts.append(data)
                if rodata_start <= start < bss_start:
                    rodataStarts.append(data)
                if bss_start <= start:
                    bssStarts.append(data)
        else:
            textStarts = getFileStartsFromEntries(textSplits, data_start)
            dataStarts = getFileStartsFromEntries(dataSplits, rodata_start)
            rodataStarts = getFileStartsFromEntries(rodataSplits, bss_start)
            bssStarts = getFileStartsFromEntries(bssSplits, self.size)
            if len(textSplits) == 0:
                textStarts.insert(0, (text_start, textStarts[0][0]-text_start, ""))
            if len(dataSplits) == 0:
                dataStarts.insert(0, (data_start, dataStarts[0][0]-data_start, ""))
            if len(rodataSplits) == 0:
                rodataStarts.insert(0, (rodata_start, rodataStarts[0][0]-rodata_start, ""))
            #if len(bssSplits) == 0:
            #    bssStarts.insert(0, (bss_start, bssStarts[0][0]-bss_start, ""))

        i = 0
        while i < len(textStarts) - 1:
            start, size, filename = textStarts[i]
            end = start + size

            text = Text(self.bytes[start:end], filename, version, context)
            text.parent = self
            text.offset = start
            text.vRamStart = self.vRamStart

            self.textList[filename] = text
            i += 1

        i = 0
        while i < len(dataStarts) - 1:
            start, size, filename = dataStarts[i]
            end = start + size

            data = Data(self.bytes[start:end], filename, version, context)
            data.parent = self
            data.offset = start
            data.vRamStart = self.vRamStart

            self.dataList[filename] = data
            i += 1

        i = 0
        while i < len(rodataStarts) - 1:
            start, size, filename = rodataStarts[i]
            end = start + size

            rodata = Rodata(self.bytes[start:end], filename, version, context)
            rodata.parent = self
            rodata.offset = start
            rodata.vRamStart = self.vRamStart

            self.rodataList[filename] = rodata
            i += 1

        i = 0
        while i < len(bssStarts) - 1:
            start, size, filename = bssStarts[i]
            end = start + size

            bss = Bss(self.bytes[start:end], filename, version, context)
            bss.parent = self
            bss.offset = start
            bss.vRamStart = self.vRamStart

            self.bssList[filename] = bss
            i += 1
