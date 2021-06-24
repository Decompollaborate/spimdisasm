#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .Instructions import InstructionBase

class Function:
    def __init__(self, name: str, instructions: List[InstructionBase], inFileOffset: int, vram: int = -1):
        self.name: str = name
        self.instructions: List[InstructionBase] = list(instructions)
        self.inFileOffset: int = inFileOffset
        self.vram: int = vram
        self.index: int = -1

        self.labels: Dict[int, str] = dict()
        self.referencedFunctions: Dict[int, str] = dict()
        self.referencedFakeFunctions: Dict[int, str] = dict()
        self.symbols: Dict[int, str] = dict()

        # TODO: this needs a better name
        self.pointersPerInstruction: Dict[int, int] = dict()

        trackedRegisters: Dict[int, int] = dict()

        instructionOffset = 0
        for instr in self.instructions:
            isLui = False

            if instr.isBranch():
                addr = from2Complement(instr.immediate, 16)
                branch = instructionOffset + addr*4 + 1*4
                if self.vram >= 0:
                    label = ".L" + toHex(self.vram + branch, 5)[2:]
                else:
                    label = ".L" + toHex(self.inFileOffset + branch, 5)[2:]

                self.labels[self.inFileOffset + branch] = label
                if self.vram >= 0:
                    self.labels[self.vram + branch] = label

            elif instr.isJType():
                target = 0x80000000 | instr.instr_index << 2
                if instr.getOpcodeName() == "J":
                    label = "fakefunc_" + toHex(target, 8)[2:]
                    self.referencedFakeFunctions[target] = label
                else:
                    label = "func_" + toHex(target, 8)[2:]
                    self.referencedFunctions[target] = label

            # symbol finder
            elif instr.isIType():
                opcode = instr.getOpcodeName()
                isLui = opcode == "LUI"
                if isLui:
                    trackedRegisters[instr.rt] = instructionOffset//4
                elif instr.isIType() and opcode not in ("ANDI", "ORI", "XORI"):
                    rs = instr.rs
                    if rs in trackedRegisters:
                        luiInstr = self.instructions[trackedRegisters[rs]]
                        upperHalf = luiInstr.immediate << 16
                        lowerHalf = from2Complement(instr.immediate, 16)
                        address = upperHalf + lowerHalf
                        self.symbols[address] = "D_" + toHex(address, 8)[2:]
                        self.pointersPerInstruction[instructionOffset] = address
                        self.pointersPerInstruction[trackedRegisters[rs]*4] = address

            if not instr.isFloatInstruction():
                if not isLui and instr.modifiesRt():
                    rt = instr.rt
                    if rt in trackedRegisters:
                        del trackedRegisters[rt]

                if instr.modifiesRd():
                    rd = instr.rd
                    if rd in trackedRegisters:
                        del trackedRegisters[rd]

            instructionOffset += 4

    @property
    def nInstr(self) -> int:
        return len(self.instructions)

    def countDiffOpcodes(self, other: Function) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            if not self.instructions[i].sameOpcode(other.instructions[i]):
                result += 1
        return result

    def countSameOpcodeButDifferentArguments(self, other: Function) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                result += 1
        return result

    def blankOutDifferences(self, other_func: Function) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False

        lui_found = False
        lui_pos = 0
        lui_1_register = 0
        lui_2_register = 0

        for i in range(min(self.nInstr, other_func.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other_func.instructions[i]
            if GlobalConfig.IGNORE_BRANCHES:
                if instr1.sameOpcode(instr2):
                    if instr1.isBranch() and instr2.isBranch():
                        instr1.blankOut()
                        instr2.blankOut()
                        was_updated = True
                    elif instr1.isJType():
                        instr1.blankOut()
                        instr2.blankOut()
                        was_updated = True

            opcode = instr1.getOpcodeName()

            if instr1.sameOpcode(instr2):
                if not lui_found:
                    if opcode == "LUI":
                        lui_found = True
                        lui_pos = i
                        lui_1_register = instr1.rt
                        lui_2_register = instr2.rt
                else:
                    if opcode == "ADDIU":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_func.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "LW":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_func.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "LWC1" or opcode == "LWC2":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_func.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "ORI":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_func.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                        was_updated = True
            if i > lui_pos + GlobalConfig.TRACK_REGISTERS:
                lui_found = False

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False

        lui_registers = dict()
        for i in range(len(self.instructions)):
            instr = self.instructions[i]
            opcode = instr.getOpcodeName()

            # Clean the tracked registers after X instructions have passed.
            lui_registers_aux = dict()
            for lui_reg in lui_registers:
                lui_pos, instructions_left = lui_registers[lui_reg]
                instructions_left -= 1
                if instructions_left > 0:
                    lui_registers_aux[lui_reg] = [lui_pos, instructions_left]
            lui_registers = lui_registers_aux

            if opcode == "LUI":
                lui_registers[instr.rt] = [i, GlobalConfig.TRACK_REGISTERS]
            elif opcode in ("ADDIU", "LW", "LWU", "LWC1", "LWC2", "ORI", "LH", "LHU", "LB", "LBU", "SW", "SWL", "SWR", "SWC1", "SWC2", "SB", "SH", "SDR"):
                rs = instr.rs
                if rs in lui_registers:
                    lui_pos, _ = lui_registers[rs]
                    self.instructions[lui_pos].blankOut() # lui
                    instr.blankOut()
                    was_updated = True
            elif instr.isJType():
                instr.blankOut()
                was_updated = True

        return was_updated


    def disassemble(self) -> str:
        output = ""

        output += f"glabel {self.name}"
        if self.index >= 0:
            output += f" # {self.index}"
        output += "\n"

        instructionOffset = 0
        auxOffset = self.inFileOffset
        for instr in self.instructions:
            offsetHex = toHex(auxOffset, 5)[2:]
            vramHex = ""
            if self.vram >= 0:
                vramHex = toHex(self.vram + instructionOffset, 8)[2:]
            instrHex = toHex(instr.instr, 8)[2:]

            comment = f" /* {offsetHex} {vramHex} {instrHex} */"

            line = instr.disassemble()
            if instr.isBranch():
                addr = from2Complement(instr.immediate, 16)
                branch = auxOffset + addr*4 + 1*4
                if branch in self.labels:
                    # TODO: this shouldn't be hardcoded
                    line = line[:-6]
                    line += self.labels[branch]
            elif instr.isIType():
                # TODO: don't hardcode
                if instructionOffset in self.pointersPerInstruction:
                    address = self.pointersPerInstruction[instructionOffset]
                    symbol = self.symbols[address]
                    if instr.getOpcodeName() == "LUI":
                        line = line[:-6]
                        line += f"%hi({symbol})"
                    else:
                        if "(" in line:
                            *line, aux = line.split("(")
                            line = "(".join(line)
                            line = line[:-6]
                            line += f"%lo({symbol})"
                            line += "(" + aux
                        else:
                            line = line[:-6]
                            line += f"%lo({symbol})"

            line = comment + "  " + line
            if auxOffset in self.labels:
                line = self.labels[auxOffset] + ":\n" + line
            elif self.vram + instructionOffset in self.referencedFakeFunctions:
                line = self.referencedFakeFunctions[self.vram + instructionOffset] + ":\n" + line
            output += line + "\n"

            instructionOffset += 4
            auxOffset += 4

        return output
