#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section


class Data(Section):
    def analyze(self):
        if self.vRamStart > -1:
            if self.parent is not None:
                initVarsAddress = self.parent.initVarsAddress
                if initVarsAddress > -1:
                    initVarsOffset = (initVarsAddress-self.vRamStart-self.offset)//4

                    initVram = self.words[initVarsOffset + 4]
                    if initVram != 0:
                        self.context.addFunction(self.filename, initVram, f"{self.filename}_Init")

                    destroyVram = self.words[initVarsOffset + 5]
                    if destroyVram != 0:
                        self.context.addFunction(self.filename, destroyVram, f"{self.filename}_Destroy")

                    updateVram = self.words[initVarsOffset + 6]
                    if updateVram != 0:
                        self.context.addFunction(self.filename, updateVram, f"{self.filename}_Update")

                    drawVram = self.words[initVarsOffset + 7]
                    if drawVram != 0:
                        self.context.addFunction(self.filename, drawVram, f"{self.filename}_Draw")

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        for i in range(self.sizew):
            top_byte = (self.words[i] >> 24) & 0xFF
            if top_byte == 0x80:
                self.words[i] = top_byte << 24
                was_updated = True
            if (top_byte & 0xF0) == 0x00 and (top_byte & 0x0F) != 0x00:
                self.words[i] = top_byte << 24
                was_updated = True

        return was_updated

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".data")

        if self.size == 0:
            return

        with open(filepath + ".data.s", "w") as f:
            f.write(".include \"macro.inc\"\n")
            f.write("\n")
            f.write("# assembler directives\n")
            f.write(".set noat      # allow manual use of $at\n")
            f.write(".set noreorder # don't insert nops after branches\n")
            f.write(".set gp=64     # allow use of 64-bit general purpose registers\n")
            f.write("\n")
            f.write(".section .data\n")
            f.write("\n")
            f.write(".balign 16\n")

            initVarsAddress = -1
            if self.parent is not None:
                initVarsAddress = self.parent.initVarsAddress
            offset = 0
            inFileOffset = self.offset
            i = 0
            while i < self.sizew:
                w = self.words[i]
                offsetHex = toHex(inFileOffset, 6)[2:]
                vramHex = ""
                label = ""
                if self.vRamStart != -1:
                    currentVram = self.getVramOffset(offset)
                    vramHex = toHex(currentVram, 8)[2:]

                    if self.context is not None:
                        auxLabel = self.context.getGenericLabel(currentVram) or self.context.getGenericSymbol(currentVram)
                        if auxLabel is not None:
                            label = "\nglabel " + auxLabel + "\n"

                    if currentVram == initVarsAddress:
                        f.write(f"\nglabel {self.filename}_InitVars\n")
                        actorId = toHex((w >> 16) & 0xFFFF, 4)
                        category = toHex((w >> 8) & 0xFF, 2)
                        flags = toHex((self.words[i+1]), 8)
                        objectId = toHex((self.words[i+2] >> 16) & 0xFFFF, 4)
                        instanceSize = toHex(self.words[i+3], 8)
                        f.write(f"/* %05X %08X {actorId[2:].zfill(8)} */  .half  {actorId}\n" % (offset + 0x0, currentVram + 0x0))
                        f.write(f"/* %05X %08X {category[2:].zfill(8)} */  .byte  {category}\n" % (offset + 0x2, currentVram + 0x2))
                        f.write(f"/* %05X %08X {flags[2:].zfill(8)} */  .word  {flags}\n" % (offset + 0x4, currentVram + 0x4))
                        f.write(f"/* %05X %08X {objectId[2:].zfill(8)} */  .half  {objectId}\n" % (offset + 0x8, currentVram + 0x8))
                        f.write(f"/* %05X %08X {instanceSize[2:].zfill(8)} */  .word  {instanceSize}\n" % (offset + 0xC, currentVram + 0xC))
                        init = f"{self.filename}_Init"
                        if self.words[i+4] == 0:
                            init = toHex(0, 8)
                        destroy = f"{self.filename}_Destroy"
                        if self.words[i+5] == 0:
                            destroy = toHex(0, 8)
                        update = f"{self.filename}_Update"
                        if self.words[i+6] == 0:
                            update = toHex(0, 8)
                        draw = f"{self.filename}_Draw"
                        if self.words[i+7] == 0:
                            draw = toHex(0, 8)
                        f.write(f"/* %05X %08X {toHex(self.words[i+4], 8)[2:]} */  .word  {init}\n" % (offset + 0x10, currentVram + 0x10))
                        f.write(f"/* %05X %08X {toHex(self.words[i+5], 8)[2:]} */  .word  {destroy}\n" % (offset + 0x14, currentVram + 0x14))
                        f.write(f"/* %05X %08X {toHex(self.words[i+6], 8)[2:]} */  .word  {update}\n" % (offset + 0x18, currentVram + 0x18))
                        f.write(f"/* %05X %08X {toHex(self.words[i+7], 8)[2:]} */  .word  {draw}\n" % (offset + 0x1C, currentVram + 0x1C))
                        f.write("\n")

                        i += 8
                        offset += 0x20
                        continue

                dataHex = toHex(w, 8)[2:]
                value = toHex(w, 8)
                if self.context is not None:
                    symbol = self.context.getAnySymbol(w)
                    if symbol is not None:
                        value = symbol

                #comment = " "
                comment = ""
                if GlobalConfig.ASM_COMMENT:
                    #comment = f"/* {offsetHex} {vramHex} {dataHex} */"
                    comment = f"/* {offsetHex} {vramHex} */"

                line = f"{label}{comment} .word {value}"
                f.write(line + "\n")
                i += 1
                offset += 4
                inFileOffset += 4
