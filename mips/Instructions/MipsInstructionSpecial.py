#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionSpecial(InstructionBase):
    SpecialOpcodes = {
        0b000_000: "SLL", # Shift word Left Logical
        0b000_001: "MOVCI", # TODO
        0b000_010: "SRL", # Shift word Right Logical
        0b000_011: "SRA", # Shift word Right Arithmetic
        0b000_100: "SLLV", # Shift word Left Logical Variable
        # 0b000_101: "",
        0b000_110: "SRLV", # Shift word Right Logical Variable
        0b000_111: "SRAV", # Shift word Right Arithmetic Variable

        0b001_000: "JR", # Jump Register
        0b001_001: "JALR", # Jump And Link Register
        0b001_010: "MOVZ", # MOVe conditional on Zero
        0b001_011: "MOVN", # MOVe conditional on Not zero
        0b001_100: "SYSCALL", # SYStem CALL
        0b001_101: "BREAK", # Break
        # 0b001_110: "",
        0b001_111: "SYNC", # Sync

        0b010_000: "MFHI", # Move From HI register
        0b010_001: "MTHI", # Move To HI register
        0b010_010: "MFLO", # Move From LO register
        0b010_011: "MTLO", # Move To LO register
        0b010_100: "DSLLV", # Doubleword Shift Left Logical Variable
        # 0b010_101: "",
        0b010_110: "DSRLV", # Doubleword Shift Right Logical Variable
        0b010_111: "DSRAV", # Doubleword Shift Right Arithmetic Variable

        0b011_000: "MULT", # MULTtiply word
        0b011_001: "MULTU", # MULTtiply Unsigned word
        0b011_010: "DIV", # DIVide word
        0b011_011: "DIVU", # DIVide Unsigned word
        0b011_100: "DMULT", # Doubleword MULTiply
        0b011_101: "DMULTU", # Doubleword MULTiply Unsigned
        0b011_110: "DDIV", # Doubleword DIVide
        0b011_111: "DDIVU", # Doubleword DIVide Unsigned

        0b100_000: "ADD", # ADD word
        0b100_001: "ADDU", # ADD Unsigned word
        0b100_010: "SUB", # Subtract word
        0b100_011: "SUBU", # SUBtract Unsigned word
        0b100_100: "AND", # AND
        0b100_101: "OR", # OR
        0b100_110: "XOR", # eXclusive OR
        0b100_111: "NOR", # Not OR

        # 0b101_000: "",
        # 0b101_001: "",
        0b101_010: "SLT", # Set on Less Than
        0b101_011: "SLTU", # Set on Less Than Unsigned
        0b101_100: "DADD", # Doubleword Add
        0b101_101: "DADDU", # Doubleword Add Unsigned
        0b101_110: "DSUB", # Doubleword SUBtract
        0b101_111: "DSUBU", # Doubleword SUBtract Unsigned

        0b110_000: "TGE", # Trap if Greater or Equal
        0b110_001: "TGEU", # Trap if Greater or Equal Unsigned
        0b110_010: "TLT", # Trap if Less Than
        0b110_011: "TLTU", # Trap if Less Than Unsigned
        0b110_100: "TEQ", # Trap if EQual
        # 0b110_101: "",
        0b110_110: "TNE", # Trap if Not Equal
        # 0b110_111: "",

        0b111_000: "DSLL", # Doubleword Shift Left Logical
        # 0b111_001: "",
        0b111_010: "DSRL", # Doubleword Shift Right Logical
        0b111_011: "DSRA", # Doubleword Shift Right Arithmetic
        0b111_100: "DSLL32", # Doubleword Shift Left Logical plus 32
        # 0b111_101: "",
        0b111_110: "DSRL32", # Doubleword Shift Right Logical plus 32
        0b111_111: "DSRA32", # Doubleword Shift Right Arithmetic plus 32
    }

    def isImplemented(self) -> bool:
        if self.function not in InstructionSpecial.SpecialOpcodes:
            return False
        opcode = self.getOpcodeName()
        if opcode in ("MOVCI", ):
            # TODO
            return False
        return True


    def isBranch(self) -> bool:
        return False
    def isTrap(self) -> bool:
        opcode = self.getOpcodeName()
        return opcode in ("TGE", "TGEU", "TLT", "TLTU", "TEQ", "TNE")


    def isRType(self) -> bool: # OP rd, rs, rt
        if self.isRType2():
            return False
        elif self.isSaType():
            return False
        elif self.instr == 0x0:
            return False
        return True # Not for all cases, but good enough
    def isRType2(self) -> bool: # OP rd, rt, rs
        opcode = self.getOpcodeName()
        return opcode in ("DSLLV", "DSRLV", "DSRAV", "SLLV", "SRLV", "SRAV")
    def isSaType(self) -> bool: # OP rd, rt, sa
        opcode = self.getOpcodeName()
        return opcode in ("SLL", "SRL", "SRA", "DSLL", "DSRL", "DSRA", "DSLL32", "DSRL32", "DSRA32")

    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.opcode != other.opcode:
            return False

        return self.function == other.function


    def modifiesRt(self) -> bool:
        return False
    def modifiesRd(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("JR", "JALR", "MTHI", "MTLO", "MULT", "MULTU", "DIV", "DIVU", "DMULT", "DMULTU", "DDIV", "DDIVU", "SYSCALL", "BREAK", "SYNC"): # TODO
            return False
        if self.isTrap():
            return False
        return True


    def blankOut(self):
        self.rs = 0
        self.rt = 0
        self.rd = 0
        self.sa = 0


    def getOpcodeName(self) -> str:
        if self.instr == 0:
            return "NOP"
        opcode = toHex(self.function, 2)
        name = InstructionSpecial.SpecialOpcodes.get(self.function, f"SPECIAL({opcode})")
        if name == "OR":
            if self.rt == 0:
                return "MOVE"
        return name


    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        rd = self.getRegisterName(self.rd)

        if opcode == "MOVCI": # Hack until I implement MOVCI instructions
            instr_index = toHex(self.instr_index, 7)
            return f"{formated_opcode} {instr_index}"

        if opcode == "NOP":
            return "nop"

        elif opcode == "MOVE": # OP rd, rs
            result = f"{formated_opcode} {rd},"
            result = result.ljust(14, ' ')
            return f"{result} {rs}"

        elif opcode in ("JR", "MTHI", "MTLO"):
            result = f"{formated_opcode} {rs}"
            return result
        elif opcode == "JALR":
            if self.rd == 31:
                rd = ""
            else:
                rd += ","
                rd = rd.ljust(6, ' ')
            result = f"{formated_opcode} {rd}{rs}"
            return result
        elif opcode in ("MFHI", "MFLO"):
            return f"{formated_opcode} {rd}"
        elif opcode in ("MULT", "MULTU",
                "DMULT", "DMULTU", "DDIVU") or self.isTrap(): # OP  rs, rt
            result = f"{formated_opcode} {rs},".ljust(14, ' ')
            return f"{result} {rt}"
        elif opcode in ("SYSCALL", "BREAK", "SYNC"):
            code = (self.instr_index) >> 16
            result = f"{formated_opcode} {code}"
            return result

        elif self.isRType(): # OP rd, rs, rt
            result = f"{formated_opcode} {rd},"
            result = result.ljust(14, ' ')
            result += f" {rs},"
            result = result.ljust(19, ' ')
            return f"{result} {rt}"
        elif self.isRType2(): # OP rd, rt, rs
            result = f"{formated_opcode} {rd},"
            result = result.ljust(14, ' ')
            result += f" {rt},"
            result = result.ljust(19, ' ')
            return f"{result} {rs}"
        elif self.isSaType(): # OP rd, rt, sa
            result = f"{formated_opcode} {rd},"
            result = result.ljust(14, ' ')
            result += f" {rt},"
            result = result.ljust(19, ' ')
            return f"{result} {self.sa}"

        return super().disassemble(context)

