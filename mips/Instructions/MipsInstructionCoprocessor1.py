#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase


class InstructionCoprocessor1(InstructionBase):
    def isImplemented(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BC1T", "BC1TL", "BC1F", "BC1FL"):
            return True
        return False

    def isFloatInstruction(self):
        return True


    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BC1T", "BC1TL", "BC1F", "BC1FL"):
            return True
        return False


    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.opcode != other.opcode:
            return False

        if self.rs == 0b01_000 and self.rs == other.rs:
            tf = (self.instr >> 16) & 0x01
            nd = (self.instr >> 17) & 0x01
            other_tf = (other.instr >> 16) & 0x01
            other_nd = (other.instr >> 17) & 0x01
            if tf == other_tf and nd == other_nd:
                return True

        # TODO: implement the rest
        return self.function == other.function


    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        # TODO
        return super().modifiesRt()
    def modifiesRd(self) -> bool:
        # TODO
        return super().modifiesRd()


    def blankOut(self):
        # TODO
        super().blankOut()


    def getOpcodeName(self) -> str:
        if self.rs == 0b01_000:
            tf = (self.instr >> 16) & 0x01
            nd = (self.instr >> 17) & 0x01
            opcodeName = "BC1"
            if tf: # Branch on FP True
                opcodeName += "T"
            else: # Branch on FP False
                opcodeName += "F"
            if nd: # Likely
                opcodeName += "L"
            return opcodeName
        # TODO: implement the rest
        function = toHex(self.function, 2)
        return f"COP1({function})"

    def disassemble(self) -> str:
        if self.isBranch():
            opcode = self.getOpcodeName().lower().ljust(7, ' ')
            immediate = toHex(self.immediate, 4)

            result = opcode
            return f"{result} {immediate}"

        opcode = "COP1".lower().ljust(7, ' ')
        instr_index = toHex(self.instr_index, 7)
        return f"{opcode} {instr_index}"
