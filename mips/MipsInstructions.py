#!/usr/bin/python3

from __future__ import annotations

from .Utils import *


class Instruction:
    def __init__(self, instr: int):
        self.opcode = (instr >> 26) & 0x3F
        self.baseRegister = (instr >> 21) & 0x1F # rs
        self.rt = (instr >> 16) & 0x1F # usually the destiny of the operation
        self.immediate = (instr) & 0xFFFF

    @property
    def instr(self):
        return (self.opcode << 26) | (self.baseRegister << 21) | (self.rt << 16) | (self.immediate)

    def isLUI(self) -> bool: # Load Upper Immediate
        return self.opcode == (0x3C >> 2) # 0b001111
    def isADDIU(self) -> bool:
        return self.opcode == (0x24 >> 2) # 0b001001
    def isLW(self) -> bool: # Load Word
        return self.opcode == (0x8C >> 2) # 0b100011
    def isLWCz(self) -> bool: # Load Word to Coprocessor
        if (self.opcode & 0x03) == 0x00:
            return False
        return (self.opcode & 0x3C) == (0xC0 >> 2) # 0b1100zz
    def isANDI(self) -> bool:
        return self.opcode == (0x30 >> 2) # 0b001100
    def isORI(self) -> bool: # Or Immediate
        return self.opcode == (0x34 >> 2) # 0b001101
    def isADDI(self) -> bool:
        return self.opcode == (0x20 >> 2) # 0b001000
    def isDADDI(self) -> bool: # Doubleword add Immediate
        return self.opcode == (0x60 >> 2) # 0b011000
    def isDADDIU(self) -> bool: # Doubleword add Immediate Unsigned
        return self.opcode == (0x64 >> 2) # 0b011001

    def isBEQ(self) -> bool:
        return self.opcode == (0x10 >> 2) # 0b000100
    def isBEQL(self) -> bool:
        return self.opcode == (0x50 >> 2) # 0b010100
    def isBLEZ(self) -> bool:
        return self.opcode == (0x18 >> 2) # 0b000110
    def isBLEZL(self) -> bool:
        return self.opcode == (0x58 >> 2) # 0b010110
    def isBGTZ(self) -> bool:
        return self.opcode == (0x1C >> 2) # 0b000111
    def isBGTZL(self) -> bool:
        return self.opcode == (0x5C >> 2) # 0b010111
    def isBNE(self) -> bool:
        return self.opcode == (0x14 >> 2) # 0b000101
    def isBNEL(self) -> bool:
        return self.opcode == (0x54 >> 2) # 0b010101

    def isJ(self) -> bool: # Jump
        return self.opcode == (0x08 >> 2) # 0b000010
    def isJAL(self) -> bool: # Jump and Link
        return self.opcode == (0x0C >> 2) # 0b000011
    # JALR
    # JR

    def isBranch(self) -> bool:
        return (self.isBEQ() or self.isBEQL() or self.isBLEZ() or self.isBLEZL() 
                or self.isBGTZ() or self.isBGTZL() or self.isBNE() or self.isBNEL() 
                or self.isJ() or self.isJAL())

    def isLB(self) -> bool: # Load Byte
        return self.opcode == (0x80 >> 2) # 0b100000
    def isLBU(self) -> bool: # Load Byte Insigned
        return self.opcode == (0x90 >> 2) # 0b100100

    def isLD(self) -> bool: # Load Doubleword
        return self.opcode == (0xDC >> 2) # 0b110111

    def isLDCz(self) -> bool: # Load Doubleword to Coprocessor z
        if (self.opcode & 0x03) == 0x00:
            return False
        return (self.opcode & 0x3C) == (0xD0 >> 2) # 0b1101zz

    def isLDL(self) -> bool: # Load Doubleword Left
        return self.opcode == (0x68 >> 2) # 0b011010
    def isLDR(self) -> bool: # Load Doubleword Right
        return self.opcode == (0x6C >> 2) # 0b011011

    def isLH(self) -> bool: # Load Halfword
        return self.opcode == (0x84 >> 2) # 0b100001
    def isLHU(self) -> bool: # Load Halfword Unsigned
        return self.opcode == (0x94 >> 2) # 0b100101

    def isLL(self) -> bool: # Load Linked word
        return self.opcode == (0xC0 >> 2) # 0b110000
    def isLLD(self) -> bool: # Load Linked Doubleword
        return self.opcode == (0xD0 >> 2) # 0b110100

    def isLWL(self) -> bool: # Load Word Left
        return self.opcode == (0x88 >> 2) # 0b100010
    def isLWR(self) -> bool: # Load Word Right
        return self.opcode == (0x98 >> 2) # 0b100110

    def isLWU(self) -> bool: # Load Word Unsigned
        return self.opcode == (0x94 >> 2) # 0b100111

    # PREF # Prefetch
    # 0b110011

    def isSB(self) -> bool: # Store Byte
        return self.opcode == (0xA0 >> 2) # 0b101000
    def isSC(self) -> bool: # Store Conditional word
        return self.opcode == (0xE0 >> 2) # 0b111000
    def isSCD(self) -> bool: # Store Conditional Doubleword
        return self.opcode == (0xF0 >> 2) # 0b111100
    def isSD(self) -> bool: # Store Doubleword
        return self.opcode == (0xFC >> 2) # 0b111111

    def isSDCz(self) -> bool: # Store Doubleword from Coprocessor
        if (self.opcode & 0x03) == 0x00:
            return False
        return (self.opcode & 0x3C) == (0xF0 >> 2) # 0b1111zz

    def isSDL(self) -> bool: # Store Doubleword Left
        return self.opcode == (0xB0 >> 2) # 0b101100
    def isSDR(self) -> bool: # Store Doubleword Right
        return self.opcode == (0xB4 >> 2) # 0b101101

    def isCOPz(self) -> bool: # Coprocessor OPeration
        if (self.opcode & 0x03) == 0x00:
            return False
        return (self.opcode & 0x3C) == (0x40 >> 2) # 0b0100zz

    def isSH(self) -> bool: # Store Halfword
        return self.opcode == (0xA4 >> 2) # 0b101001

    # SLL # Shift word Left Logical
    # SLLV # Shift word Left Logical Variable
    # SLT # Set on Less Than

    def isSLTI(self) -> bool: # Set on Less Than Immediate
        return self.opcode == (0x28 >> 2) # 0b001010
    def isSLTIU(self) -> bool: # Set on Less Than Immediate Unsigned
        return self.opcode == (0x2C >> 2) # 0b001011

    # SLTU # Set on Less Than Unsigned

    # SRA # Shift word Right Arithmetic
    # SRAV # Shift word Right Arithmetic Variable
    # SRL # Shift word Right Logical
    # SRLV # Shift word Right Logical Variable

    # SUB # Subtract word
    # SUBU # Subtract Unsigned word

    def isSW(self) -> bool: # Store Word
        return self.opcode == (0xAC >> 2) # 0b101011
    def isSWCz(self) -> bool: # Store Word from Coprocessor z
        if (self.opcode & 0x03) == 0x00:
            return False
        return (self.opcode & 0x3C) == (0xE0 >> 2) # 0b1110zz

    def isSWL(self) -> bool: # Store Word Left
        return self.opcode == (0xA8 >> 2) # 0b101010
    def isSWR(self) -> bool: # Store Word Right
        return self.opcode == (0xB8 >> 2) # 0b101110

    # XOR # eXclusive OR

    def isXORI(self) -> bool: # eXclusive OR Immediate
        return self.opcode == (0x38 >> 2) # 0b001110

    def isSPECIAL(self) -> bool:
        return self.opcode == 0x00 # 0b000000
    def isREGIMM(self) -> bool:
        return self.opcode == 0x01 # 0b000001

    def sameOpcode(self, other: Instruction) -> bool:
        return self.opcode == other.opcode

    def sameBaseRegister(self, other: Instruction):
        return self.baseRegister == other.baseRegister

    def sameOpcodeButDifferentArguments(self, other: Instruction) -> bool:
        if not self.sameOpcode(other):
            return False
        return self.instr != other.instr

    def blankOut(self):
        self.baseRegister = 0
        self.rt = 0
        self.immediate = 0

    def __str__(self) -> str:
        result = ""
        if self.isLUI():
            result += "LUI"
        elif self.isADDIU():
            result += "ADDIU"
        elif self.isLW():
            result += "LW"
        elif self.isLWCz():
            result += f"LWC{self.opcode&0x3}"
        elif self.isANDI():
            result += "ANDI"
        elif self.isORI():
            result += "ORI"
        elif self.isADDI():
            result += "ADDI"
        elif self.isDADDI():
            result += "DADDI"
        elif self.isDADDIU():
            result += "DADDIU"

        elif self.isBEQ():
            result += "BEQ"
        elif self.isBEQL():
            result += "BEQL"
        elif self.isBLEZ():
            result += "BLEZ"
        elif self.isBLEZL():
            result += "BLEZL"
        elif self.isBGTZ():
            result += "BGTZ"
        elif self.isBGTZL():
            result += "BGTZL"
        elif self.isBNE():
            result += "BNE"
        elif self.isBNEL():
            result += "BNEL"

        elif self.isJ():
            result += "J"
        elif self.isJAL():
            result += "JAL"

        elif self.isLB():
            result += "LB"
        elif self.isLBU():
            result += "LBU"

        elif self.isLD():
            result += "LD"

        elif self.isLDCz():
            result += f"LDC{self.opcode&0x3}"

        elif self.isLDL():
            result += "LDL"
        elif self.isLDR():
            result += "LDR"

        elif self.isLH():
            result += "LH"
        elif self.isLHU():
            result += "LHU"

        elif self.isLL():
            result += "LL"
        elif self.isLLD():
            result += "LLD"

        elif self.isLWL():
            result += "LWL"
        elif self.isLWR():
            result += "LWR"

        elif self.isLWU():
            result += "LWU"

        elif self.isSB():
            result += "SB"
        elif self.isSC():
            result += "SC"
        elif self.isSCD():
            result += "SCD"
        elif self.isSD():
            result += "SD"

        elif self.isSDCz():
            result += f"SDC{self.opcode&0x3}"

        elif self.isSDL():
            result += "SDL"
        elif self.isSDR():
            result += "SDR"

        elif self.isCOPz():
            result += f"COP{self.opcode&0x3}"

        elif self.isSH():
            result += "SH"

        # SLL # Shift word Left Logical
        # SLLV # Shift word Left Logical Variable
        # SLT # Set on Less Than

        elif self.isSLTI():
            result += "SLTI"
        elif self.isSLTIU():
            result += "SLTIU"

        # SLTU # Set on Less Than Unsigned

        # SRA # Shift word Right Arithmetic
        # SRAV # Shift word Right Arithmetic Variable
        # SRL # Shift word Right Logical
        # SRLV # Shift word Right Logical Variable

        # SUB # Subtract word
        # SUBU # Subtract Unsigned word

        elif self.isSW():
            result += "SW"
        elif self.isSWCz():
            result += f"SWC{self.opcode&0x3}"

        elif self.isSWL():
            result += "SWL"
        elif self.isSWR():
            result += "SWR"

        # XOR # eXclusive OR

        elif self.isXORI():
            result += "XORI"

        elif self.isCOPz():
            result += f"COP{self.opcode&0x3}"

        elif self.isSPECIAL():
            result += "SPECIAL"
        elif self.isREGIMM():
            result += "REGIMM"

        else:
            result += hex(self.opcode)
            eprint(f"Unknown opcode: {result}")
        return f"{result} {hex(self.baseRegister)} {hex(self.rt)} {hex(self.immediate)}"

    def __repr__(self) -> str:
        return self.__str__()
