#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from .... import common

from ... import instructions

from .TrackedRegisterState import TrackedRegisterState


class RegistersTracker:
    def __init__(self, other: RegistersTracker|None=None):
        self.registers: dict[int, TrackedRegisterState] = dict()

        for register in range(32):
            state = TrackedRegisterState(register)
            if other is not None:
                state.copyState(other.registers[register])
            self.registers[register] = state


    def moveRegisters(self, instr: instructions.InstructionBase) -> bool:
        if instr.uniqueId not in (instructions.InstructionId.MOVE, instructions.InstructionId.OR, instructions.InstructionId.ADDU):
            return False
        if instr.rt == 0 and instr.rs == 0:
            return False

        if instr.rt == 0:
            register = instr.rs
        elif instr.rs == 0:
            register = instr.rt
        else:
            # Check stuff like  `addu   $3, $3, $2`
            if instr.rd == instr.rs:
                register = instr.rt
                if self.registers[instr.rs].hasLuiValue:
                    register = instr.rs
            elif instr.rd == instr.rt:
                register = instr.rs
                if self.registers[instr.rt].hasLuiValue:
                    register = instr.rt
            else:
                return False

            srcState = self.registers[register]
            self.registers[instr.rd].copyState(srcState)
            return True

        srcState = self.registers[register]
        dstState = self.registers[instr.rd]
        if srcState.hasLoValue or srcState.hasLuiValue:
            dstState.copyState(srcState)
            return True
        dstState.clear()
        return False

    def overwriteRegisters(self, instr: instructions.InstructionBase, instructionOffset: int, currentVram: int|None=None) -> None:
        shouldRemove = False
        register = 0

        if self.moveRegisters(instr):
            return

        if instr.isFloatInstruction():
            if instr.uniqueId in (instructions.InstructionId.MTC1, instructions.InstructionId.DMTC1, instructions.InstructionId.CTC1):
                # IDO usually use a register as a temp when loading a constant value
                # into the float coprocessor, after that IDO never re-uses the value
                # in that register for anything else
                shouldRemove = True
                register = instr.rt
        elif instr.isRType() or (instr.isBranch() and isinstance(instr, instructions.InstructionNormal)):
            # $at is a one-use register
            at = 0
            if instr.rs == 1:
                at = instr.rs
            elif instr.rt == 1:
                at = instr.rt

            state = self.registers[at]
            if state.hasLoValue or state.hasLuiValue:
                shouldRemove = True
                register = at

        if instr.modifiesRt():
            shouldRemove = True
            register = instr.rt
            if instr.uniqueId == instructions.InstructionId.LUI:
                self.registers[instr.rt].clearLo()
                shouldRemove = False
        if instr.modifiesRd():
            shouldRemove = True
            register = instr.rd

        if shouldRemove:
            state = self.registers[register]
            if state.hasLuiValue:
                self._printDebugInfo_clearRegister(instr, register, currentVram)
            state.clearHi()
            # if instructionOffset != state.loOffset and instructionOffset != state.dereferenceOffset:
            if not state.wasSetInCurrentOffset(instructionOffset):
                state.clearLo()

    def unsetRegistersAfterFuncCall(self, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase, currentVram: int|None=None) -> None:
        if prevInstr.uniqueId != instructions.InstructionId.JAL and prevInstr.uniqueId != instructions.InstructionId.JALR:
            return

        # It happen that every register that we want to clear on a function call have the same raw values for both o32 and n32 ABIs, so no need to worry about it for now...
        registersToInvalidate = (
            1,  # $at
            2,  # $v0
            3,  # $v1
            4,  # $a0
            5,  # $a1
            6,  # $a2
            7,  # $a3
            8,  # $t0 / $a4
            9,  # $t1 / $a5
            10, # $t2 / $a6
            11, # $t3 / $a7
            12, # $t4 / $t0
            13, # $t5 / $t1
            14, # $t6 / $t2
            15, # $t7 / $t3
            24, # $t8 / $t8
            25, # $t9 / $t9
            31, # $ra
        )

        for register in registersToInvalidate:
            state = self.registers[register]
            if state.hasLuiValue:
                self._printDebugInfo_clearRegister(instr, register, currentVram)
            state.clear()

    def getAddressIfCanSetType(self, instr: instructions.InstructionBase, instrOffset: int) -> int|None:
        state = self.registers[instr.rs]
        if not state.hasLoValue:
            return None

        if not state.dereferenced or instrOffset == state.dereferenceOffset:
            return state.value

        return None

    def getJrInfo(self, instr: instructions.InstructionBase) -> tuple[int, int]|None:
        state = self.registers[instr.rs]
        if state.hasLoValue and state.dereferenced:
            return state.loOffset, state.value
        return None


    def processLui(self, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase|None, instrOffset: int) -> None:
        assert instr.uniqueId == instructions.InstructionId.LUI

        state = self.registers[instr.rt]
        state.clear()
        state.setHi(instr.immediate, instrOffset)

        if prevInstr is not None:
            # If the previous instructions is a branch likely, then nulify
            # the effects of this instruction for future analysis
            state.luiSetOnBranchLikely = prevInstr.isBranchLikely() or prevInstr.isUnconditionalBranch()

    def getLuiOffsetForConstant(self, instr: instructions.InstructionBase) -> int|None:
        stateSrc = self.registers[instr.rs]
        if stateSrc.hasLuiValue:
            return stateSrc.luiOffset
        return None

    def processConstant(self, value: int, instr: instructions.InstructionBase, offset: int) -> None:
        stateDst = self.registers[instr.rt]
        stateDst.setLo(value, offset)

    def getLuiOffsetForLo(self, instr: instructions.InstructionBase, instrOffset: int) -> tuple[int|None, bool]:
        stateSrc = self.registers[instr.rs]
        if stateSrc.hasLuiValue and not stateSrc.luiSetOnBranchLikely:
            return stateSrc.luiOffset, True

        if instr.rs == 28: # $gp
            return None, True

        if instr.modifiesRt() and instr.uniqueId not in {instructions.InstructionId.ADDIU, instructions.InstructionId.ADDI}:
            if stateSrc.hasLoValue and not stateSrc.dereferenced:
                # Simulate a dereference
                self.registers[instr.rt].dereferenceState(stateSrc, instrOffset)
        return None, False

    def processLo(self, instr: instructions.InstructionBase, value: int, offset: int) -> None:
        if not instr.modifiesRt():
            return

        stateDst = self.registers[instr.rt]
        stateDst.setLo(value, offset)
        if instr.uniqueId not in {instructions.InstructionId.ADDIU, instructions.InstructionId.ADDI}:
            stateDst.deref(offset)
        if instr.rt == instr.rs:
            stateDst.clearHi()


    def _printDebugInfo_clearRegister(self, instr: instructions.InstructionBase, register: int, currentVram: int|None=None) -> None:
        if not common.GlobalConfig.PRINT_SYMBOL_FINDER_DEBUG_INFO:
            return

        if currentVram is None:
            return

        print("Clearing register:")
        # print()
        print(f"vram: {currentVram:X}")
        print(instr)
        print(self.registers)
        print(f"deleting {register} / {instr.getRegisterName(register)}")
        print()

