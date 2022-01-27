#!/usr/bin/python3

from __future__ import annotations

from .MipsConstants import InstructionId, InstructionVectorId, InstructionsNotEmitedByIDO
from .MipsInstructionBase import InstructionBase

from .MipsInstructionNormal import InstructionNormal
from .MipsInstructionSpecial import InstructionSpecial
from .MipsInstructionRegimm import InstructionRegimm
from .MipsInstructionCoprocessor0 import InstructionCoprocessor0
from .MipsInstructionCoprocessor1 import InstructionCoprocessor1
from .MipsInstructionCoprocessor2 import InstructionCoprocessor2

from .MipsInstructionNormalRsp import InstructionNormalRsp
from .MipsInstructionSpecialRsp import InstructionSpecialRsp
from .MipsInstructionRegimmRsp import InstructionRegimmRsp

from .MipsInstructions import wordToInstruction, wordToInstructionRsp
