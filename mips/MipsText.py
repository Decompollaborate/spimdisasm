#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .MipsFile import File
from .MipsInstructions import Instruction


class Text(File):
    def __init__(self, array_of_bytes):
        super().__init__(array_of_bytes)

        self.instructions: List[Instruction] = list()
        for word in self.words:
            self.instructions.append(Instruction(word))

    @property
    def nInstr(self):
        return len(self.instructions)

    def compareToFile(self, other: File, args):
        result = super().compareToFile(other, args)

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

    def blankOutDifferences(self, other_file: File, args):
        super().blankOutDifferences(other_file, args)
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
            if args.ignore_branches:
                if instr1.isBranch() and instr2.isBranch() and instr1.sameOpcode(instr2):
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

            if not lui_found:
                if instr1.isLUI() and instr2.isLUI():
                    lui_found = True
                    lui_pos = i
                    lui_1_register = instr1.rt
                    lui_2_register = instr2.rt
            else:
                if instr1.isADDIU() and instr2.isADDIU():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
                elif instr1.isLW() and instr2.isLW():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
                elif instr1.isLWCz() and instr2.isLWCz():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
                        instr1.blankOut()
                        instr2.blankOut()
                        self.instructions[lui_pos].blankOut() # lui
                        other_file.instructions[lui_pos].blankOut() # lui
                        lui_found = False
                        was_updated = True
                elif instr1.isORI() and instr2.isORI():
                    if instr1.baseRegister == lui_1_register and instr2.baseRegister == lui_2_register:
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

    def updateWords(self):
        self.words = []
        for instr in self.instructions:
            self.words.append(instr.instr)
        self.updateBytes()
