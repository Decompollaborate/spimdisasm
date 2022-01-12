#!/usr/bin/python3

from __future__ import annotations

from .MipsConstants import InstructionId, InstructionVectorId

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionCoprocessor2(InstructionBase):
    Cop2Opcodes: Dict[int, InstructionVectorId] = {
        0x00: InstructionVectorId.VMULF,
        0x01: InstructionVectorId.VMULU,
        0x04: InstructionVectorId.VMUDL,
        0x05: InstructionVectorId.VMUDM,
        0x06: InstructionVectorId.VMUDN,
        0x07: InstructionVectorId.VMUDH,
        0x08: InstructionVectorId.VMACF,
        0x09: InstructionVectorId.VMACU,
        0x0C: InstructionVectorId.VMADL,
        0x0D: InstructionVectorId.VMADM,
        0x0E: InstructionVectorId.VMADN,
        0x0F: InstructionVectorId.VMADH,
        0x10: InstructionVectorId.VADD,
        0x14: InstructionVectorId.VADDC,
        0x1D: InstructionVectorId.VSAR,
        0x28: InstructionVectorId.VAND,
        0x29: InstructionVectorId.VNAND,
        0x2A: InstructionVectorId.VOR,
        0x2B: InstructionVectorId.VNOR,
        0x2C: InstructionVectorId.VXOR,
        0x2D: InstructionVectorId.VNXOR,

        0x20: InstructionVectorId.VLT,
        0x21: InstructionVectorId.VEQ,
        0x22: InstructionVectorId.VNE,
        0x23: InstructionVectorId.VGE,
        0x24: InstructionVectorId.VCL,
        0x25: InstructionVectorId.VCH,
        0x26: InstructionVectorId.VCR,
        0x27: InstructionVectorId.VMRG,
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        self.opcodesDict = dict(InstructionCoprocessor2.Cop2Opcodes)
        self.processUniqueId()


    def processUniqueId(self):
        super().processUniqueId()

        self.uniqueId = self.opcodesDict.get(self.function, InstructionVectorId.INVALID)

    def getOpcodeName(self) -> str:
        if self.uniqueId == InstructionVectorId.INVALID or self.uniqueId == InstructionId.INVALID:
            opcode = toHex(self.function, 2)
            return f"COP2({opcode})"
        return super().getOpcodeName()

    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        e_upper = self[25]
        e = f"{self.e}"
        vt = f"${self.vt}"
        vs = f"${self.vs}"
        vd = f"${self.vd}"

        if e_upper == 1:
            result = f"{formated_opcode} {vd},"
            result = result.ljust(14, ' ')
            result += f" {vs},"
            result = result.ljust(19, ' ')
            result += f" {vt}"
            if self.e != 0:
                result += f"[{e}]"
            return result

        return super().disassemble(context)
