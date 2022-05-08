#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from . import InstructionId, InstructionBase


class InstructionCoprocessor2(InstructionBase):
    def getOpcodeName(self) -> str:
        if self.uniqueId == InstructionId.INVALID:
            return f"COP2(0x{self.function:02X})"
        return super().getOpcodeName()
