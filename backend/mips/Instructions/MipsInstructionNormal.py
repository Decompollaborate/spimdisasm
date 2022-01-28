#!/usr/bin/python3

from __future__ import annotations

from ...common.Utils import *

from .MipsConstants import InstructionId
from .MipsInstructionBase import InstructionBase


class InstructionNormal(InstructionBase):
    NormalOpcodes: Dict[int, InstructionId] = {
        # 0b000_000: "SPECIAL",
        # 0b000_001: "REGIMM",
        0b000_010: InstructionId.J,
        0b000_011: InstructionId.JAL,
        0b000_100: InstructionId.BEQ,
        0b000_101: InstructionId.BNE,
        0b000_110: InstructionId.BLEZ,
        0b000_111: InstructionId.BGTZ,

        0b001_000: InstructionId.ADDI,
        0b001_001: InstructionId.ADDIU,
        0b001_010: InstructionId.SLTI,
        0b001_011: InstructionId.SLTIU,
        0b001_100: InstructionId.ANDI,
        0b001_101: InstructionId.ORI,
        0b001_110: InstructionId.XORI,
        0b001_111: InstructionId.LUI,

        # 0b010_000: "COP0", # Coprocessor OPeration z
        # 0b010_001: "COP1", # Coprocessor OPeration z
        # 0b010_010: "COP2", # Coprocessor OPeration z
        # 0b010_011: "COP3", # Coprocessor OPeration z
        0b010_100: InstructionId.BEQL,
        0b010_101: InstructionId.BNEL,
        0b010_110: InstructionId.BLEZL,
        0b010_111: InstructionId.BGTZL,

        0b011_000: InstructionId.DADDI,
        0b011_001: InstructionId.DADDIU,
        0b011_010: InstructionId.LDL,
        0b011_011: InstructionId.LDR,
        # 0b011_100: "",
        # 0b011_101: "",
        # 0b011_110: "",
        # 0b011_111: "",

        0b100_000: InstructionId.LB,
        0b100_001: InstructionId.LH,
        0b100_010: InstructionId.LWL,
        0b100_011: InstructionId.LW,
        0b100_100: InstructionId.LBU,
        0b100_101: InstructionId.LHU,
        0b100_110: InstructionId.LWR,
        0b100_111: InstructionId.LWU,

        0b101_000: InstructionId.SB,
        0b101_001: InstructionId.SH,
        0b101_010: InstructionId.SWL,
        0b101_011: InstructionId.SW,
        0b101_100: InstructionId.SDL,
        0b101_101: InstructionId.SDR,
        0b101_110: InstructionId.SWR,
        0b101_111: InstructionId.CACHE,

        0b110_000: InstructionId.LL,
        0b110_001: InstructionId.LWC1,
        0b110_010: InstructionId.LWC2,
        0b110_011: InstructionId.PREF,
        0b110_100: InstructionId.LLD,
        0b110_101: InstructionId.LDC1,
        0b110_110: InstructionId.LDC2,
        0b110_111: InstructionId.LD,

        0b111_000: InstructionId.SC,
        0b111_001: InstructionId.SWC1,
        0b111_010: InstructionId.SWC2,
        # 0b111_011: "",
        0b111_100: InstructionId.SCD,
        0b111_101: InstructionId.SDC1,
        0b111_110: InstructionId.SDC2,
        0b111_111: InstructionId.SD,
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        self.opcodesDict = dict(InstructionNormal.NormalOpcodes)
        self.processUniqueId()


    def processUniqueId(self):
        super().processUniqueId()

        self.uniqueId = self.opcodesDict.get(self.opcode, InstructionId.INVALID)
        if self.rt == 0:
            if self.uniqueId == InstructionId.BEQ:
                if self.rs == 0:
                    self.uniqueId = InstructionId.B
                else:
                    self.uniqueId = InstructionId.BEQZ
            elif self.uniqueId == InstructionId.BNE:
                self.uniqueId = InstructionId.BNEZ

    def isFloatInstruction(self) -> bool:
        if self.isDoubleFloatInstruction():
            return True
        if self.uniqueId in (InstructionId.LWC1, InstructionId.SWC1):
            return True
        return False

    def isDoubleFloatInstruction(self) -> bool:
        if self.uniqueId in (InstructionId.LDC1, InstructionId.SDC1):
            return True
        return False


    def isBranch(self) -> bool:
        if self.uniqueId in (InstructionId.BEQ, InstructionId.BEQL, InstructionId.BLEZ, InstructionId.BLEZL,
                             InstructionId.BNE, InstructionId.BNEL, InstructionId.BGTZ, InstructionId.BGTZL,
                             InstructionId.BEQZ, InstructionId.BNEZ, InstructionId.B):
            return True
        return super().isBranch()
    def isBranchLikely(self) -> bool:
        if self.uniqueId in (InstructionId.BEQL, InstructionId.BLEZL, InstructionId.BNEL, InstructionId.BGTZL):
            return True
        return False

    # OP LABEL
    def isJType(self) -> bool:
        if self.uniqueId in (InstructionId.J, InstructionId.JAL):
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
        if self.uniqueId in (InstructionId.BLEZ, InstructionId.BGTZ, InstructionId.BLEZL, InstructionId.BGTZL,
                             InstructionId.BEQZ, InstructionId.BNEZ):
            return True
        return False

    # OP  rs, rt, IMM
    def isBinaryBranch(self) -> bool:
        if self.uniqueId in (InstructionId.BEQ, InstructionId.BEQL, InstructionId.BNE, InstructionId.BNEL):
            return True
        return False

    # OP  rt, IMM
    def isUnaryOperation(self) -> bool:
        if self.uniqueId in (InstructionId.LUI, ):
            return True
        return False

    # OP  rt, rs, IMM
    def isBinaryOperation(self) -> bool:
        if self.uniqueId in (InstructionId.ADDI, InstructionId.ADDIU, InstructionId.ANDI, InstructionId.DADDI,
                             InstructionId.DADDIU, InstructionId.ORI, InstructionId.XORI, InstructionId.SLTI,
                             InstructionId.SLTIU):
            return True
        return False

    def isOperation(self) -> bool:
        return self.isBinaryOperation() or self.isUnaryOperation()

    def isUnsigned(self) -> bool:
        if self.uniqueId in (InstructionId.LUI, InstructionId.ANDI, InstructionId.ORI, InstructionId.XORI):
            return True
        return False


    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        if self.isJType():
            return False

        if self.uniqueId in (InstructionId.SB, InstructionId.SH, InstructionId.SWL, InstructionId.SW, 
                             InstructionId.SDL, InstructionId.SDR, InstructionId.SWR):
            return False

        # Changes the value of the coprocessor's register
        if self.uniqueId in (InstructionId.LWC1, InstructionId.LWC2, InstructionId.LDC1, InstructionId.LDC2):
            return False

        if self.uniqueId in (InstructionId.SWC1, InstructionId.SWC2, InstructionId.SDC1, InstructionId.SDC2):
            return False
        return super().modifiesRt()

    def getOpcodeName(self) -> str:
        if self.uniqueId == InstructionId.INVALID:
            opcode = toHex(self.opcode, 2)
            return f"Unknown({opcode})"
        return super().getOpcodeName()


    def disassemble(self, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        immediate = hex(self.immediate)
        if not self.isUnsigned():
            immediate = hex(from2Complement(self.immediate, 16))
        if immOverride is not None:
            immediate = immOverride

        result = f"{formated_opcode} "

        if self.isJType():
            vram = self.getInstrIndexAsVram()
            label = f"func_{vram:06X}"
            if immOverride is not None:
                label = immOverride
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
            # OP  IMM
            else:
                result = f"{formated_opcode}"
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
            result += self.getFloatRegisterName(self.rt)
        elif self.uniqueId == InstructionId.CACHE:
            result += toHex(self.rt, 2)
        elif self.uniqueId in (InstructionId.LWC2, InstructionId.SWC2, InstructionId.LDC2, InstructionId.SDC2):
            result += self.getCop2RegisterName(self.rt)
        else:
            result += rt

        result += ","
        result = result.ljust(14, ' ')
        return f"{result} {immediate}({rs})"
