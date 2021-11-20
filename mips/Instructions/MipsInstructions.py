#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsConstants import InstructionId
from .MipsInstructionBase import InstructionBase
from .MipsInstructionNormal import InstructionNormal
from .MipsInstructionSpecial import InstructionSpecial
from .MipsInstructionRegimm import InstructionRegimm
from .MipsInstructionCoprocessor0 import InstructionCoprocessor0
from .MipsInstructionCoprocessor1 import InstructionCoprocessor1
from .MipsInstructionCoprocessor2 import InstructionCoprocessor2


def wordToInstruction(word: int) -> InstructionBase:
    if ((word >> 26) & 0x3F) == 0x00:
        return InstructionSpecial(word)
    if ((word >> 26) & 0x3F) == 0x01:
        return InstructionRegimm(word)
    if ((word >> 26) & 0x3F) == 0x10:
        return InstructionCoprocessor0(word)
    if ((word >> 26) & 0x3F) == 0x11:
        return InstructionCoprocessor1(word)
    if ((word >> 26) & 0x3F) == 0x12:
        return InstructionCoprocessor2(word)
    return InstructionNormal(word)
