#!/usr/bin/python3

from __future__ import annotations

from .MipsInstructionBase import InstructionBase
from .MipsInstructionNormal import InstructionNormal
from .MipsInstructionSpecial import InstructionSpecial
from .MipsInstructionRegimm import InstructionRegimm
from .MipsInstructionCoprocessor0 import InstructionCoprocessor0
from .MipsInstructionCoprocessor1 import InstructionCoprocessor1

from .MipsInstructions import wordToInstruction
