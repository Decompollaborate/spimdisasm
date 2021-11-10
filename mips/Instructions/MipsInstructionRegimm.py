#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionRegimm(InstructionBase):
    RegimmOpcodes = {
        0b00_000: "BLTZ",
        0b00_001: "BGEZ",
        0b00_010: "BLTZL",
        0b00_011: "BGEZL",

        0b01_000: "TGEI",
        0b01_001: "TGEIU",
        0b01_010: "TLTI",
        0b01_011: "TLTIU",

        0b10_000: "BLTZAL",
        0b10_001: "BGEZAL",
        0b10_010: "BLTZALL",
        0b10_011: "BGEZALL",

        0b01_100: "TEQI",
        0b01_110: "TNEI",
    }

    def isImplemented(self) -> bool:
        return self.rt in InstructionRegimm.RegimmOpcodes


    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BLTZ", "BGEZ", "BLTZL", "BGEZL"):
            return True
        if opcode in ("BLTZAL", "BGEZAL", "BLTZALL", "BGEZALL"):
            return True
        return False
    def isTrap(self) -> bool:
        opcode = self.getOpcodeName()
        return opcode in ("TGEI", "TGEIU", "TLTI", "TLTIU", "TEQI", "TNEI")


    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.opcode != other.opcode:
            return False

        return self.rt == other.rt


    def modifiesRt(self) -> bool:
        return False
    def modifiesRd(self) -> bool:
        return False


    def blankOut(self):
        self.rs = 0
        self.rd = 0
        self.sa = 0
        self.function = 0


    def getOpcodeName(self) -> str:
        opcode = toHex(self.rt, 2)
        return InstructionRegimm.RegimmOpcodes.get(self.rt, f"REGIMM({opcode})")


    # OP  rs, IMM
    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName().lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        #immediate = toHex(self.immediate, 4)
        immediate = hex(self.immediate)
        if immOverride is not None:
            immediate = immOverride

        result = f"{opcode} {rs},"
        result = result.ljust(14, ' ')
        return f"{result} {immediate}"
