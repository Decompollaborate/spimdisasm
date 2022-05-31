#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses

from . import instructions


@dataclasses.dataclass
class TrackedRegisterState:
    registerNum: int

    hasLuiValue: bool = False
    luiOffset: int = 0
    "The offset of last lui which set a value to this register"
    luiSetOnBranchLikely: bool = False

    hasLoValue: bool = False
    loOffset: int = 0
    dereferenced: bool = False
    dereferenceOffset: int = 0

    value: int = 0

    def clear(self) -> None:
        self.hasLuiValue = False
        self.luiOffset = 0
        self.luiSetOnBranchLikely = False
        self.hasLoValue = False
        self.loOffset = 0
        self.dereferenced = False
        self.dereferenceOffset = 0
        self.value = 0

    def clearHi(self) -> None:
        self.hasLuiValue = False
        self.luiOffset = 0
        self.luiSetOnBranchLikely = False

    def clearLo(self) -> None:
        self.hasLoValue = False
        self.loOffset = 0
        self.dereferenced = False
        self.dereferenceOffset = 0
        self.value = 0

    def copyState(self, other: TrackedRegisterState) -> None:
        self.hasLuiValue = other.hasLuiValue
        self.luiOffset = other.luiOffset
        self.luiSetOnBranchLikely = other.luiSetOnBranchLikely

        self.hasLoValue = other.hasLoValue
        self.loOffset = other.loOffset
        self.dereferenced = other.dereferenced
        self.dereferenceOffset = other.dereferenceOffset

        self.value = other.value

    def makeACopy(self) -> TrackedRegisterState:
        other = TrackedRegisterState(self.registerNum)
        other.copyState(self)
        return other

    def processLui(self, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase|None, instructionOffset: int) -> None:
        self.clear()

        self.hasLuiValue = True
        self.luiOffset = instructionOffset
        if prevInstr is not None:
            # If the previous instructions is a branch likely, then nulify
            # the effects of this instruction for future analysis
            self.luiSetOnBranchLikely = prevInstr.isBranchLikely() or prevInstr.isUnconditionalBranch()
        self.value = instr.immediate << 16

    def setLo(self, value: int, offset: int) -> None:
        self.value = value
        self.loOffset = offset
        self.hasLoValue = True
        self.dereferenced = False
        self.dereferenceOffset = 0

    def deref(self, offset: int) -> None:
        self.dereferenced = True
        self.dereferenceOffset = offset

    def dereferenceState(self, other: TrackedRegisterState, offset: int) -> None:
        assert other.hasLoValue
        assert not other.dereferenced

        self.copyState(other)
        self.deref(offset)

