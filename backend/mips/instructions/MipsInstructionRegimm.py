#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from . import InstructionId, InstructionBase
from .MipsInstructionConfig import InstructionConfig


class InstructionRegimm(InstructionBase):
    RegimmOpcodes: dict[int, InstructionId] = {
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

        self.opcodesDict = dict(InstructionRegimm.RegimmOpcodes)
        self.processUniqueId()


    def processUniqueId(self):
        super().processUniqueId()

        self.uniqueId = self.opcodesDict.get(self.rt, InstructionId.INVALID)


    def isBranch(self) -> bool:
        if self.uniqueId in (InstructionId.BLTZ, InstructionId.BGEZ, InstructionId.BLTZL, InstructionId.BGEZL):
            return True
        if self.uniqueId in (InstructionId.BLTZAL, InstructionId.BGEZAL, InstructionId.BLTZALL, InstructionId.BGEZALL):
            return True
        return False
    def isBranchLikely(self) -> bool:
        if self.uniqueId in (InstructionId.BLTZL, InstructionId.BGEZL, InstructionId.BLTZALL, InstructionId.BGEZALL):
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
        if not self.isImplemented():
            return f"Regimm(0x{self.rt:02X})"
        return super().getOpcodeName()


    # OP  rs, IMM
    def disassembleInstruction(self, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName().lower().ljust(InstructionConfig.OPCODE_LJUST + self.extraLjustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        immediate = hex(self.immediate)
        if immOverride is not None:
            immediate = immOverride

        result = f"{opcode} {rs},"
        result = result.ljust(14, ' ')
        return f"{result} {immediate}"
