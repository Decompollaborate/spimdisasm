#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .Instructions import InstructionBase
from .MipsContext import Context, ContextVariable

class Function:
    def __init__(self, name: str, instructions: List[InstructionBase], context: Context, inFileOffset: int, vram: int = -1):
        self.name: str = name
        self.instructions: List[InstructionBase] = list(instructions)
        self.context: Context = context
        self.inFileOffset: int = inFileOffset
        self.vram: int = vram
        self.index: int = -1
        self.pointersRemoved: bool = False

        self.localLabels: Dict[int, str] = dict()
        # TODO: this needs a better name
        self.pointersPerInstruction: Dict[int, int] = dict()
        self.branchInstructions: List[int] = list()

        self.pointersOffsets: List[int] = list()

    @property
    def nInstr(self) -> int:
        return len(self.instructions)

    def analyze(self):
        trackedRegisters: Dict[int, int] = dict()
        registersValues: Dict[int, int] = dict()

        instructionOffset = 0
        for instr in self.instructions:
            isLui = False
            opcode = instr.getOpcodeName()

            if instr.isBranch():
                diff = from2Complement(instr.immediate, 16)
                branch = instructionOffset + diff*4 + 1*4
                if self.vram >= 0:
                    auxLabel = self.context.getGenericLabel(self.vram + branch)
                    if auxLabel is not None:
                        label = auxLabel
                    else:
                        label = ".L" + toHex(self.vram + branch, 5)[2:]
                else:
                    label = ".L" + toHex(self.inFileOffset + branch, 5)[2:]

                self.localLabels[self.inFileOffset + branch] = label
                if self.vram >= 0:
                    if self.vram + branch not in self.context.labels:
                        self.context.labels[self.vram + branch] = label
                self.branchInstructions.append(instructionOffset)

            elif instr.isJType():
                target = 0x80000000 | instr.instr_index << 2
                if instr.getOpcodeName() == "J":
                    if target not in self.context.fakeFunctions:
                        label = "fakefunc_" + toHex(target, 8)[2:]
                        self.context.fakeFunctions[target] = label
                else:
                    if target not in self.context.funcAddresses:
                        label = "func_" + toHex(target, 8)[2:]
                        self.context.funcAddresses[target] = label
                self.pointersPerInstruction[instructionOffset] = target

            # symbol finder
            elif instr.isIType():
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
                        if address not in self.context.symbols:
                            self.context.symbols[address] = ContextVariable(address, "D_" + toHex(address, 8)[2:])
                        self.pointersPerInstruction[instructionOffset] = address
                        self.pointersPerInstruction[trackedRegisters[rs]*4] = address
                        registersValues[instr.rt] = address

            elif opcode == "JR":
                rs = instr.rs
                if instr.getRegisterName(rs) != "$ra":
                    if rs in registersValues:
                        address = registersValues[rs]
                        if address not in self.context.jumpTables:
                            self.context.jumpTables[address] = "jtbl_" + toHex(address, 8)[2:]

            if not instr.isFloatInstruction():
                if not isLui and instr.modifiesRt():
                    rt = instr.rt
                    if rt in trackedRegisters:
                        del trackedRegisters[rt]

                if instr.modifiesRd():
                    if opcode not in ("ADDU",):
                        rd = instr.rd
                        if rd in trackedRegisters:
                            del trackedRegisters[rd]

            instructionOffset += 4

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

        for i in range(min(self.nInstr, other_func.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other_func.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                instr1.blankOut()
                instr2.blankOut()
                was_updated = True

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False

        for instructionOffset in self.pointersPerInstruction:
            self.instructions[instructionOffset//4].blankOut()
        was_updated = len(self.pointersPerInstruction) > 0 or was_updated

        for fileOffset in self.pointersOffsets:
            index = (fileOffset - self.inFileOffset)//4
            if index < 0:
                continue
            if index >= self.nInstr:
                continue
            self.instructions[index].blankOut()

        if GlobalConfig.IGNORE_BRANCHES:
            for instructionOffset in self.branchInstructions:
                self.instructions[instructionOffset//4].blankOut()
            was_updated = len(self.branchInstructions) > 0 or was_updated

        self.pointersRemoved = True

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False
        first_nop = self.nInstr

        for i in range(self.nInstr-1, 0-1, -1):
            instr = self.instructions[i]
            opcodeName = instr.getOpcodeName()
            if opcodeName != "NOP":
                if opcodeName == "JR" and instr.getRegisterName(instr.rs) == "$ra":
                    first_nop += 1
                break
            first_nop = i

        if first_nop < self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated


    def disassemble(self) -> str:
        output = ""

        output += f"glabel {self.name}"
        #if self.index >= 0:
        #    output += f" # {self.index}"
        output += "\n"

        wasLastInstABranch = False

        instructionOffset = 0
        auxOffset = self.inFileOffset
        for instr in self.instructions:
            offsetHex = toHex(auxOffset, 6)[2:]
            vramHex = ""
            if self.vram >= 0:
                vramHex = toHex(self.vram + instructionOffset, 8)[2:]
            instrHex = toHex(instr.instr, 8)[2:]

            immOverride = None
            if instr.isBranch():
                if not GlobalConfig.IGNORE_BRANCHES:
                    diff = from2Complement(instr.immediate, 16)
                    branch = instructionOffset + diff*4 + 1*4
                    label = self.context.getGenericLabel(self.vram + branch)
                    if self.vram >= 0 and label is not None:
                        immOverride = label
                    elif self.inFileOffset + branch in self.localLabels:
                        immOverride = self.localLabels[self.inFileOffset + branch]

            elif instr.isIType():
                if not self.pointersRemoved and instructionOffset in self.pointersPerInstruction:
                    address = self.pointersPerInstruction[instructionOffset]

                    symbol = self.context.getGenericSymbol(address)
                    if symbol is not None:
                        if instr.getOpcodeName() == "LUI":
                            immOverride = f"%hi({symbol})"
                        else:
                            immOverride= f"%lo({symbol})"

            if wasLastInstABranch:
                instr.ljustWidthOpcode -= 1

            line = instr.disassemble(self.context, immOverride)

            #comment = " "
            comment = ""
            if GlobalConfig.ASM_COMMENT:
                comment = f"/* {offsetHex} {vramHex} {instrHex} */  "
            if wasLastInstABranch:
                comment += " "
            line = comment + line

            label = ""
            if not GlobalConfig.IGNORE_BRANCHES:
                labelAux = self.context.getGenericLabel(self.vram + instructionOffset)
                if self.vram >= 0 and labelAux is not None:
                    if self.vram + instructionOffset in self.context.jumpTablesLabels:
                        label = "glabel " + labelAux + "\n"
                    else:
                        label = labelAux + ":\n"
                elif auxOffset in self.localLabels:
                    label = self.localLabels[auxOffset] + ":\n"
                elif self.vram + instructionOffset in self.context.fakeFunctions:
                    label = self.context.fakeFunctions[self.vram + instructionOffset] + ":\n"

            output += label + line + "\n"

            wasLastInstABranch = instr.isBranch() or instr.isJType()

            instructionOffset += 4
            auxOffset += 4

        return output
