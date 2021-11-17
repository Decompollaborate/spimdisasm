#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionNormal(InstructionBase):
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
        0b101_111: "CACHE", # Cache

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

    def isImplemented(self) -> bool:
        if self.opcode not in InstructionNormal.NormalOpcodes:
            return False
        opcode = self.getOpcodeName()
        if opcode in ("SPECIAL", "REGIMM", "COP0", "COP1", "COP2", "COP3"):
            return False
        if opcode in ("LWC2", "SWC2", "LDC2", "SDC2"):
            # TODO
            return False
        return True

    def isFloatInstruction(self) -> bool:
        if self.isDoubleFloatInstruction():
            return True
        opcode = self.getOpcodeName()
        if opcode in ("LWC1", "SWC1"):
            return True
        return False

    def isDoubleFloatInstruction(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("LDC1", "SDC1"):
            return True
        return False


    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BEQ", "BEQL", "BLEZ", "BLEZL", "BNE", "BNEL", "BGTZ", "BGTZL"):
            return True
        return super().isBranch()

    # OP LABEL
    def isJType(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("J", "JAL"):
            return True
        return super().isJType()

    def isIType(self) -> bool:
        if self.isBranch():
            return False
        if self.isJType():
            return False
        return True

    # OP  rs, IMM
    def isUnaryBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BLEZ", "BGTZ", "BLEZL", "BGTZL"):
            return True
        return False

    # OP  rs, rt, IMM
    def isBinaryBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode == "BEQ" or opcode == "BEQL":
            return True
        if opcode == "BNE" or opcode == "BNEL":
            return True
        return False

    # OP  rt, IMM
    def isUnaryOperation(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("LUI", ):
            return True
        return False

    # OP  rt, rs, IMM
    def isBinaryOperation(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("ADDI", "ADDIU", "ANDI", "DADDI", "DADDIU", "ORI", "XORI", "SLTI", "SLTIU"):
            return True
        return False

    def isOperation(self) -> bool:
        return self.isBinaryOperation() or self.isUnaryOperation()

    def isUnsigned(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("LUI", "ANDI", "ORI", "XORI", ):
            return True
        return False

    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.opcode != other.opcode:
            return False

        return self.isImplemented()


    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        if self.isJType():
            return False
        opcode = self.getOpcodeName()
        if opcode in ("SB", "SH", "SWL", "SW", "SDL", "SDR", "SWR"):
            return False
        if opcode in ("LWC1", "LWC2", "LDC1", "LDC2"): # Changes the value of the coprocessor's register
            return False
        if opcode in ("SWC1", "SWC2", "SDC1", "SDC2"):
            return False
        return super().modifiesRt()


    def getOpcodeName(self) -> str:
        if self.opcode in InstructionNormal.NormalOpcodes:
            return InstructionNormal.NormalOpcodes[self.opcode]
        return super().getOpcodeName()


    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        #immediate = toHex(self.immediate, 4)
        immediate = hex(self.immediate)
        if not self.isUnsigned():
            immediate = hex(from2Complement(self.immediate, 16))
        if immOverride is not None:
            immediate = immOverride

        result = f"{formated_opcode} "

        if "COP" in opcode: # Hack until I implement COPz instructions
            instr_index = toHex(self.instr_index, 7)
            result += instr_index
            return result

        if self.isJType():
            # instr_index = toHex(self.instr_index, 7)
            # return f"{opcode} {instr_index}"
            vram = (self.instr_index<<2) | 0x80000000
            instrIndexHex = toHex(vram, 6)[2:]
            label = f"func_{instrIndexHex}"
            if context is not None:
                symbol = context.getAnySymbol(vram)
                if symbol is not None:
                    #label = f"{symbol} # func_{instrIndexHex}"
                    label = f"{symbol}"
            return f"{result}{label}"

        if self.isBranch():
            result += f"{rs},"
            result = result.ljust(14, ' ')
            # OP  rs, IMM
            if self.isUnaryBranch():
                pass
            # OP  rs, rt, IMM
            elif self.isBinaryBranch():
                result += f" {rt},"
                result = result.ljust(19, ' ')
                if opcode == "BEQ":
                    if self.rs == 0 and self.rt == 0:
                        result = "b".ljust(self.ljustWidthOpcode, ' ')
                    #elif self.rt == 0:
                    #    result = "beqz".ljust(self.ljustWidthOpcode, ' ')
                    #    result += f" {rs},"
                #else:
                #    if self.rt == 0:
                #        result = opcode.lower() +"z"
                #        result = result.ljust(self.ljustWidthOpcode, ' ')
                #        result += f" {rs},"
            return f"{result} {immediate}"

        if self.isOperation():
            result += f"{rt},"
            result = result.ljust(14, ' ')
            # OP  rt, IMM
            if self.isUnaryOperation():
                pass
            # OP  rt, rs, IMM
            elif self.isBinaryOperation():
                result += f" {rs},"
                result = result.ljust(19, ' ')
            return f"{result} {immediate}"

        # OP rt, IMM(rs)
        if self.isFloatInstruction():
            result += f"{self.getFloatRegisterName(self.rt)}"
        elif opcode == "CACHE":
            result += f"{toHex(self.rt, 2)}"
        else:
            result += f"{rt}"

        result = result.ljust(14, ' ')
        return f"{result}, {immediate}({rs})"
