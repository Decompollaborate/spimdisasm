#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionCoprocessor2(InstructionBase):
    def isImplemented(self) -> bool:
        return False


    def getOpcodeName(self) -> str:
        opcode = toHex(self.opcode, 2)
        return f"COP2({opcode})"
