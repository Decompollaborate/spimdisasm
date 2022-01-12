#!/usr/bin/python3

from __future__ import annotations

from .MipsConstants import InstructionId

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionSpecial(InstructionBase):
    SpecialOpcodes: Dict[int, InstructionId] = {
        0b000_000: InstructionId.SLL,
        # 0b000_001: "MOVCI", # TODO
        0b000_010: InstructionId.SRL,
        0b000_011: InstructionId.SRA,
        0b000_100: InstructionId.SLLV,
        # 0b000_101: "",
        0b000_110: InstructionId.SRLV,
        0b000_111: InstructionId.SRAV,

        0b001_000: InstructionId.JR,
        0b001_001: InstructionId.JALR,
        0b001_010: InstructionId.MOVZ,
        0b001_011: InstructionId.MOVN,
        0b001_100: InstructionId.SYSCALL,
        0b001_101: InstructionId.BREAK,
        # 0b001_110: "",
        0b001_111: InstructionId.SYNC,

        0b010_000: InstructionId.MFHI,
        0b010_001: InstructionId.MTHI,
        0b010_010: InstructionId.MFLO,
        0b010_011: InstructionId.MTLO,
        0b010_100: InstructionId.DSLLV,
        # 0b010_101: "",
        0b010_110: InstructionId.DSRLV,
        0b010_111: InstructionId.DSRAV,

        0b011_000: InstructionId.MULT,
        0b011_001: InstructionId.MULTU,
        0b011_010: InstructionId.DIV,
        0b011_011: InstructionId.DIVU,
        0b011_100: InstructionId.DMULT,
        0b011_101: InstructionId.DMULTU,
        0b011_110: InstructionId.DDIV,
        0b011_111: InstructionId.DDIVU,

        0b100_000: InstructionId.ADD,
        0b100_001: InstructionId.ADDU,
        0b100_010: InstructionId.SUB,
        0b100_011: InstructionId.SUBU,
        0b100_100: InstructionId.AND,
        0b100_101: InstructionId.OR,
        0b100_110: InstructionId.XOR,
        0b100_111: InstructionId.NOR,

        # 0b101_000: "",
        # 0b101_001: "",
        0b101_010: InstructionId.SLT,
        0b101_011: InstructionId.SLTU,
        0b101_100: InstructionId.DADD,
        0b101_101: InstructionId.DADDU,
        0b101_110: InstructionId.DSUB,
        0b101_111: InstructionId.DSUBU,

        0b110_000: InstructionId.TGE,
        0b110_001: InstructionId.TGEU,
        0b110_010: InstructionId.TLT,
        0b110_011: InstructionId.TLTU,
        0b110_100: InstructionId.TEQ,
        # 0b110_101: "",
        0b110_110: InstructionId.TNE,
        # 0b110_111: "",

        0b111_000: InstructionId.DSLL,
        # 0b111_001: "",
        0b111_010: InstructionId.DSRL,
        0b111_011: InstructionId.DSRA,
        0b111_100: InstructionId.DSLL32,
        # 0b111_101: "",
        0b111_110: InstructionId.DSRL32,
        0b111_111: InstructionId.DSRA32,
    }

    def __init__(self, instr: int):
        super().__init__(instr)

        self.opcodesDict = dict(InstructionSpecial.SpecialOpcodes)
        self.processUniqueId()


    def processUniqueId(self):
        super().processUniqueId()

        self.uniqueId = self.opcodesDict.get(self.function, InstructionId.INVALID)
        if self.instr == 0:
            self.uniqueId = InstructionId.NOP
        elif self.rt == 0:
            if self.uniqueId == InstructionId.OR:
                self.uniqueId = InstructionId.MOVE
            elif self.uniqueId == InstructionId.NOR:
                self.uniqueId = InstructionId.NOT
        elif self.uniqueId == InstructionId.SUBU:
            if self.rs == 0:
                self.uniqueId = InstructionId.NEGU


    def isBranch(self) -> bool:
        return False
    def isTrap(self) -> bool:
        if self.uniqueId in (InstructionId.TGE, InstructionId.TGEU, InstructionId.TLT, InstructionId.TLTU,
                             InstructionId.TEQ, InstructionId.TNE):
            return True
        return False


    def isRType(self) -> bool: # OP rd, rs, rt
        if self.isRType2():
            return False
        elif self.isSaType():
            return False
        elif self.instr == 0x0:
            return False
        return True # Not for all cases, but good enough
    def isRType2(self) -> bool: # OP rd, rt, rs
        if self.uniqueId in (InstructionId.DSLLV, InstructionId.DSRLV, InstructionId.DSRAV, InstructionId.SLLV, 
                             InstructionId.SRLV, InstructionId.SRAV):
            return True
        return False
    def isSaType(self) -> bool: # OP rd, rt, sa
        if self.uniqueId in (InstructionId.SLL, InstructionId.SRL, InstructionId.SRA, InstructionId.DSLL,
                             InstructionId.DSRL, InstructionId.DSRA, InstructionId.DSLL32, InstructionId.DSRL32,
                             InstructionId.DSRA32):
            return True
        return False


    def modifiesRt(self) -> bool:
        return False
    def modifiesRd(self) -> bool:
        if self.uniqueId in (InstructionId.JR, InstructionId.JALR, InstructionId.MTHI, InstructionId.MTLO,
                             InstructionId.MULT, InstructionId.MULTU, InstructionId.DIV, InstructionId.DIVU,
                             InstructionId.DMULT, InstructionId.DMULTU, InstructionId.DDIV, InstructionId.DDIVU,
                             InstructionId.SYSCALL, InstructionId.BREAK, InstructionId.SYNC): # TODO
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
        if self.uniqueId == InstructionId.INVALID:
            opcode = toHex(self.function, 2)
            return f"Special({opcode})"
        return super().getOpcodeName()


    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        rd = self.getRegisterName(self.rd)

        if self.uniqueId == InstructionId.NOP:
            return "nop"

        elif self.uniqueId in (InstructionId.MOVE, InstructionId.NOT): # OP rd, rs
            result = f"{formated_opcode} {rd},"
            result = result.ljust(14, ' ')
            return f"{result} {rs}"

        elif self.uniqueId in (InstructionId.JR, InstructionId.MTHI, InstructionId.MTLO):
            result = f"{formated_opcode} {rs}"
            return result
        elif self.uniqueId == InstructionId.JALR:
            if self.rd == 31:
                rd = ""
            else:
                rd += ","
                rd = rd.ljust(6, ' ')
            result = f"{formated_opcode} {rd}{rs}"
            return result
        elif self.uniqueId in (InstructionId.MFHI, InstructionId.MFLO):
            return f"{formated_opcode} {rd}"
        elif self.uniqueId in (InstructionId.MULT, InstructionId.MULTU, InstructionId.DMULT, InstructionId.DMULTU) or self.isTrap(): # OP  rs, rt
            result = f"{formated_opcode} {rs},".ljust(14, ' ')
            return f"{result} {rt}"
        elif self.uniqueId in (InstructionId.SYSCALL, InstructionId.BREAK, InstructionId.SYNC):
            code = (self.instr_index) >> 16
            result = f"{formated_opcode} {code}"
            return result

        elif self.uniqueId in (InstructionId.NEGU,):
            result = f"{formated_opcode} {rd},"
            result = result.ljust(14, ' ')
            return f"{result} {rt}"

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

