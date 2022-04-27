#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Anghelo Carvajal <angheloalf95@gmail.com>
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ...common.Utils import *

from .MipsConstants import InstructionId
from .MipsInstructionBase import InstructionBase


class InstructionCoprocessor2(InstructionBase):
    def getOpcodeName(self) -> str:
        if self.uniqueId == InstructionId.INVALID:
            opcode = toHex(self.function, 2)
            return f"COP2({opcode})"
        return super().getOpcodeName()
