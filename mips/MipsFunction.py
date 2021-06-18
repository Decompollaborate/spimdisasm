#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsInstructions import Instruction, wordToInstruction

class Function:
    def __init__(self, name: str, instructions: List[Instruction]):
        self.name: str = name
        self.instructions: List[Instruction] = list(instructions)
        self.vram: int = -1
