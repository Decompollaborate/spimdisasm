#!/usr/bin/python3

from __future__ import annotations

from .MipsConstants import InstructionId

from ..Utils import *

from .MipsInstructionNormal import InstructionNormal
from ..MipsContext import Context


class InstructionNormalRsp(InstructionNormal):
    RemovedOpcodes: Dict[int, InstructionId] = {
        0b010_100: InstructionId.BEQL,
        0b010_101: InstructionId.BNEL,
        0b010_110: InstructionId.BLEZL,
        0b010_111: InstructionId.BGTZL,

        0b011_000: InstructionId.DADDI,
        0b011_001: InstructionId.DADDIU,
        0b011_010: InstructionId.LDL,
        0b011_011: InstructionId.LDR,

        0b100_010: InstructionId.LWL,
        0b100_110: InstructionId.LWR,
        0b100_111: InstructionId.LWU,

        0b101_010: InstructionId.SWL,
        0b101_100: InstructionId.SDL,
        0b101_101: InstructionId.SDR,
        0b101_110: InstructionId.SWR,

        0b110_000: InstructionId.LL,
        0b110_100: InstructionId.LLD,
        0b110_101: InstructionId.LDC1,
        0b110_110: InstructionId.LDC2,
        0b110_111: InstructionId.LD,

        0b111_000: InstructionId.SC,
        0b111_100: InstructionId.SCD,
        0b111_101: InstructionId.SDC1,
        0b111_110: InstructionId.SDC2,
        0b111_111: InstructionId.SD,
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        for opcode in InstructionNormalRsp.RemovedOpcodes:
            if opcode in self.opcodesDict:
                del self.opcodesDict[opcode]

        self.processUniqueId()
