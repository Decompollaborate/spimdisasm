#!/usr/bin/python3

from __future__ import annotations

from .MipsConstants import InstructionId

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionCoprocessor1(InstructionBase):
    Cop1Opcodes_ByFormat = {
        0b00_000: InstructionId.MFC1,
        0b00_001: InstructionId.DMFC1,
        0b00_010: InstructionId.CFC1,

        0b00_100: InstructionId.MTC1,
        0b00_101: InstructionId.DMTC1,
        0b00_110: InstructionId.CTC1,
    }
    Cop1Opcodes_ByFunction = {
        0b000_000: { 0: InstructionId.ADD_S, 1: InstructionId.ADD_D },
        0b000_001: { 0: InstructionId.SUB_S, 1: InstructionId.SUB_D },
        0b000_010: { 0: InstructionId.MUL_S, 1: InstructionId.MUL_D },
        0b000_011: { 0: InstructionId.DIV_S, 1: InstructionId.DIV_D },

        0b000_100: { 0: InstructionId.SQRT_S, 1: InstructionId.SQRT_D },
        0b000_101: { 0: InstructionId.ABS_S,  1: InstructionId.ABS_D },
        0b000_110: { 0: InstructionId.MOV_S,  1: InstructionId.MOV_D },
        0b000_111: { 0: InstructionId.NEG_S,  1: InstructionId.NEG_D },

        0b001_000: { 0: InstructionId.ROUND_L_S, 1: InstructionId.ROUND_L_D },
        0b001_001: { 0: InstructionId.TRUNC_L_S, 1: InstructionId.TRUNC_L_D },
        0b001_010: { 0: InstructionId.CEIL_L_S, 1: InstructionId.CEIL_L_D },
        0b001_011: { 0: InstructionId.FLOOR_L_S, 1: InstructionId.FLOOR_L_D },

        0b001_100: { 0: InstructionId.ROUND_W_S, 1: InstructionId.ROUND_W_D },
        0b001_101: { 0: InstructionId.TRUNC_W_S, 1: InstructionId.TRUNC_W_D },
        0b001_110: { 0: InstructionId.CEIL_W_S, 1: InstructionId.CEIL_W_D },
        0b001_111: { 0: InstructionId.FLOOR_W_S, 1: InstructionId.FLOOR_W_D },
    }
    CompareConditionsCodes = {
        0b0_000: { 0: InstructionId.C_F_S,    1: InstructionId.C_F_D }, # False
        0b0_001: { 0: InstructionId.C_UN_S,   1: InstructionId.C_UN_D }, # UNordered
        0b0_010: { 0: InstructionId.C_EQ_S,   1: InstructionId.C_EQ_D }, # EQual
        0b0_011: { 0: InstructionId.C_UEQ_S,  1: InstructionId.C_UEQ_D }, # Unordered or EQual
        0b0_100: { 0: InstructionId.C_OLT_S,  1: InstructionId.C_OLT_D }, # Ordered or Less Than
        0b0_101: { 0: InstructionId.C_ULT_S,  1: InstructionId.C_ULT_D }, # Unordered or Less Than
        0b0_110: { 0: InstructionId.C_OLE_S,  1: InstructionId.C_OLE_D }, # Ordered or Less than or Equal
        0b0_111: { 0: InstructionId.C_ULE_S,  1: InstructionId.C_ULE_D }, # Unordered or Less than or Equal

        0b1_000: { 0: InstructionId.C_SF_S,   1: InstructionId.C_SF_D }, # Signaling False
        0b1_001: { 0: InstructionId.C_NGLE_S, 1: InstructionId.C_NGLE_D }, # Not Greater than or Less than or Equal
        0b1_010: { 0: InstructionId.C_SEQ_S,  1: InstructionId.C_SEQ_D }, # Signaling Equal
        0b1_011: { 0: InstructionId.C_NGL_S,  1: InstructionId.C_NGL_D }, # Not Greater than or Less than
        0b1_100: { 0: InstructionId.C_LT_S,   1: InstructionId.C_LT_D }, # Less than
        0b1_101: { 0: InstructionId.C_NGE_S,  1: InstructionId.C_NGE_D }, # Not Greater than or Equal
        0b1_110: { 0: InstructionId.C_LE_S,   1: InstructionId.C_LE_D }, # Less than or Equal
        0b1_111: { 0: InstructionId.C_NGT_S,  1: InstructionId.C_NGT_D }, # Not Greater than
    }
    ConvertCodes = {
        0b000: { 0b001: InstructionId.CVT_S_D, 0b100: InstructionId.CVT_S_W, 0b101: InstructionId.CVT_S_L },
        0b001: { 0b000: InstructionId.CVT_D_S, 0b100: InstructionId.CVT_D_W, 0b101: InstructionId.CVT_D_L },
        0b100: { 0b000: InstructionId.CVT_W_S, 0b001: InstructionId.CVT_W_D, },
        0b101: { 0b000: InstructionId.CVT_L_S, 0b001: InstructionId.CVT_L_D, },
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        # self.opcodesDict = 
        self.processUniqueId()


    def processUniqueId(self):
        super().processUniqueId()

        if self.fmt in InstructionCoprocessor1.Cop1Opcodes_ByFormat:
            self.uniqueId = InstructionCoprocessor1.Cop1Opcodes_ByFormat[self.fmt]

        elif self.fmt == 0b01_000: # fmt = BC
            tf = (self.instr >> 16) & 0x01
            nd = (self.instr >> 17) & 0x01
            if tf:
                if nd:
                    self.uniqueId = InstructionId.BC1TL
                else:
                    self.uniqueId = InstructionId.BC1T
            else:
                if nd:
                    self.uniqueId = InstructionId.BC1FL
                else:
                    self.uniqueId = InstructionId.BC1F

        elif self.function in InstructionCoprocessor1.Cop1Opcodes_ByFunction:
            perFmt = InstructionCoprocessor1.Cop1Opcodes_ByFunction[self.function]
            fmt = self.fmt & 0x07
            if fmt in perFmt:
                self.uniqueId = perFmt[fmt]

        elif self.fc == 0b11:
            if self.cond in InstructionCoprocessor1.CompareConditionsCodes:
                perFmt = InstructionCoprocessor1.CompareConditionsCodes[self.cond]
                fmt = self.fmt & 0x07
                if fmt in perFmt:
                    self.uniqueId = perFmt[fmt]

        elif self.fc == 0b10:
            fun = self.function & 0x07
            if fun in InstructionCoprocessor1.ConvertCodes:
                perFmt = InstructionCoprocessor1.ConvertCodes[fun]
                fmt = self.fmt & 0x07
                if fmt in perFmt:
                    self.uniqueId = perFmt[fmt]

    def isFloatInstruction(self) -> bool:
        return True

    def isBranch(self) -> bool:
        if self.uniqueId in (InstructionId.BC1T, InstructionId.BC1TL, InstructionId.BC1F, InstructionId.BC1FL):
            return True
        return False
    def isBranchLikely(self) -> bool:
        if self.uniqueId in (InstructionId.BC1TL, InstructionId.BC1FL):
            return True
        return False

    def isBinaryOperation(self) -> bool:
        if self.uniqueId in (InstructionId.ADD_S, InstructionId.ADD_D, InstructionId.SUB_S, InstructionId.SUB_D,
                             InstructionId.MUL_S, InstructionId.MUL_D, InstructionId.DIV_S, InstructionId.DIV_D):
            return True
        return False


    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        if self.uniqueId in (InstructionId.MFC1, InstructionId.DMFC1, InstructionId.CFC1):
            return True
        # TODO
        return super().modifiesRt()
    def modifiesRd(self) -> bool:
        # modifying fs shouldn't be the same as modifying rd
        #if self.uniqueId in (InstructionId.MTC1, InstructionId.DMTC1, InstructionId.CTC1):
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

    def getOpcodeName(self) -> str:
        if self.uniqueId == InstructionId.INVALID:
            opcode = toHex(self.function, 2)
            return f"COP1({opcode})"
        return super().getOpcodeName().replace("_", ".")


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
