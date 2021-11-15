#!/usr/bin/python3

from __future__ import annotations

from ..Utils import *

from .MipsInstructionBase import InstructionBase
from ..MipsContext import Context


class InstructionCoprocessor0(InstructionBase):
    Cop0Opcodes_ByFormat = {
        0b00_000: "MFC0", # Move word From CP0
        0b00_001: "DMFC0", # Doubleword Move From CP0
        0b00_010: "CFC0", # Move control word From CP0
        # 0b00_011: "",
        0b00_100: "MTC0", # Move word to CP0
        0b00_101: "DMTC0", # Doubleword Move To CP0
        0b00_110: "CTC0", # Move control word To CP0
        # 0b00_111: "",
    }
    Cop0Opcodes_ByFunction = {
        0b000_001: "TLBR", # Read Indexed TLB Entry
        0b000_010: "TLBWI", # Write Indexed TLB Entry
        0b000_110: "TLBWR", # Write Random TLB Entry
        0b001_000: "TLBP", # Probe TLB for Matching Entry
        0b011_000: "ERET", # Return from Exception
    }

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

    def isImplemented(self) -> bool:
        if self.fmt in InstructionCoprocessor0.Cop0Opcodes_ByFormat:
            return True
        if self.fmt == 0b01_000:
            return True
        if self.fmt == 0b10_000:
            if self.function in InstructionCoprocessor0.Cop0Opcodes_ByFunction:
                return True
        return False

    def isBranch(self) -> bool:
        opcode = self.getOpcodeName()
        if opcode in ("BC0T", "BC0TL", "BC0F", "BC0FL"):
            return True
        return False


    def sameOpcode(self, other: InstructionBase) -> bool:
        if self.opcode != other.opcode:
            return False

        if self.fmt == other.fmt:
            if self.fmt in InstructionCoprocessor0.Cop0Opcodes_ByFormat:
                return True
            if self.fmt == 0b01_000:
                if self.tf == other.tf and self.nd == other.nd:
                    return True
                return False

            return self.function == other.function

        return False


    def modifiesRt(self) -> bool:
        if self.isBranch():
            return False
        opcode = self.getOpcodeName()
        if opcode in ("MFC0", "DMFC0", "CFC0"):
            return True
        # TODO
        return super().modifiesRt()
    def modifiesRd(self) -> bool:
        opcode = self.getOpcodeName()
        # modifying fs shouldn't be the same as modifying rd
        #if opcode in ("MTC0", "DMTC0", "CTC0"):
        #    return True
        # TODO
        return super().modifiesRd()


    def blankOut(self):
        if self.fmt in InstructionCoprocessor0.Cop0Opcodes_ByFormat:
            self.rt = 0
            self.rd = 0
            self.sa = 0
            self.function = 0
        elif self.fmt == 0b01_000:
            self.rd = 0
            self.sa = 0
            self.function = 0
        elif self.function in InstructionCoprocessor0.Cop0Opcodes_ByFunction:
            self.rt = 0
            self.rd = 0
            self.sa = 0

    def getOpcodeName(self) -> str:
        if self.fmt in InstructionCoprocessor0.Cop0Opcodes_ByFormat:
            return InstructionCoprocessor0.Cop0Opcodes_ByFormat[self.fmt]

        if self.fmt == 0b01_000: # fmt = BC
            opcodeName = "BC0"
            if self.tf: # Branch on FP True
                opcodeName += "T"
            else: # Branch on FP False
                opcodeName += "F"
            if self.nd: # Likely
                opcodeName += "L"
            return opcodeName

        if self.function in InstructionCoprocessor0.Cop0Opcodes_ByFunction:
            return InstructionCoprocessor0.Cop0Opcodes_ByFunction[self.function]

        fmt = toHex(self.fmt, 2)
        function = toHex(self.function, 2)
        return f"COP0.{fmt}({function})"

    def getCop0RegisterName(self, register: int) -> str:
        if register in InstructionCoprocessor0.Cop0RegisterNames:
            return InstructionCoprocessor0.Cop0RegisterNames[register]
        return hex(register)


    def disassemble(self, context: Context|None, immOverride: str|None=None) -> str:
        opcode = self.getOpcodeName()
        formated_opcode = opcode.lower().ljust(self.ljustWidthOpcode, ' ')
        rt = self.getRegisterName(self.rt)
        rd = self.getCop0RegisterName(self.rd)
        #immediate = toHex(self.immediate, 4)
        immediate = hex(self.immediate)
        if immOverride is not None:
            immediate = immOverride

        if self.fmt in InstructionCoprocessor0.Cop0Opcodes_ByFormat:
            result = f"{formated_opcode} {rt},"
            result = result.ljust(14, ' ')
            result += f" {rd}"
            return result

        if self.isBranch():
            result = formated_opcode
            return f"{result} {immediate}"

        if self.function in InstructionCoprocessor0.Cop0Opcodes_ByFunction:
            result = f"{opcode.lower()}"
            return result

        instr_index = toHex(self.instr_index, 7)
        return f"{formated_opcode} {instr_index}"
