#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class Instruction:
    NormalOpcodes = {
        0b000_000: "SPECIAL",
        0b000_001: "REGIMM",
        0b000_010: "J", # Jump
        0b000_011: "JAL", # Jump And Link
        0b000_100: "BEQ", # Branch on EQual
        0b000_101: "BNE", # Branch on Not Equal
        0b000_110: "BLEZ", # Branch on Less than or Equal to Zero
        0b000_111: "BGTZ", # Branch on Greater Than Zero

        0b001_000: "ADDI", # Add Immediate
        0b001_001: "ADDIU", # Add Immediate Unsigned Word
        0b001_010: "SLTI", # Set on Less Than Immediate
        0b001_011: "SLTIU", # Set on Less Than Immediate Unsigned
        0b001_100: "ANDI", # And Immediate
        0b001_101: "ORI", # Or Immediate
        0b001_110: "XORI", # eXclusive OR Immediate
        0b001_111: "LUI", # Load Upper Immediate

        0b010_000: "COP0", # Coprocessor OPeration z
        0b010_001: "COP1", # Coprocessor OPeration z
        0b010_010: "COP2", # Coprocessor OPeration z
        0b010_011: "COP3", # Coprocessor OPeration z
        0b010_100: "BEQL", # Branch on EQual Likely
        0b010_101: "BNEL", # Branch on Not Equal Likely
        0b010_110: "BLEZL", # Branch on Less than or Equal to Zero Likely
        0b010_111: "BGTZL", # Branch on Greater Than Zero Likely

        0b011_000: "DADDI", # Doubleword add Immediate
        0b011_001: "DADDIU", # Doubleword add Immediate Unsigned
        0b011_010: "LDL", # Load Doubleword Left
        0b011_011: "LDR", # Load Doubleword Right
        # 0b011_100: "",
        # 0b011_101: "",
        # 0b011_110: "",
        # 0b011_111: "",

        0b100_000: "LB", # Load Byte
        0b100_001: "LH", # Load Halfword
        0b100_010: "LWL", # Load Word Left
        0b100_011: "LW", # Load Word
        0b100_100: "LBU", # Load Byte Insigned
        0b100_101: "LHU", # Load Halfword Unsigned
        0b100_110: "LWR", # Load Word Right
        0b100_111: "LWU", # Load Word Unsigned

        0b101_000: "SB", # Store Byte
        0b101_001: "SH", # Store Halfword
        0b101_010: "SWL", # Store Word Left
        0b101_011: "SW", # Store Word
        0b101_100: "SDL", # Store Doubleword Left
        0b101_101: "SDR", # Store Doubleword Right
        0b101_110: "SWR", # Store Word Right
        # 0b101_111: "",

        0b110_000: "LL", # Load Linked word
        0b110_001: "LWC1", # Load Word to Coprocessor z
        0b110_010: "LWC2", # Load Word to Coprocessor z
        0b110_011: "PREF", # Prefetch
        0b110_100: "LLD", # Load Linked Doubleword
        0b110_101: "LDC1", # Load Doubleword to Coprocessor z
        0b110_110: "LDC2", # Load Doubleword to Coprocessor z
        0b110_111: "LD", # Load Doubleword

        0b111_000: "SC", # Store Conditional word
        0b111_001: "SWC1", # Store Word from Coprocessor z
        0b111_010: "SWC2", # Store Word from Coprocessor z
        # 0b111_011: "",
        0b111_100: "SCD", # Store Conditional Doubleword
        0b111_101: "SDC1", # Store Doubleword from Coprocessor z
        0b111_110: "SDC2", # Store Doubleword from Coprocessor z
        0b111_111: "SD", # Store Doubleword
    }

    def __init__(self, instr: int):
        self.opcode = (instr >> 26) & 0x3F
        self.rs = (instr >> 21) & 0x1F # rs
        self.rt = (instr >> 16) & 0x1F # usually the destiny of the operation
        self.rd = (instr >> 11) & 0x1F # destination register in R-Type instructions
        self.sa = (instr >>  6) & 0x1F
        self.function = (instr >> 0) & 0x3F

    @property
    def instr(self) -> int:
        return (self.opcode << 26) | (self.rs << 21) | (self.rt << 16) | (self.immediate)

    @property
    def immediate(self) -> int:
        return (self.rd << 11) | (self.sa << 6) | (self.function)
    @property
    def instr_index(self) -> int:
        return (self.rs << 21) | (self.rt << 16) | (self.immediate)
    @property
    def baseRegister(self) -> int:
        return self.rs

    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode == "BEQ" or opcode == "BEQL":
            return True
        if opcode == "BLEZ" or opcode == "BLEZL":
            return True
        if opcode == "BNE" or opcode == "BNEL":
            return True
        if opcode in ("BGTZ", "BGTZL"):
            return True
        return False
    def isTrap(self) -> bool:
        return False

    def isJType(self) -> bool: # OP LABEL
        opcode = self.getOpcodeName()
        return opcode in ("J", "JAL")
    def isRType(self) -> bool: # OP rd, rs, rt
        return False
    def isRType2(self) -> bool: # OP rd, rt, rs
        return False
    def isSaType(self) -> bool: # OP rd, rt, sa
        return False
    def isIType(self) -> bool: # OP rt, IMM(rs)
        if self.isJType():
            return False
        if self.isRType():
            return False
        if self.isRType2():
            return False
        if self.isSaType():
            return False
        if self.isIType2():
            return False
        if self.isIType3():
            return False
        if self.isIType4():
            return False
        return True
    def isIType2(self) -> bool: # OP  rs, rt, IMM
        opcode = self.getOpcodeName()
        if opcode == "BEQ" or opcode == "BEQL":
            return True
        if opcode == "BNE" or opcode == "BNEL":
            return True
        return False
    def isIType3(self) -> bool: # OP  rt, rs, IMM
        opcode = self.getOpcodeName()
        if opcode == "ADDI" or opcode == "ADDIU":
            return True
        if opcode == "ANDI":
            return True
        if opcode == "DADDI" or opcode == "DADDIU":
            return True
        if opcode == "ORI" or opcode == "XORI":
            return True
        if opcode == "SLTI" or opcode == "SLTIU":
            return True
        return False
    def isIType4(self) -> bool: # OP  rs, IMM
        opcode = self.getOpcodeName()
        if opcode in ("BLEZ", "BGTZ", "BLEZL", "BGTZL"):
            return True
        return False
    def isIType5(self) -> bool: # OP  rt, IMM
        opcode = self.getOpcodeName()
        if opcode in ("LUI", ):
            return True
        return False
    def isMType(self) -> bool: # OP rd, rs
        return False

    def sameOpcode(self, other: Instruction) -> bool:
        return self.opcode == other.opcode

    def sameBaseRegister(self, other: Instruction):
        return self.baseRegister == other.baseRegister

    def sameOpcodeButDifferentArguments(self, other: Instruction) -> bool:
        if not self.sameOpcode(other):
            return False
        return self.instr != other.instr

    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        opcode = self.getOpcodeName()
        if opcode in ("SB", "SH", "SWL", "SW", "SDL", "SDR", "SWR"):
            return False
        if opcode in ("LWC1", "LWC2", "LDC1", "LDC2"): # Changes the value of the coprocessor's register
            return False
        if opcode in ("SWC1", "SWC2", "SDC1", "SDC2"):
            return False
        return True
    def modifiesRd(self) -> bool:
        return False

    def blankOut(self):
        self.rs = 0
        self.rt = 0
        self.rd = 0
        self.sa = 0
        self.function = 0

    def getOpcodeName(self) -> str:
        opcode = toHex(self.opcode, 2)
        return Instruction.NormalOpcodes.get(self.opcode, f"({opcode})")

    def getRegisterName(self, register: int) -> str:
        if register == 0:
            return "$zero"
        elif register == 1:
            return "$at"
        elif 2 <= register <= 3:
            return "$v" + str(register-2)
        elif 4 <= register <= 7:
            return "$a" + str(register-4)
        elif 8 <= register <= 15:
            return "$t" + str(register-8)
        elif 16 <= register <= 23:
            return "$s" + str(register-16)
        elif 24 <= register <= 25:
            return "$t" + str(register-24 + 8)
        elif 26 <= register <= 27:
            return "$k" + str(register-26)
        elif register == 28:
            return "$gp"
        elif register == 29:
            return "$sp"
        elif register == 30:
            return "$fp"
        elif register == 31:
            return "$ra"
        return hex(register)

    def getFloatRegisterName(self, register: int) -> str:
        if 0 <= register <= 31:
            return "$f" + str(register)
        return hex(register)

    def __str__(self) -> str:
        opcode = self.getOpcodeName().lower().ljust(7, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        immediate = toHex(self.immediate, 4)

        if "COP" in self.getOpcodeName(): # Hack until I implement COPz instructions
            instr_index = toHex(self.instr_index, 7)
            return f"{opcode} {instr_index}"

        if self.getOpcodeName() == "NOP":
            return "nop"
        if self.isIType5():
            result = f"{opcode} {rt},"
            result = result.ljust(14, ' ')
            return f"{result} {immediate}"
        elif self.isIType():
            # TODO: use float registers
            result = f"{opcode} {rt},"
            result = result.ljust(14, ' ')
            return f"{result} {immediate}({rs})"
        elif self.isIType2():
            result = f"{opcode} {rs},"
            result = result.ljust(14, ' ')
            result += f" {rt},"
            result = result.ljust(19, ' ')
            if self.getOpcodeName() == "BEQ":
                if self.rs == 0 and self.rt == 0:
                    result = "b".ljust(7, ' ')
            return f"{result} {immediate}"
        elif self.isIType3():
            result = f"{opcode} {rt},"
            result = result.ljust(14, ' ')
            result += f" {rs},"
            result = result.ljust(19, ' ')
            return f"{result} {immediate}"
        elif self.isIType4():
            result = f"{opcode} {rs},"
            result = result.ljust(14, ' ')
            return f"{result} {immediate}"
        elif self.isJType():
            # instr_index = toHex(self.instr_index, 7)
            # return f"{opcode} {instr_index}"
            instrIndexHex = toHex(self.instr_index<<2, 6)[2:]
            label = f"func_80{instrIndexHex}"
            #if (self.instr_index<<2) % 16 == 0 and (self.instr_index<<2) & 0x800000:
                #print(label)
            return f"{opcode} {label}"
        elif self.isRType():
            rd = self.getRegisterName(self.rd)
            result = f"{opcode} {rd},"
            result = result.ljust(14, ' ')
            result += f" {rs},"
            result = result.ljust(19, ' ')
            return f"{result} {rt}"
        elif self.isRType2():
            rd = self.getRegisterName(self.rd)
            result = f"{opcode} {rd},"
            result = result.ljust(14, ' ')
            result += f" {rt},"
            result = result.ljust(19, ' ')
            return f"{result} {rs}"
        elif self.isSaType():
            rd = self.getRegisterName(self.rd)
            result = f"{opcode} {rd},"
            result = result.ljust(14, ' ')
            result += f" {rt},"
            result = result.ljust(19, ' ')
            return f"{result} {self.sa}"
        elif self.isMType():
            rd = self.getRegisterName(self.rd)
            result = f"{opcode} {rd},"
            result = result.ljust(14, ' ')
            return f"{result} {rs}"
        return "ERROR"

    def __repr__(self) -> str:
        return self.__str__()

class InstructionSpecial(Instruction):
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

    def isTrap(self) -> bool:
        opcode = self.getOpcodeName()
        return opcode in ("TGE", "TGEU", "TLT", "TLTU", "TEQ", "TNE")

    def isJType(self) -> bool: # OP LABEL
        return False
    def isRType(self) -> bool: # OP rd, rs, rt
        if self.isRType2():
            return False
        elif self.isSaType():
            return False
        elif self.isMType():
            return False
        return True # Not for all cases, but good enough
    def isRType2(self) -> bool: # OP rd, rt, rs
        opcode = self.getOpcodeName()
        return opcode in ("DSLLV", "DSRLV", "DSRAV")
    def isSaType(self) -> bool: # OP rd, rt, sa
        opcode = self.getOpcodeName()
        return opcode in ("SLL", "SRL", "SRA", "DSLL", "DSRL", "DSRA", "DSLL32", "DSRL32", "DSRA32")
    def isIType(self) -> bool: # OP rt, IMM(rs)
        return False
    def isIType2(self) -> bool: # OP  rs, rt, IMM
        return False
    def isMType(self) -> bool: # OP rd, rs
        opcode = self.getOpcodeName()
        return opcode in ("MOVE",)

    def modifiesRt(self) -> bool:
        return False
    def modifiesRd(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("JR", "JALR", "MTHI", "MTLO", "MULT", "MULTU", "DIV", "DIVU", "DMULT", "DMULTU", "DDIV", "DDIVU", "SYSCALL", "BREAK", "SYNC"): # TODO
            return False
        if self.isTrap():
            return False
        return True

    def getOpcodeName(self) -> str:
        if self.instr == 0:
            return "NOP"
        opcode = toHex(self.function, 2)
        name = InstructionSpecial.SpecialOpcodes.get(self.function, f"SPECIAL({opcode})")
        if name == "OR":
            if self.rt == 0:
                return "MOVE"
        return name

    def __str__(self) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(7, ' ')

        if opcode == "MOVCI": # Hack until I implement MOVCI instructions
            instr_index = toHex(self.instr_index, 7)
            return f"{formated_opcode} {instr_index}"

        if opcode in ("JR", "MTHI", "MTLO"):
            rs = self.getRegisterName(self.rs)
            result = f"{formated_opcode} {rs}"
            return result
        elif opcode == "JALR":
            rs = self.getRegisterName(self.rs)
            rd = ""
            if self.rd != 31:
                rd = self.getRegisterName(self.rd) + ","
                rd = rd.ljust(6, ' ')
            result = f"{formated_opcode} {rd}{rs}"
            return result
        elif opcode in ("MFHI", "MFLO"):
            rd = self.getRegisterName(self.rd)
            return f"{formated_opcode} {rd}"
        elif opcode in ("MULT", "MULTU", "DIV", "DIVU", 
                "DMULT", "DMULTU", "DDIV", "DDIVU") or self.isTrap(): # OP  rs, rt
            rs = self.getRegisterName(self.rs)
            rt = self.getRegisterName(self.rt)
            result = f"{formated_opcode} {rs},".ljust(14, ' ')
            return f"{result} {rt}"
        elif opcode in ("SYSCALL", "BREAK", "SYNC"):
            code = (self.instr_index) >> 6
            result = f"{formated_opcode} {code}"
            return result
        return super().__str__()

class InstructionRegimm(Instruction):
    RegimmOpcodes = {
        0b00_000: "BLTZ",
        0b00_001: "BGEZ",
        0b00_010: "BLTZL",
        0b00_011: "BGEZL",

        0b01_000: "TGEI",
        0b01_001: "TGEIU",
        0b01_010: "TLTI",
        0b01_011: "TLTIU",

        0b10_000: "BLTZAL",
        0b10_001: "BGEZAL",
        0b10_010: "BLTZALL",
        0b10_011: "BGEZALL",

        0b01_100: "TEQI",
        0b01_110: "TNEI",
    }

    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BLTZ", "BGEZ", "BLTZL", "BGEZL"):
            return True
        if opcode in ("BLTZAL", "BGEZAL", "BLTZALL", "BGEZALL"):
            return True
        return False
    def isTrap(self) -> bool:
        opcode = self.getOpcodeName()
        return opcode in ("TGEI", "TGEIU", "TLTI", "TLTIU", "TEQI", "TNEI")

    def isJType(self) -> bool: # OP LABEL
        return False
    def isRType(self) -> bool: # OP rd, rs, rt
        return False
    def isIType(self) -> bool: # OP rt, IMM(rs)
        return False
    def isIType2(self) -> bool: # OP  rs, rt, IMM
        return False

    def modifiesRt(self) -> bool:
        return False
    def modifiesRd(self) -> bool:
        return False

    def getOpcodeName(self) -> str:
        opcode = toHex(self.rt, 2)
        return InstructionRegimm.RegimmOpcodes.get(self.rt, f"REGIMM({opcode})")

    def __str__(self) -> str:
        opcode = self.getOpcodeName().lower().ljust(7, ' ')
        rs = self.getRegisterName(self.rs)
        immediate = toHex(self.immediate, 4)

        result = f"{opcode} {rs},"
        result = result.ljust(14, ' ')
        return f"{result} {immediate}"

def wordToInstruction(word: int) -> Instruction:
    if ((word >> 26) & 0xFF) == 0x00:
        return InstructionSpecial(word)
    if ((word >> 26) & 0xFF) == 0x01:
        return InstructionRegimm(word)
    if ((word >> 26) & 0xFF) == 0x11:
        # COP1
        pass
    return Instruction(word)


