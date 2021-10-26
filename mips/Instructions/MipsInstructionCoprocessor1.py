#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionCoprocessor1(InstructionBase):
    Cop1Opcodes_ByFormat = {
        0b00_000: "MFC1", # Move Word From Floating-Point
        0b00_001: "DMFC1", # Doubleword Move From Floating-Point
        0b00_010: "CFC1", # Move Control Word from Floating-Point

        0b00_100: "MTC1", # Move Word to Floating-Point
        0b00_101: "DMTC1", # Doubleword Move To Floating-Point
        0b00_110: "CTC1", # Move Control Word to Floating-Point
    }
    Cop1Opcodes_ByFunction = {
        0b000_000: "ADD", # Floating-Point Add # fd, fs, ft
        0b000_001: "SUB", # Floating-Point Add # fd, fs, ft
        0b000_010: "MUL", # Floating-Point Multiply # fd, fs, ft
        0b000_011: "DIV", # Floating-Point Divide # fd, fs, ft

        0b000_100: "SQRT", # Floating-Point Square Root # fd, fs
        0b000_101: "ABS", # Floating-Point Absolute Value # fd, fs
        0b000_110: "MOV", # Floating-Point Move # fd, fs
        0b000_111: "NEG", # Floating-Point Negate # fd, fs

        0b001_000: "ROUND.L", # Floating-Point Round to Long Fixed-Point # fd, fs
        0b001_001: "TRUNC.L", # Floating-Point Truncate to Long Fixed-Point # fd, fs
        0b001_010: "CEIL.L", # Floating-Point Ceiling Convert to Long Fixed-Point # fd, fs
        0b001_011: "FLOOR.L", # Floating-Point Floor Convert to Long Fixed-Point # fd, fs

        0b001_100: "ROUND.W", # Floating-Point Round to Word Fixed-Point # fd, fs
        0b001_101: "TRUNC.W", # Floating-Point Truncate to Word Fixed-Point # fd, fs
        0b001_110: "CEIL.W", # Floating-Point Ceiling Convert to Word Fixed-Point # fd, fs
        0b001_111: "FLOOR.W", # Floating-Point Floor Convert to Word Fixed-Point # fd, fs
    }
    CompareConditionsCodes = {
        0b0_000: "F", # False
        0b0_001: "UN", # UNordered
        0b0_010: "EQ", # EQual
        0b0_011: "UEQ", # Unordered or EQual
        0b0_100: "OLT", # Ordered or Less Than
        0b0_101: "ULT", # Unordered or Less Than
        0b0_110: "OLE", # Ordered or Less than or Equal
        0b0_111: "ULE", # Unordered or Less than or Equal

        0b1_000: "SF", # Signaling False
        0b1_001: "NGLE", # Not Greater than or Less than or Equal
        0b1_010: "SEQ", # Signaling Equal
        0b1_011: "NGL", # Not Greater than or Less than
        0b1_100: "LT", # Less than
        0b1_101: "NGE", # Not Greater than or Equal
        0b1_110: "LE", # Less than or Equal
        0b1_111: "NGT", # Not Greater than
    }

    def isImplemented(self) -> bool:
        if self.fmt in InstructionCoprocessor1.Cop1Opcodes_ByFormat:
            return True
        if self.fmt == 0b01_000: # fmt = BC
            return True
        if self.function in InstructionCoprocessor1.Cop1Opcodes_ByFunction:
            return True
        if self.fc == 0b11:
            return True
        if self.fc == 0b10:
            return True
        return False

    def isFloatInstruction(self) -> bool:
        return True

    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BC1T", "BC1TL", "BC1F", "BC1FL"):
            return True
        return False

    def isBinaryOperation(self) -> bool:
        opcode = self.getOpcodeName().split(".")[0]
        if opcode in ("ADD", "SUB", "MUL", "DIV"):
            return True
        return False


    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.opcode != other.opcode:
            return False

        if self.fmt == other.fmt:
            if self.fmt in InstructionCoprocessor1.Cop1Opcodes_ByFormat:
                return True
            if self.fmt == 0b01_000: # fmt = BC
                if self.tf == other.tf and self.nd == other.nd:
                    return True
                return False

            return self.function == other.function

        return False


    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        opcode = self.getOpcodeName()
        if opcode in ("MFC1", "DMFC1", "CFC1"):
            return True
        # TODO
        return super().modifiesRt()
    def modifiesRd(self) -> bool:
        opcode = self.getOpcodeName()
        # modifying fs shouldn't be the same as modifying rd
        #if opcode in ("MTC1", "DMTC1", "CTC1"):
        #    return True
        # TODO
        return super().modifiesRd()


    def blankOut(self):
        if self.fmt in InstructionCoprocessor1.Cop1Opcodes_ByFormat:
            self.rt = 0
            self.rd = 0
            self.sa = 0
            self.function = 0
        elif self.fmt == 0b01_000: # fmt = BC
            self.rd = 0
            self.sa = 0
            self.function = 0
        elif self.function in InstructionCoprocessor1.Cop1Opcodes_ByFunction:
            self.rt = 0
            self.rd = 0
            self.sa = 0
        elif self.fc == 0b11 or self.fc == 0b10:
            self.rt = 0
            self.rd = 0
            self.sa = 0

    def getFmtStr(self, fmt: int) -> str:
        fmt_low = fmt & 0x07
        if fmt_low == 0b000:
            return "S"
        elif fmt_low == 0b001:
            return "D"
        elif fmt_low == 0b100:
            return "W"
        elif fmt_low == 0b101:
            return "L"
        return toHex(fmt, 2)

    def getOpcodeName(self) -> str:
        if self.fmt in InstructionCoprocessor1.Cop1Opcodes_ByFormat:
            return InstructionCoprocessor1.Cop1Opcodes_ByFormat[self.fmt]

        if self.fmt == 0b01_000: # fmt = BC
            tf = (self.instr >> 16) & 0x01
            nd = (self.instr >> 17) & 0x01
            opcodeName = "BC1"
            if tf: # Branch on FP True
                opcodeName += "T"
            else: # Branch on FP False
                opcodeName += "F"
            if nd: # Likely
                opcodeName += "L"
            return opcodeName

        fmt = self.getFmtStr(self.fmt)

        if self.function in InstructionCoprocessor1.Cop1Opcodes_ByFunction:
            opcode = InstructionCoprocessor1.Cop1Opcodes_ByFunction[self.function]
            return f"{opcode}.{fmt}"

        if self.fc == 0b11:
            cond = InstructionCoprocessor1.CompareConditionsCodes[self.cond]
            return f"C.{cond}.{fmt}"

        if self.fc == 0b10:
            dst_fmt = self.getFmtStr(self.function)
            return f"CVT.{dst_fmt}.{fmt}"

        function = toHex(self.function, 2)
        return f"COP1.{fmt}({function})"


    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName().lower().ljust(self.ljustWidthOpcode, ' ')
        rt = self.getRegisterName(self.rt)
        ft = self.getFloatRegisterName(self.ft)
        fs = self.getFloatRegisterName(self.fs)
        fd = self.getFloatRegisterName(self.fd)
        #immediate = toHex(self.immediate, 4)
        immediate = hex(self.immediate)
        if immOverride is not None:
            immediate = immOverride

        if self.fmt in InstructionCoprocessor1.Cop1Opcodes_ByFormat:
            result = f"{opcode} {rt},"
            result = result.ljust(14, ' ')
            result += f" {fs}"
            return result

        if self.isBranch():
            result = opcode
            return f"{result} {immediate}"

        if self.function in InstructionCoprocessor1.Cop1Opcodes_ByFunction:
            result = f"{opcode} {fd},"
            result = result.ljust(14, ' ')
            result += f" {fs}"
            if self.isBinaryOperation():
                result += ","
                result.ljust(19, ' ')
                result += f" {ft}"
            return result

        if self.fc == 0b11:
            result = f"{opcode} {fs},"
            result = result.ljust(14, ' ')
            result += f" {ft}"
            return result

        if self.fc == 0b10:
            result = f"{opcode} {fd},"
            result = result.ljust(14, ' ')
            result += f" {fs}"
            return result

        opcode = "COP1".lower().ljust(self.ljustWidthOpcode, ' ')
        instr_index = toHex(self.instr_index, 7)
        return f"{opcode} {instr_index}"
