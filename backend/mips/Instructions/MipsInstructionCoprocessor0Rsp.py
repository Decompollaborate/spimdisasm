#!/usr/bin/python3

from __future__ import annotations

from .MipsConstants import InstructionId, InstructionVectorId

from ...common.Utils import *

from .MipsInstructionCoprocessor0 import InstructionCoprocessor0


class InstructionCoprocessor0Rsp(InstructionCoprocessor0):
    Cop0Opcodes_ByFormat = {
        0b00_000: InstructionId.MFC0,
        0b00_100: InstructionId.MTC0,
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        self.isRsp = True

        # self.opcodesDict = 
        self.processUniqueId()


    def processUniqueId(self):
        super().processUniqueId()

        self.uniqueId = self.Cop0Opcodes_ByFormat.get(self.fmt, InstructionId.INVALID)


    def disassemble(self, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        # rt = self.getRegisterName(self.rt)
        # rd = self.getCop0RegisterName(self.rd)
        rt = self.getGprRspRegisterName(self.rt)
        rd = self.getCop0RspRegisterName(self.rd)

        if not self.isImplemented():
            return super().disassemble(immOverride)

        result = f"{formated_opcode} {rt},"
        result = result.ljust(14, ' ')
        result += f" {rd}"
        return result
