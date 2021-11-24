#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *
from ..MipsContext import Context
from .MipsConstants import InstructionId

class InstructionBase:
    Cop0RegisterNames = {
        0: "Index",
        1: "Random",
        2: "EntryLo0",
        3: "EntryLo1",
        4: "Context",
        5: "PageMask",
        6: "Wired",
        7: "Reserved07",
        8: "BadVaddr",
        9: "Count",
        10: "EntryHi",
        11: "Compare",
        12: "Status",
        13: "Cause",
        14: "EPC",
        15: "PRevID",
        16: "Config",
        17: "LLAddr",
        18: "WatchLo",
        19: "WatchHi",
        20: "XContext",
        21: "Reserved21",
        22: "Reserved22",
        23: "Reserved23",
        24: "Reserved24",
        25: "Reserved25",
        26: "PErr",
        27: "CacheErr",
        28: "TagLo",
        29: "TagHi",
        30: "ErrorEPC",
        31: "Reserved31",
    }

    Cop2RegisterNames = {
        0: "$0",
        1: "$1",
        2: "$2",
        3: "$3",
        4: "$4",
        5: "$5",
        6: "$6",
        7: "$7",
        8: "$8",
        9: "$9",
        10: "$10",
        11: "$11",
        12: "$12",
        13: "$13",
        14: "$14",
        15: "$15",
        16: "$16",
        17: "$17",
        18: "$18",
        19: "$19",
        20: "$20",
        21: "$21",
        22: "$22",
        23: "$23",
        24: "$24",
        25: "$25",
        26: "$26",
        27: "$27",
        28: "$28",
        29: "$29",
        30: "$30",
        31: "$31",
    }

    def __init__(self, instr: int):
        self.opcode = (instr >> 26) & 0x3F
        self.rs = (instr >> 21) & 0x1F # rs
        self.rt = (instr >> 16) & 0x1F # usually the destiny of the operation
        self.rd = (instr >> 11) & 0x1F # destination register in R-Type instructions
        self.sa = (instr >>  6) & 0x1F
        self.function = (instr >> 0) & 0x3F

        self.uniqueId: InstructionId = InstructionId.INVALID

        self.ljustWidthOpcode = 7+4

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

    @property
    def fmt(self) -> int:
        return self.rs

    @property
    def ft(self) -> int:
        return self.rt
    @property
    def fs(self) -> int:
        return self.rd
    @property
    def fd(self) -> int:
        return self.sa

    @property
    def nd(self) -> int:
        return (self.rt >> 0) & 0x01
    @property
    def tf(self) -> int:
        return (self.rt >> 1) & 0x01
    @property
    def fc(self) -> int:
        return (self.function >> 4) & 0x03
    @property
    def cond(self) -> int:
        return (self.function >> 0) & 0x0F


    def isImplemented(self) -> bool:
        return self.uniqueId != InstructionId.INVALID

    def isFloatInstruction(self) -> bool:
        return False

    def isDoubleFloatInstruction(self) -> bool:
        return False


    def isBranch(self) -> bool:
        return False
    def isBranchLikely(self) -> bool:
        return False
    def isTrap(self) -> bool:
        return False

    def isJType(self) -> bool:
        return False

    def isIType(self) -> bool:
        return False


    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.uniqueId == InstructionId.INVALID:
            return False
        if other.uniqueId == InstructionId.INVALID:
            return False
        return self.uniqueId == other.uniqueId

    def sameBaseRegister(self, other: InstructionBase):
        return self.baseRegister == other.baseRegister

    def sameOpcodeButDifferentArguments(self, other: InstructionBase) -> bool:
        if not self.sameOpcode(other):
            return False
        return self.instr != other.instr


    def modifiesRt(self) -> bool:
        if self.isBranch():
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
        if self.uniqueId != InstructionId.INVALID:
            return self.uniqueId.name
        opcode = toHex(self.opcode, 2)
        return f"({opcode})"

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
        return f"${register:02X}"

    def getFloatRegisterName(self, register: int) -> str:
        if register == 31:
            return "$31"
        if 0 <= register <= 31:
            return "$f" + str(register)
        return f"${register:02X}"

    def getCop0RegisterName(self, register: int) -> str:
        if register in InstructionBase.Cop0RegisterNames:
            return InstructionBase.Cop0RegisterNames[register]
        return f"${register:02X}"

    def getCop2RegisterName(self, register: int) -> str:
        if register in InstructionBase.Cop2RegisterNames:
            return InstructionBase.Cop2RegisterNames[register]
        return f"${register:02X}"


    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName().lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        #immediate = toHex(self.immediate, 4)
        immediate = hex(self.immediate)
        if immOverride is not None:
            immediate = immOverride

        return f"ERROR # {opcode} {rs} {rt} {immediate}"


    def __str__(self) -> str:
        return self.disassemble(None)

    def __repr__(self) -> str:
        return self.__str__()
