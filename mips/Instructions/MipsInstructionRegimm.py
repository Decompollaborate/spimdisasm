#!/usr/bin/python3

from __future__ import annotations

from mips.Instructions.MipsConstants import InstructionId

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionRegimm(InstructionBase):
    RegimmOpcodes: Dict[int, InstructionId] = {
        0b00_000: InstructionId.BLTZ,
        0b00_001: InstructionId.BGEZ,
        0b00_010: InstructionId.BLTZL,
        0b00_011: InstructionId.BGEZL,

        0b01_000: InstructionId.TGEI,
        0b01_001: InstructionId.TGEIU,
        0b01_010: InstructionId.TLTI,
        0b01_011: InstructionId.TLTIU,

        0b10_000: InstructionId.BLTZAL,
        0b10_001: InstructionId.BGEZAL,
        0b10_010: InstructionId.BLTZALL,
        0b10_011: InstructionId.BGEZALL,

        0b01_100: InstructionId.TEQI,
        0b01_110: InstructionId.TNEI,
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        self.uniqueId = InstructionRegimm.RegimmOpcodes.get(self.rt, InstructionId.INVALID)


    def isBranch(self) -> bool:
        if self.uniqueId in (InstructionId.BLTZ, InstructionId.BGEZ, InstructionId.BLTZL, InstructionId.BGEZL):
            return True
        if self.uniqueId in (InstructionId.BLTZAL, InstructionId.BGEZAL, InstructionId.BLTZALL, InstructionId.BGEZALL):
            return True
        return False
    def isTrap(self) -> bool:
        if self.uniqueId in (InstructionId.TGEI, InstructionId.TGEIU, InstructionId.TLTI, InstructionId.TLTIU,
                             InstructionId.TEQI, InstructionId.TNEI):
            return True
        return False


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
        if self.uniqueId == InstructionId.INVALID:
            opcode = toHex(self.rt, 2)
            return f"Regimm({opcode})"
        return super().getOpcodeName()


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
