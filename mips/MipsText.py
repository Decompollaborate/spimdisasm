#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .MipsFile import File
from .MipsInstructions import Instruction, wordToInstruction
# TODO: remove?
from .ZeldaTables import OverlayTableEntry
from .ZeldaOffsets import address_Graph_OpenDisps
from .GlobalConfig import GlobalConfig


class Text(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str):
        super().__init__(array_of_bytes, filename, version)

        self.instructions: List[Instruction] = list()
        for word in self.words:
            self.instructions.append(wordToInstruction(word))

        # TODO: make this a class?
        self.functions: List[List[Instruction]] = list()

    @property
    def nInstr(self):
        return len(self.instructions)

    def findFunctions(self):
        functionEnded = False
        func = list()
        offset = 0
        farthestBranch = 0
        for instr in self.instructions:
            func.append(instr)
            if functionEnded:
                self.functions.append(func)
                func = list()
                functionEnded = False

            if instr.isBranch():
                branch = from2Complement(instr.immediate, 16) + 1
                if branch > farthestBranch:
                    farthestBranch = branch

            if instr.getOpcodeName() == "JR" and instr.getRegisterName(instr.rs) == "$ra" and not farthestBranch > 0:
                functionEnded = True

            offset += 4
            farthestBranch -= 1
        if len(func) > 0:
            self.functions.append(func)

    def compareToFile(self, other: File):
        result = super().compareToFile(other)

        if isinstance(other, Text):
            result["text"] = {
                "diff_opcode": self.countDiffOpcodes(other),
                "same_opcode_same_args": self.countSameOpcodeButDifferentArguments(other),
            }

        return result

    def countDiffOpcodes(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            if not self.instructions[i].sameOpcode(other.instructions[i]):
                result += 1
        return result

    def countSameOpcodeButDifferentArguments(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                result += 1
        return result

    def blankOutDifferences(self, other_file: File):
        if not GlobalConfig.REMOVE_POINTERS:
            return
        super().blankOutDifferences(other_file)
        if not isinstance(other_file, Text):
            return

        was_updated = False

        lui_found = False
        lui_pos = 0
        lui_1_register = 0
        lui_2_register = 0

        for i in range(min(self.nInstr, other_file.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other_file.instructions[i]
            if not GlobalConfig.IGNORE_BRANCHES:
                if instr1.sameOpcode(instr2):
                    if instr1.isBranch() and instr2.isBranch():
                        instr1.blankOut()
                        instr2.blankOut()
                        was_updated = True
                    elif instr1.isJType():
                        instr1.blankOut()
                        instr2.blankOut()
                        was_updated = True

            #if (instr1.isADDIU() or instr1.isSB() or instr1.isSW() or instr1.isLWCz() 
            #    or instr1.isLBU() or instr1.isLH() or instr1.isLW() or instr1.isSWCz() 
            #    or instr1.isLHU() or instr1.isSH() or instr1.isLB() or instr1.isLUI()
            #    or instr1.isLDCz()):
            #    if instr1.sameOpcode(instr2) and instr1.sameBaseRegister(instr2) and instr1.rt == instr2.rt:
            #        if abs(instr1.immediate - instr2.immediate) == 0x10:
            #            instr1.blankOut()
            #            instr2.blankOut()

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
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "LW":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "LWC1" or opcode == "LWC2":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "ORI":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                        was_updated = True
            if i > lui_pos + 5:
                lui_found = False

        if was_updated:
            self.updateWords()
            other_file.updateWords()

    def removePointers(self):
        if not GlobalConfig.REMOVE_POINTERS:
            return

        was_updated = False

        if not GlobalConfig.DELETE_OPENDISPS:
            was_updated = self.deleteCallers_Graph_OpenDisps()

        was_updated = self.removeTrailingNops() or was_updated

        super().removePointers()

        lui_registers = dict()
        for i in range(len(self.instructions)):
            instr = self.instructions[i]
            opcode = instr.getOpcodeName()

            # Clean the tracked registers after X instructions have passed.
            lui_registers_aux = dict(lui_registers)
            lui_registers = dict()
            for lui_reg in lui_registers_aux:
                lui_pos, instructions_left = lui_registers_aux[lui_reg]
                instructions_left -= 1
                if instructions_left > 0:
                    lui_registers[lui_reg] = [lui_pos, instructions_left]

            if opcode == "LUI":
                lui_registers[instr.rt] = [i, GlobalConfig.TRACK_REGISTERS]
            elif opcode in ("ADDIU", "LW", "LWU", "LWC1", "LWC2", "ORI", "LH", "LHU", "LB", "LBU"):
                rs = instr.rs
                if rs in lui_registers:
                    lui_pos, _ = lui_registers[rs]
                    self.instructions[lui_pos].blankOut() # lui
                    instr.blankOut()
                    was_updated = True
            elif instr.isJType():
                instr.blankOut()
                was_updated = True

        if was_updated:
            self.updateWords()

    def deleteCallers_Graph_OpenDisps(self) -> bool:
        was_updated = False
        graph_openDisps = address_Graph_OpenDisps.get(self.version)
        if graph_openDisps is None or graph_openDisps == 0:
            return was_updated

        last_jr = 0
        found_openDisps = False
        ranges_to_delete = []
        for i in range(self.nInstr):
            instr = self.instructions[i]
            opcode = instr.getOpcodeName()
            if opcode == "JR":
                # found end of function
                if found_openDisps:
                    ranges_to_delete.append((last_jr, i))
                    was_updated = True
                found_openDisps = False
                last_jr = i
            elif opcode == "JAL":
                # check for Graph_OpenDisps
                if graph_openDisps == instr.instr_index:
                    found_openDisps = True

        # Remove all functions that call Graph_openDisps
        for begin, end in ranges_to_delete[::-1]:
            del self.instructions[begin:end]

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False
        first_nop = self.nInstr
        for i in range(self.nInstr-1, 0-1, -1):
            instr = self.instructions[i]
            if instr.getOpcodeName() != "NOP":
                break
            first_nop = i
        if first_nop != self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated

    def updateWords(self):
        self.words = []
        for instr in self.instructions:
            self.words.append(instr.instr)
        self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".text")

        with open(filepath + ".text.asm", "w") as f:
            f.write(".section .text\n\n")
            i = 0
            offset = 0
            for func in self.functions:
                funcName = f"func_{i}"
                if self.vRamStart != -1:
                    funcName = "func_" + toHex(self.vRamStart + offset, 6)[2:]
                f.write(f"glabel {funcName}\n")
                functionOffset = offset
                processed = []
                offsetsBranches = set()
                for instr in func:
                    offsetHex = toHex(offset, 5)[2:]
                    vramHex = ""
                    if self.vRamStart != -1:
                        vramHex = toHex(self.getVramOffset(offset), 6)[2:]
                    instrHex = toHex(instr.instr, 8)[2:]

                    comment = f"/* {offsetHex} {vramHex} {instrHex} */"

                    line = str(instr)
                    if instr.isBranch():
                        #line += " HERE"
                        line = line[:-6]
                        addr = from2Complement(instr.immediate, 16)
                        branch = offset + 1*4 + addr*4
                        offsetsBranches.add(branch)
                        if self.vRamStart != -1:
                            line += ".L" + toHex(self.vRamStart + branch, 5)[2:]
                        else:
                            line += ".L" + toHex(branch, 5)[2:]

                    data = {"comment": comment, "instr": instr, "line": line}
                    processed.append(data)

                    offset += 4

                auxOffset = functionOffset
                for data in processed:
                    line = data["comment"] + "  " + data["line"]
                    if auxOffset in offsetsBranches:
                        if self.vRamStart != -1:
                            line = ".L" + toHex(self.vRamStart + auxOffset, 5)[2:] + ":\n" + line
                        else:
                            line = ".L" + toHex(auxOffset, 5)[2:] + ":\n" + line
                    f.write(line + "\n")

                    auxOffset += 4

                f.write("\n")
                i += 1
