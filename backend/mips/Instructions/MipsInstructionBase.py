#!/usr/bin/python3

from __future__ import annotations

from ...common.Utils import *

from .MipsConstants import InstructionId, InstructionVectorId

class InstructionBase:
    GprRegisterNames = {
        0:  "$zero",
        1:  "$at",
        2:  "$v0",
        3:  "$v1",
        4:  "$a0",
        5:  "$a1",
        6:  "$a2",
        7:  "$a3",
        8:  "$t0",
        9:  "$t1",
        10: "$t2",
        11: "$t3",
        12: "$t4",
        13: "$t5",
        14: "$t6",
        15: "$t7",
        16: "$s0",
        17: "$s1",
        18: "$s2",
        19: "$s3",
        20: "$s4",
        21: "$s5",
        22: "$s6",
        23: "$s7",
        24: "$t8",
        25: "$t9",
        26: "$k0",
        27: "$k1",
        28: "$gp",
        29: "$sp",
        30: "$fp",
        31: "$ra",
    }

    Cop0RegisterNames = {
        0:  "Index",
        1:  "Random",
        2:  "EntryLo0",
        3:  "EntryLo1",
        4:  "Context",
        5:  "PageMask",
        6:  "Wired",
        7:  "Reserved07",
        8:  "BadVaddr",
        9:  "Count",
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

    # Float registers
    Cop1RegisterNames = {
        0:  "$f0",
        1:  "$f1",
        2:  "$f2",
        3:  "$f3",
        4:  "$f4",
        5:  "$f5",
        6:  "$f6",
        7:  "$f7",
        8:  "$f8",
        9:  "$f9",
        10: "$f10",
        11: "$f11",
        12: "$f12",
        13: "$f13",
        14: "$f14",
        15: "$f15",
        16: "$f16",
        17: "$f17",
        18: "$f18",
        19: "$f19",
        20: "$f20",
        21: "$f21",
        22: "$f22",
        23: "$f23",
        24: "$f24",
        25: "$f25",
        26: "$f26",
        27: "$f27",
        28: "$f28",
        29: "$f29",
        30: "$f30",
        31: "FpcCsr",
    }

    Cop2RegisterNames = {
        0:  "$0",
        1:  "$1",
        2:  "$2",
        3:  "$3",
        4:  "$4",
        5:  "$5",
        6:  "$6",
        7:  "$7",
        8:  "$8",
        9:  "$9",
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

    GprRspRegisterNames = {
        0:  "$zero",
        1:  "$1",
        2:  "$2",
        3:  "$3",
        4:  "$4",
        5:  "$5",
        6:  "$6",
        7:  "$7",
        8:  "$8",
        9:  "$9",
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

    Cop0RspRegisterNames = {
        0:  "SP_MEM_ADDR",
        1:  "SP_DRAM_ADDR",
        2:  "SP_RD_LEN",
        3:  "SP_WR_LEN",
        4:  "SP_STATUS",
        5:  "SP_DMA_FULL",
        6:  "SP_DMA_BUSY",
        7:  "SP_SEMAPHORE",
        8:  "DPC_START",
        9:  "DPC_END",
        10: "DPC_CURRENT",
        11: "DPC_STATUS",
        12: "DPC_CLOCK",
        13: "DPC_BUFBUSY",
        14: "DPC_PIPEBUSY",
        15: "DPC_TMEM",
    }

    VectorRspRegisterNames = {
        0:  "$v0",
        1:  "$v1",
        2:  "$v2",
        3:  "$v3",
        4:  "$v4",
        5:  "$v5",
        6:  "$v6",
        7:  "$v7",
        8:  "$v8",
        9:  "$v9",
        10: "$v10",
        11: "$v11",
        12: "$v12",
        13: "$v13",
        14: "$v14",
        15: "$v15",
        16: "$v16",
        17: "$v17",
        18: "$v18",
        19: "$v19",
        20: "$v20",
        21: "$v21",
        22: "$v22",
        23: "$v23",
        24: "$v24",
        25: "$v25",
        26: "$v26",
        27: "$v27",
        28: "$v28",
        29: "$v29",
        30: "$v30",
        31: "$v31",
    }

    def __init__(self, instr: int):
        self.opcode = (instr >> 26) & 0x3F
        self.rs = (instr >> 21) & 0x1F # rs
        self.rt = (instr >> 16) & 0x1F # usually the destiny of the operation
        self.rd = (instr >> 11) & 0x1F # destination register in R-Type instructions
        self.sa = (instr >>  6) & 0x1F
        self.function = (instr >> 0) & 0x3F

        self.opcodesDict: Dict[int, InstructionId | InstructionVectorId] = dict()
        self.uniqueId: InstructionId|InstructionVectorId = InstructionId.INVALID

        self.ljustWidthOpcode = 7+4

        self.isRsp: bool = False

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

    # vector registers
    @property
    def vd(self) -> int:
        return self.sa
    @property
    def vs(self) -> int:
        return self.rd
    @property
    def vt(self) -> int:
        return self.rt
    @property
    def elementHigh(self) -> int:
        return self.rs & 0xF
    @property
    def elementLow(self) -> int:
        return (self.sa >> 1) & 0xF
    @property
    def offsetVector(self) -> int:
        return self.immediate & 0x7F

    def getInstrIndexAsVram(self) -> int:
        vram = self.instr_index<<2
        if not self.isRsp:
            vram |= 0x80000000
        return vram


    def __getitem__(self, key):
        if key < 0 or key > 31:
            raise IndexError()
        return (self.instr >> key) & 0x1


    def processUniqueId(self):
        self.uniqueId = self.opcodesDict.get(self.opcode, InstructionId.INVALID)

    def isImplemented(self) -> bool:
        if self.uniqueId == InstructionId.INVALID:
            return False
        if self.uniqueId == InstructionVectorId.INVALID:
            return False
        return True

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
        if self.uniqueId in (InstructionId.INVALID, InstructionVectorId.INVALID):
            return False
        if other.uniqueId in (InstructionId.INVALID, InstructionVectorId.INVALID):
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
        if self.uniqueId != InstructionId.INVALID and self.uniqueId != InstructionVectorId.INVALID:
            return self.uniqueId.name
        opcode = toHex(self.opcode, 2)
        return f"({opcode})"

    def getRegisterName(self, register: int) -> str:
        return self.GprRegisterNames.get(register, f"${register:02X}")

    def getFloatRegisterName(self, register: int) -> str:
        return self.Cop1RegisterNames.get(register, f"${register:02X}")

    def getCop0RegisterName(self, register: int) -> str:
        if register in InstructionBase.Cop0RegisterNames:
            return InstructionBase.Cop0RegisterNames[register]
        return f"${register:02X}"

    def getCop2RegisterName(self, register: int) -> str:
        if register in InstructionBase.Cop2RegisterNames:
            return InstructionBase.Cop2RegisterNames[register]
        return f"${register:02X}"

    def getGprRspRegisterName(self, register: int) -> str:
        return self.GprRspRegisterNames.get(register, f"${register:02X}")

    def getCop0RspRegisterName(self, register: int) -> str:
        return self.Cop0RspRegisterNames.get(register, f"${register:02X}")

    def getVectorRspRegisterName(self, register: int) -> str:
        return self.VectorRspRegisterNames.get(register, f"${register:02X}")


    def processVectorElement(self, element: int) -> int:
        if (element & 0x8) == 0x8:
            return element & 7
        if (element & 0xC) == 0x4:
            return element & 4
        if (element & 0xE) == 0x2:
            return element & 2
        return element


    def disassemble(self, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName().lower().ljust(self.ljustWidthOpcode, ' ')
        rs = self.getRegisterName(self.rs)
        rt = self.getRegisterName(self.rt)
        #immediate = toHex(self.immediate, 4)
        immediate = hex(self.immediate)
        if immOverride is not None:
            immediate = immOverride

        return f"ERROR # {opcode} {rs} {rt} {immediate}"


    def mapInstrToType(self) -> str|None:
        if self.isFloatInstruction():
            if self.isDoubleFloatInstruction():
                return "f64"
            else:
                return "f32"
        if self.uniqueId in (InstructionId.LW, InstructionId.SW):
            return "s32"
        if self.uniqueId in (InstructionId.LWU, ):
            return "u32"
        if self.uniqueId in (InstructionId.LH, InstructionId.SH):
            return "s16"
        if self.uniqueId in (InstructionId.LHU, ):
            return "u16"
        if self.uniqueId in (InstructionId.LB, InstructionId.SB):
            return "s8"
        if self.uniqueId in (InstructionId.LBU, ):
            return "u8"
        if self.uniqueId in (InstructionId.LD, InstructionId.SD):
            return "s64"
        # if self.uniqueId in (InstructionId.LDU, InstructionId.SDU):
        #     return "u64"
        return None


    def __str__(self) -> str:
        return self.disassemble()

    def __repr__(self) -> str:
        return self.__str__()
