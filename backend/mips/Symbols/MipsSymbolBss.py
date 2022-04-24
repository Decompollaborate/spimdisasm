#!/usr/bin/env python3

from __future__ import annotations

from ...common.Utils import *
from ...common.GlobalConfig import GlobalConfig
from ...common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ...common.FileSectionType import FileSectionType

from .MipsSymbolBase import SymbolBase


class SymbolBss(SymbolBase):
    def __init__(self, context: Context, name: str, inFileOffset: int, vram: int|None, size: int):
        super().__init__(context, name, inFileOffset, vram)

        self.size: int = size

        self.sectionType = FileSectionType.Bss


    def disassembleAsBss(self) -> str:
        output = f""

        localOffset = 0
        inFileOffset = self.inFileOffset

        label = self.getSymbolLabelAtOffset(inFileOffset, "")

        # if we have vram available, try to get the symbol name from the Context
        if self.vram is not None:
            currentVram = self.getVramOffset(localOffset)

            label = self.getSymbolLabelAtVram(currentVram, label)

            contVariable = self.context.getSymbol(currentVram, False)
            if contVariable is not None:
                contVariable.isDefined = True

        comment = self.generateAsmLineComment(localOffset)
        line = f"{label}{comment}  .space  0x{self.size:02X}"
        output += line + "\n"
        localOffset += 4
        inFileOffset += 4

        return output

    def disassemble(self) -> str:
        return self.disassembleAsBss()
