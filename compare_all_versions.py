#!/usr/bin/python3

from __future__ import annotations

import argparse
import os
import hashlib
import struct
from typing import List, Dict
import sys
import subprocess
from multiprocessing import Pool, cpu_count
from functools import partial


versions = {
    "ntsc_1.0_rc" : "NNR",
    "ntsc_1.0" : "NN0",
    "ntsc_1.1" : "NN1",
    "pal_1.0" : "NP0",
    "ntsc_1.2" : "NN2",
    "pal_1.1" : "NP1",
    "jp_gc" : "GJO",
    "jp_mq" : "GJM",
    "usa_gc" : "GUO",
    "usa_mq" : "GUM",
    "pal_gc" : "GPO",
    "pal_gc_dbg" : "GPOD",
    "pal_gc_dbg2" : "GPOD2",
    "pal_mq" : "GPM",
    "pal_mq_dbg" : "GPMD",
    "jp_gc_ce" : "GJC",
    "cn_ique" : "IC",
    "tw_ique" : "IT",
}


# in JAL format. # Real address would be (address << 2)
address_Graph_OpenDisps = {
    "ntsc_1.0_rc" : 0x001F856,
    "ntsc_1.0" : 0x001F8A6,
    "ntsc_1.1" : 0x001F8A6,
    "pal_1.0" : 0x001FA2A,
    "ntsc_1.2" : 0x001FA4A,
    "pal_1.1" : 0x001FA2A,
    "jp_gc" : 0x001F792,
    "jp_mq" : 0x001F792,
    "usa_gc" : 0x001F78A,
    "usa_mq" : 0x001F78A,
    "pal_gc" : 0x001F77E,
    "pal_gc_dbg" : 0x0,
    "pal_gc_dbg2" : 0x0,
    "pal_mq" : 0x001F77E,
    "pal_mq_dbg" : 0x0031AB1,
    "jp_gc_ce" : 0x001F78A,
    "cn_ique" : 0x0,
    "tw_ique" : 0x0,
}

ENTRYPOINT = 0x80000400

ACTOR_ID_MAX = 0x01D7

# The offset of the overlay table in file `code`.
offset_ActorOverlayTable = {
    "ntsc_0.9" : 0x0D7280,
    "ntsc_1.0" : 0x0D7490,
    "ntsc_1.1" : 0x0D7650,
    "pal_1.0" : 0xD4D80,
    "ntsc_1.2" : 0x0D7490,
    "pal_1.1" : 0x0D4DE0,
    "jp_gc" : 0x0D6B60,
    "jp_mq" : 0x0D6B40,
    "usa_gc" : 0x0D6B40,
    "usa_mq" : 0x0D6B20,
    "pal_gc" : 0x0D44A0,
    "pal_gc_dbg" : 0x0F9460,
    "pal_gc_dbg2" : 0x0F9460,
    "pal_mq" : 0x0D4480,
    "pal_mq_dbg" : 0x0F9440,
    "jp_gc_ce" : 0x0D6B40,
    "ique_cn" : 0x0D7180,
    "ique_tw" : 0x0D6AA0,
}

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

# Returns the md5 hash of a bytearray
def getStrHash(byte_array: bytearray) -> str:
    return str(hashlib.md5(byte_array).hexdigest())

def writeBytearrayToFile(filepath: str, array_of_bytes: bytearray):
    with open(filepath, mode="wb") as f:
       f.write(array_of_bytes)

def readFileAsBytearray(filepath: str) -> bytearray:
    if not os.path.exists(filepath):
        return bytearray(0)
    with open(filepath, mode="rb") as f:
        return bytearray(f.read())

def readFile(filepath: str):
    with open(filepath) as f:
        return [x.strip() for x in f.readlines()]

def runCommandGetOutput(command: str, args: List[str]) -> List[str] | None:
    try:
        output = subprocess.check_output([command, *args]).decode("utf-8")
        return output.strip().split("\n")
    except:
        return None

def removeExtraWhitespace(line: str) -> str:
    return" ".join(line.split()) 

def bytesToBEWords(array_of_bytes: bytearray) -> List[int]:
    words = len(array_of_bytes)//4
    big_endian_format = f">{words}I"
    return list(struct.unpack_from(big_endian_format, array_of_bytes, 0))

def beWordsToBytes(words_list: List[int], buffer: bytearray) -> bytearray:
    words = len(words_list)
    big_endian_format = f">{words}I"
    struct.pack_into(big_endian_format, buffer, 0, *words_list)
    return buffer

def toHex(number: int, digits: int) -> str:
    return "0x" + hex(number)[2:].zfill(digits).upper()

def from2Complement(number: int, bits: int) -> int:
    isNegative = number & (1 << (bits - 1))
    if isNegative:
        return -((~number + 1) & ((1 << bits) - 1))
    return number


class File:
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, tableEntry: OverlayTableEntry=None, args=None):
        self.bytes: bytearray = array_of_bytes
        self.words: List[int] = bytesToBEWords(self.bytes)
        self.filename: str = filename
        self.version: str = version
        self.args = args
        self.vRamStart: int = -1
        self.initVarsAddress: int = -1
        if tableEntry is not None:
            self.vRamStart = tableEntry.vramStart
            self.initVarsAddress = tableEntry.initVars

    @property
    def size(self):
        return len(self.bytes)
    @property
    def sizew(self):
        return len(self.words)

    def getHash(self):
        return getStrHash(self.bytes)

    def compareToFile(self, other_file: File):
        hash_one = self.getHash()
        hash_two = other_file.getHash()

        result = {
            "equal": hash_one == hash_two,
            "hash_one": hash_one,
            "hash_two": hash_two,
            "size_one": self.size,
            "size_two": other_file.size,
            "diff_bytes": 0,
            "diff_words": 0,
        }

        if not result["equal"]:
            min_len = min(self.size, other_file.size)
            for i in range(min_len):
                if self.bytes[i] != other_file.bytes[i]:
                    result["diff_bytes"] += 1

            min_len = min(self.sizew, other_file.sizew)
            for i in range(min_len):
                if self.words[i] != other_file.words[i]:
                    result["diff_words"] += 1
                    #if isinstance(self, Text) and isinstance(other_file, Text):
                        #eprint(f"Differing instruction: {self.instructions[i]}")
                        #eprint(f"Differing instruction: {other_file.instructions[i]}")
                        #eprint(f"")
                        #pass

        return result

    def blankOutDifferences(self, other: File):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        was_updated = False
        if self.args is not None and (self.args.ignore80 or self.args.ignore06 or self.args.ignore04):
            min_len = min(self.sizew, other.sizew)
            for i in range(min_len):
                if self.args.ignore80:
                    if ((self.words[i] >> 24) & 0xFF) == 0x80 and ((other.words[i] >> 24) & 0xFF) == 0x80:
                        self.words[i] = 0x80000000
                        other.words[i] = 0x80000000
                        was_updated = True
                if self.args.ignore06:
                    if ((self.words[i] >> 24) & 0xFF) == 0x06 and ((other.words[i] >> 24) & 0xFF) == 0x06:
                        self.words[i] = 0x06000000
                        other.words[i] = 0x06000000
                        was_updated = True
                if self.args.ignore04:
                    if ((self.words[i] >> 24) & 0xFF) == 0x04 and ((other.words[i] >> 24) & 0xFF) == 0x04:
                        self.words[i] = 0x04000000
                        other.words[i] = 0x04000000
                        was_updated = True
        if was_updated:
            self.updateBytes()
            other.updateBytes()

    def removePointers(self):
        pass

    def updateBytes(self):
        beWordsToBytes(self.words, self.bytes)
        # Truncate extra data
        self.bytes = self.bytes[:self.sizew*4]

    def saveToFile(self, filepath: str):
        if self.size == 0:
            return
        writeBytearrayToFile(filepath, self.bytes)


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
        if self.isIType():
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
        elif self.isIType5():
            result = f"{opcode} {rt},"
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


class Text(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, tableEntry: OverlayTableEntry=None, args=None):
        super().__init__(array_of_bytes, filename, version, tableEntry=tableEntry, args=args)

        self.instructions: List[Instruction] = list()
        for word in self.words:
            self.instructions.append(wordToInstruction(word))

        # TODO: make this a class?
        self.functions: List[List[Instruction]] = list()

    @property
    def nInstr(self):
        return len(self.instructions)

    def findFunctions(self):
        functionEnded = False
        func = list()
        offset = 0
        farthestBranch = 0
        for instr in self.instructions:
            func.append(instr)
            if functionEnded:
                self.functions.append(func)
                func = list()
                functionEnded = False

            if instr.isBranch():
                branch = from2Complement(instr.immediate, 16) + 1
                if branch > farthestBranch:
                    farthestBranch = branch

            if instr.getOpcodeName() == "JR" and instr.getRegisterName(instr.rs) == "$ra" and not farthestBranch > 0:
                functionEnded = True

            offset += 4
            farthestBranch -= 1
        if len(func) > 0:
            self.functions.append(func)

    def compareToFile(self, other: File):
        result = super().compareToFile(other)

        if isinstance(other, Text):
            result["text"] = {
                "diff_opcode": self.countDiffOpcodes(other),
                "same_opcode_same_args": self.countSameOpcodeButDifferentArguments(other),
            }

        return result

    def countDiffOpcodes(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            if not self.instructions[i].sameOpcode(other.instructions[i]):
                result += 1
        return result

    def countSameOpcodeButDifferentArguments(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                result += 1
        return result

    def blankOutDifferences(self, other_file: File):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().blankOutDifferences(other_file)
        if not isinstance(other_file, Text):
            return

        was_updated = False

        lui_found = False
        lui_pos = 0
        lui_1_register = 0
        lui_2_register = 0

        for i in range(min(self.nInstr, other_file.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other_file.instructions[i]
            if self.args is not None and self.args.ignore_branches:
                if instr1.sameOpcode(instr2):
                    if instr1.isBranch() and instr2.isBranch():
                        instr1.blankOut()
                        instr2.blankOut()
                        was_updated = True
                    elif instr1.isJType():
                        instr1.blankOut()
                        instr2.blankOut()
                        was_updated = True

            #if (instr1.isADDIU() or instr1.isSB() or instr1.isSW() or instr1.isLWCz() 
            #    or instr1.isLBU() or instr1.isLH() or instr1.isLW() or instr1.isSWCz() 
            #    or instr1.isLHU() or instr1.isSH() or instr1.isLB() or instr1.isLUI()
            #    or instr1.isLDCz()):
            #    if instr1.sameOpcode(instr2) and instr1.sameBaseRegister(instr2) and instr1.rt == instr2.rt:
            #        if abs(instr1.immediate - instr2.immediate) == 0x10:
            #            instr1.blankOut()
            #            instr2.blankOut()

            opcode = instr1.getOpcodeName()

            if instr1.sameOpcode(instr2):
                if not lui_found:
                    if opcode == "LUI":
                        lui_found = True
                        lui_pos = i
                        lui_1_register = instr1.rt
                        lui_2_register = instr2.rt
                else:
                    if opcode == "ADDIU":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "LW":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "LWC1" or opcode == "LWC2":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                            was_updated = True
                    elif opcode == "ORI":
                        if instr1.rs == lui_1_register and instr2.rs == lui_2_register:
                            instr1.blankOut()
                            instr2.blankOut()
                            self.instructions[lui_pos].blankOut() # lui
                            other_file.instructions[lui_pos].blankOut() # lui
                            lui_found = False
                        was_updated = True
            if i > lui_pos + 5:
                lui_found = False

        if was_updated:
            self.updateWords()
            other_file.updateWords()

    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return

        was_updated = False

        if self.args is not None and self.args.delete_opendisps:
            was_updated = self.deleteCallers_Graph_OpenDisps()

        was_updated = self.removeTrailingNops() or was_updated

        super().removePointers()

        lui_registers = dict()
        for i in range(len(self.instructions)):
            instr = self.instructions[i]
            opcode = instr.getOpcodeName()

            # Clean the tracked registers after X instructions have passed.
            lui_registers_aux = dict(lui_registers)
            lui_registers = dict()
            for lui_reg in lui_registers_aux:
                lui_pos, instructions_left = lui_registers_aux[lui_reg]
                instructions_left -= 1
                if instructions_left > 0:
                    lui_registers[lui_reg] = [lui_pos, instructions_left]

            if opcode == "LUI":
                lui_registers[instr.rt] = [i, self.args.track_registers]
            elif opcode in ("ADDIU", "LW", "LWU", "LWC1", "LWC2", "ORI", "LH", "LHU", "LB", "LBU"):
                rs = instr.rs
                if rs in lui_registers:
                    lui_pos, _ = lui_registers[rs]
                    self.instructions[lui_pos].blankOut() # lui
                    instr.blankOut()
                    was_updated = True
            elif instr.isJType():
                instr.blankOut()
                was_updated = True

        if was_updated:
            self.updateWords()

    def deleteCallers_Graph_OpenDisps(self) -> bool:
        was_updated = False
        graph_openDisps = address_Graph_OpenDisps.get(self.version)
        if graph_openDisps is None or graph_openDisps == 0:
            return was_updated

        last_jr = 0
        found_openDisps = False
        ranges_to_delete = []
        for i in range(self.nInstr):
            instr = self.instructions[i]
            opcode = instr.getOpcodeName()
            if opcode == "JR":
                # found end of function
                if found_openDisps:
                    ranges_to_delete.append((last_jr, i))
                    was_updated = True
                found_openDisps = False
                last_jr = i
            elif opcode == "JAL":
                # check for Graph_OpenDisps
                if graph_openDisps == instr.instr_index:
                    found_openDisps = True

        # Remove all functions that call Graph_openDisps
        for begin, end in ranges_to_delete[::-1]:
            del self.instructions[begin:end]

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False
        first_nop = self.nInstr
        for i in range(self.nInstr-1, 0-1, -1):
            instr = self.instructions[i]
            if instr.getOpcodeName() != "NOP":
                break
            first_nop = i
        if first_nop != self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated

    def updateWords(self):
        self.words = []
        for instr in self.instructions:
            self.words.append(instr.instr)
        self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".text")

        with open(filepath + ".text.asm", "w") as f:
            i = 0
            offset = 0
            for func in self.functions:
                funcName = f"func_{i}"
                if self.vRamStart != -1:
                    funcName = "func_" + toHex(self.vRamStart + offset, 6)[2:]
                f.write(f"glabel {funcName}\n")
                functionOffset = offset
                processed = []
                offsetsBranches = set()
                for instr in func:
                    offsetHex = toHex(offset, 5)[2:]
                    vramHex = ""
                    if self.vRamStart != -1:
                        vramHex = toHex(self.vRamStart + offset, 6)[2:]
                    instrHex = toHex(instr.instr, 8)[2:]

                    comment = f"/* {offsetHex} {vramHex} {instrHex} */"

                    line = str(instr)
                    if instr.isBranch():
                        #line += " HERE"
                        line = line[:-6]
                        addr = from2Complement(instr.immediate, 16)
                        branch = offset + 1*4 + addr*4
                        offsetsBranches.add(branch)
                        if self.vRamStart != -1:
                            line += ".L" + toHex(self.vRamStart + branch, 5)[2:]
                        else:
                            line += ".L" + toHex(branch, 5)[2:]

                    data = {"comment": comment, "instr": instr, "line": line}
                    processed.append(data)

                    offset += 4

                auxOffset = functionOffset
                for data in processed:
                    line = data["comment"] + "  " + data["line"]
                    if auxOffset in offsetsBranches:
                        if self.vRamStart != -1:
                            line = ".L" + toHex(self.vRamStart + auxOffset, 5)[2:] + ":\n" + line
                        else:
                            line = ".L" + toHex(auxOffset, 5)[2:] + ":\n" + line
                    f.write(line + "\n")

                    auxOffset += 4

                f.write("\n")
                i += 1


class Data(File):
    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().removePointers()

        was_updated = False
        for i in range(self.sizew):
            top_byte = (self.words[i] >> 24) & 0xFF
            if top_byte == 0x80:
                self.words[i] = top_byte << 24
                was_updated = True
            if (top_byte & 0xF0) == 0x00 and (top_byte & 0x0F) != 0x00:
                self.words[i] = top_byte << 24
                was_updated = True

        if was_updated:
            self.updateBytes() 

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".data")

        if self.size == 0:
            return

        with open(filepath + ".data.asm", "w") as f:
            # f.write(".section .data\n\n.balign 16\n\n")
            offset = 0
            for w in self.words:
                offsetHex = toHex(offset, 5)[2:]
                dataHex = toHex(w, 8)[2:]
                line = toHex(w, 8)

                f.write(f"/* {offsetHex} {dataHex} */  .word  {line}\n")
                offset += 4


class Rodata(File):
    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().removePointers()

        was_updated = False
        for i in range(self.sizew):
            top_byte = (self.words[i] >> 24) & 0xFF
            if top_byte == 0x80:
                self.words[i] = top_byte << 24
                was_updated = True
            if (top_byte & 0xF0) == 0x00 and (top_byte & 0x0F) != 0x00:
                self.words[i] = top_byte << 24
                was_updated = True

        if was_updated:
            self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".rodata")

        if self.size == 0:
            return

        with open(filepath + ".rodata.asm", "w") as f:
            # f.write(".section .rodata\n\n.balign 16\n\n")
            offset = 0
            for w in self.words:
                offsetHex = toHex(offset, 5)[2:]
                rodataHex = toHex(w, 8)[2:]
                line = toHex(w, 8)

                f.write(f"/* {offsetHex} {rodataHex} */  .word  {line}\n")
                offset += 4


class Bss(File):
    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().removePointers()
        self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".bss")

        if self.size == 0:
            return

        with open(filepath + ".bss.asm", "w") as f:
            # f.write(".section .bss\n\n.balign 16\n\n")
            offset = 0
            for w in self.words:
                offsetHex = toHex(offset, 5)[2:]
                bssHex = toHex(w, 8)[2:]
                line = toHex(w, 8)

                f.write(f"/* {offsetHex} {bssHex} */  .word  {line}\n")
                offset += 4


class RelocEntry:
    sectionNames = {
        #0: ".bss",
        1: ".text",
        2: ".data",
        3: ".rodata",
        4: ".bss", # ?
    }
    relocationsNames = {
        2: "R_MIPS_32",
        4: "R_MIPS_26",
        5: "R_MIPS_HI16",
        6: "R_MIPS_LO16",
    }

    def __init__(self, entry: int):
        self.sectionId = entry >> 30
        self.relocType = (entry >> 24) & 0x3F
        self.offset = entry & 0x00FFFFFF

    @property
    def reloc(self):
        return (self.sectionId << 30) | (self.relocType << 24) | (self.offset)

    def getSectionName(self) -> str:
        return RelocEntry.sectionNames.get(self.sectionId, str(self.sectionId))

    def getTypeName(self) -> str:
        return RelocEntry.relocationsNames.get(self.relocType, str(self.relocType))

    def __str__(self) -> str:
        section = self.getSectionName()
        reloc = self.getTypeName()
        return f"{section} {reloc} {hex(self.offset)}"
    def __repr__(self) -> str:
        return self.__str__()

class Reloc(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, tableEntry: OverlayTableEntry=None, args=None):
        super().__init__(array_of_bytes, filename, version, tableEntry=tableEntry, args=args)

        self.entries: List[RelocEntry] = list()
        for word in self.words:
            self.entries.append(RelocEntry(word))

    @property
    def nRelocs(self):
        return len(self.entries)

    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)
        # TODO
        return result

    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().removePointers()
        self.updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".reloc")

        if self.size == 0:
            return

        with open(filepath + ".reloc.asm", "w") as f:
            # f.write(".section .rodata\n\n.balign 16\n\n")
            offset = 0
            for r in self.entries:
                offsetHex = toHex(offset, 5)[2:]
                relocHex = toHex(r.reloc, 8)[2:]
                line = str(r)

                f.write(f"/* {offsetHex} {relocHex} */  {line}\n")
                offset += 4


class Overlay(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, tableEntry: OverlayTableEntry=None, args=None):
        super().__init__(array_of_bytes, filename, version, tableEntry=tableEntry, args=args)

        seekup = self.words[-1]
        self.headerBPos = self.size - seekup
        self.headerWPos = self.headerBPos//4

        text_size = self.words[self.headerWPos]
        data_size = self.words[self.headerWPos+1]
        rodata_size = self.words[self.headerWPos+2]
        bss_size = self.words[self.headerWPos+3]
        header_size = 4*5
        reloc_size = 4*self.words[self.headerWPos+4]

        start = 0
        end = text_size
        self.text = Text(self.bytes[start:end], filename, version, tableEntry=tableEntry, args=args)

        start += text_size
        end += data_size
        self.data = Data(self.bytes[start:end], filename, version, tableEntry=tableEntry, args=args)

        start += data_size
        end += rodata_size
        self.rodata = Rodata(self.bytes[start:end], filename, version, tableEntry=tableEntry, args=args)

        #start += rodata_size
        #end += bss_size
        #self.bss = Bss(self.bytes[start:end], filename, version, tableEntry=tableEntry, args=args)
        # TODO
        self.bss = Bss(self.bytes[0:0], filename, version, tableEntry=tableEntry, args=args)

        #start += bss_size
        start += rodata_size
        end += header_size
        self.header = bytesToBEWords(self.bytes[start:end])

        start += header_size
        end += reloc_size
        self.reloc = Reloc(self.bytes[start:end], filename, version, tableEntry=tableEntry, args=args)

        self.tail = bytesToBEWords(self.bytes[end:])

        """
        functions = set()
        lastHigh = 0
        for relocEntry in self.reloc.entries:
            if relocEntry.getSectionName() == ".text":
                relocType = relocEntry.getTypeName()
                offset = relocEntry.offset >> 2
                if relocType == "R_MIPS_26":
                    # print(self.text.instructions[offset])
                    functions.add(self.text.instructions[offset].instr_index<<2)
                elif relocType == "R_MIPS_HI16":
                    lastHigh = self.text.instructions[offset].immediate
                elif relocType == "R_MIPS_LO16":
                    low = self.text.instructions[offset].immediate
                    # print (toHex((lastHigh << 16) | low, 8)[2:])
        #print(len(functions))
        #for f in sorted(functions):
        #    print("func_80"+toHex(f, 6)[2:])
        """

        self.text.findFunctions()


    def compareToFile(self, other_file: File):
        result = super().compareToFile(other_file)

        if isinstance(other_file, Overlay):
            result["ovl"] = {
                "text": self.text.compareToFile(other_file.text),
                "data": self.data.compareToFile(other_file.data),
                "rodata": self.rodata.compareToFile(other_file.rodata),
                "bss": self.bss.compareToFile(other_file.bss),
                "reloc": self.reloc.compareToFile(other_file.reloc),
            }

        return result

    def blankOutDifferences(self, other_file: File):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().blankOutDifferences(other_file)
        if not isinstance(other_file, Overlay):
            return

        self.text.blankOutDifferences(other_file.text)

        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words + self.header + self.reloc.words + self.tail
        self.updateBytes()

        other_file.words = other_file.text.words + other_file.data.words  + other_file.rodata.words + other_file.bss.words + other_file.header + other_file.reloc.words + other_file.tail
        other_file.updateBytes()

    def removePointers(self):
        if self.args is not None and self.args.dont_remove_ptrs:
            return
        super().removePointers()

        for entry in self.reloc.entries:
            section = entry.getSectionName()
            type_name = entry.getTypeName()
            offset = entry.offset//4
            if entry.reloc == 0:
                continue
            if section == ".text":
                instr = self.text.instructions[offset]
                if type_name == "R_MIPS_26":
                    self.text.instructions[offset] = wordToInstruction(instr.instr & 0xFC000000)
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.text.instructions[offset] = wordToInstruction(instr.instr & 0xFFFF0000)
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .text of file '{self.version}/{self.filename}'. Reloc: {entry}")
            elif section == ".data":
                word = self.data.words[offset]
                if type_name == "R_MIPS_32":
                    self.data.words[offset] = word & 0xFF000000
                elif type_name == "R_MIPS_26":
                    self.data.words[offset] = word & 0xFC000000
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.data.words[offset] = word & 0xFFFF0000
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .data of file '{self.version}/{self.filename}'. Reloc: {entry}")
            elif section == ".rodata":
                word = self.rodata.words[offset]
                if type_name == "R_MIPS_32":
                    self.rodata.words[offset] = word & 0xFF000000
                elif type_name == "R_MIPS_26":
                    self.rodata.words[offset] = word & 0xFC000000
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.rodata.words[offset] = word & 0xFFFF0000
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .rodata of file '{self.version}/{self.filename}'. Reloc: {entry}")
            elif section == ".bss":
                word = self.bss.words[offset]
                if type_name == "R_MIPS_32":
                    self.bss.words[offset] = word & 0xFF000000
                elif type_name == "R_MIPS_26":
                    self.bss.words[offset] = word & 0xFC000000
                elif type_name in ("R_MIPS_HI16", "R_MIPS_LO16"):
                    self.bss.words[offset] = word & 0xFFFF0000
                else:
                    raise RuntimeError(f"Invalid <{type_name}> in .bss of file '{self.version}/{self.filename}'. Reloc: {entry}")
            else:
                pass
                #raise RuntimeError(f"Invalid reloc section <{section}> in file '{self.version}/{self.filename}'. Reloc: {entry}")


        self.text.removePointers()
        self.data.removePointers()
        self.rodata.removePointers()
        self.bss.removePointers()
        self.reloc.removePointers()

        self.updateBytes()

    def updateBytes(self):
        self.words = self.text.words + self.data.words + self.rodata.words + self.bss.words + self.header + self.reloc.words + self.tail
        super().updateBytes()

    def saveToFile(self, filepath: str):
        self.text.saveToFile(filepath)
        self.data.saveToFile(filepath)
        self.rodata.saveToFile(filepath)
        self.bss.saveToFile(filepath)
        self.reloc.saveToFile(filepath)

class OverlayTableEntry:
    def __init__(self, array_of_bytes: bytearray):
        wordsArray = bytesToBEWords(array_of_bytes)
        self.vromStart = wordsArray[0]
        self.vromEnd = wordsArray[1]
        self.vramStart = wordsArray[2]
        self.vramEnd = wordsArray[3]
        self.ramAddress = wordsArray[4]
        self.initVars = wordsArray[5]
        self.filenameAddres = wordsArray[6]
        self.allocationType = (wordsArray[7] > 16) & 0xFFFF
        self.instancesNum = (wordsArray[7] > 8) & 0xFF

def getVersionAbbr(filename: str) -> str:
    for ver in versions:
        if "baserom_" + ver + "/" in filename:
            return versions[ver]
    # If the version wasn't found.
    return filename

def countUnique(row: list) -> int:
    unique = set(row)
    count = len(unique)
    if "" in unique:
        count -= 1
    return count

def removePointers(args, filedata: bytearray) -> bytearray:
    if args.dont_remove_ptrs:
        return filedata
    if not args.ignore04: # This will probably grow...
        return filedata

    words = bytesToBEWords(filedata)
    for i in range(len(words)):
        w = words[i]
        if args.ignore04:
            if ((w >> 24) & 0xFF) == 0x04:
                words[i] = 0x04000000
    return beWordsToBytes(words, filedata)


def getHashesOfFiles(args, filesPath: List[str]) -> List[str]:
    hashList = []
    for path in filesPath:
        f = readFileAsBytearray(path)
        if len(f) != 0:
            fHash = getStrHash(removePointers(args, f))
            line = fHash + " " + path # To be consistent with runCommandGetOutput("md5sum", md5arglist)
            hashList.append(line)
    return hashList

def compareFileAcrossVersions(filename: str, versionsList: List[str], dmaAddresses: dict, actorOverlayTable: Dict[str, List[OverlayTableEntry]], args) -> List[List[str]]:
    md5arglist = list(map(lambda orig_string: "baserom_" + orig_string + "/" + filename, versionsList))
    # os.system( "md5sum " + " ".join(filesPath) )

    # Get hashes.
    # output = runCommandGetOutput("md5sum", filesPath)
    output = getHashesOfFiles(args, md5arglist)

    # Print md5hash
    #print("\n".join(output))
    #print()

    filesHashes = dict() # "NN0": "339614255f179a1e308d954d8f7ffc0a"
    firstFilePerHash = dict() # "339614255f179a1e308d954d8f7ffc0a": "NN0"

    for line in output:
        trimmed = removeExtraWhitespace(line)
        filehash, filepath = trimmed.split(" ")
        abbr = getVersionAbbr(filepath)

        # Map each abbreviation and its hash.
        filesHashes[abbr] = filehash

        # Find out where in which version this hash appeared for first time.
        if filehash not in firstFilePerHash:
            firstFilePerHash[filehash] = abbr

    row = [filename]
    for ver in versionsList:
        abbr = versions.get(ver, None)

        if abbr in filesHashes:
            fHash = filesHashes[abbr]
            row.append(firstFilePerHash[fHash])
        else:
            row.append("")
    return [row]

def compareOverlayAcrossVersions(filename: str, versionsList: List[str], dmaAddresses: dict, actorOverlayTable: Dict[str, List[OverlayTableEntry]], args) -> List[List[str]]:
    column = []
    filesHashes = dict() # "filename": {"NN0": hash}
    firstFilePerHash = dict() # "filename": {hash: "NN0"}

    if filename.startswith("#"):
        return column

    is_overlay = filename.startswith("ovl_")

    for version in versionsList:
        path = os.path.join("baserom_" + version, filename)

        array_of_bytes = readFileAsBytearray(path)
        if len(array_of_bytes) == 0:
            continue

        if is_overlay:
            virtStart, virtEnd, physStart, physEnd = -1, -1, -1, -1
            tableEntry = None
            if version in dmaAddresses:
                versionData = dmaAddresses[version]
                if filename in versionData:
                    dmaData = versionData[filename]
                    virtStart, virtEnd, physStart, physEnd = dmaData
            if virtStart != -1 and version in actorOverlayTable:
                for entry in actorOverlayTable[version]:
                    if entry.vromStart == virtStart:
                        tableEntry = entry
                        break

            f = Overlay(array_of_bytes, filename, version, tableEntry=tableEntry, args=args)
        else:
            f = File(array_of_bytes, filename, version, args=args)
        f.removePointers()
        if args.savetofile:
            new_file_path = os.path.join(args.savetofile, version, filename)
            f.saveToFile(new_file_path)

        abbr = getVersionAbbr(path)

        if isinstance(f, Overlay):
            subfiles = {
                ".text" : f.text,
                ".data" : f.data,
                ".rodata" : f.rodata,
                #".bss" : f.bss,
                #".reloc" : f.reloc,
            }
        else:
            subfiles = {
                "" : f,
            }

        for section, sub in subfiles.items():
            file_section = filename + section
            if file_section not in filesHashes:
                filesHashes[file_section] = dict()
                firstFilePerHash[file_section] = dict()

            f_hash = sub.getHash()
            # Map each abbreviation to its hash.
            filesHashes[file_section][abbr] = f_hash

            # Find out where in which version this hash appeared for first time.
            if f_hash not in firstFilePerHash[file_section]:
                firstFilePerHash[file_section][f_hash] = abbr

    for file_section in filesHashes:
        row = [file_section]
        for version in versionsList:
            abbr = versions.get(version)

            if abbr in filesHashes[file_section]:
                fHash = filesHashes[file_section][abbr]
                row.append(firstFilePerHash[file_section][fHash])
            else:
                row.append("")
        column.append(row)

    return column


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("versionlist", help="Path to version list.")
    parser.add_argument("filelist", help="List of filenames of the ROM that will be compared.")
    parser.add_argument("--noheader", help="Disables the csv header.", action="store_true")
    parser.add_argument("--ignore04", help="Ignores words starting with 0x04.", action="store_true")
    parser.add_argument("--overlays", help="Treats the files in filelist as overlays.", action="store_true")
    parser.add_argument("--savetofile", help="Specify a folder where each part of an overlay will be written.", metavar="FOLDER")
    parser.add_argument("--track-registers", help="Set for how many instructions a register will be tracked.", type=int, default=8)
    parser.add_argument("--delete-opendisps", help="Will try to find and delete every function that calls Graph_OpenDisps.", action="store_true")
    parser.add_argument("--dont-remove-ptrs", help="Disable the pointer removal feature.", action="store_true")
    args = parser.parse_args()

    versionsList = []
    with open(args.versionlist) as f:
        for version in f:
            if version.startswith("#"):
                continue
            versionsList.append(version.strip())
    filesList = readFile(args.filelist)

    if args.savetofile is not None:
        for ver in versionsList:
            os.makedirs(os.path.join(args.savetofile, ver), exist_ok=True)

    dmaAddresses = dict()
    actorOverlayTable: Dict[str, List[OverlayTableEntry]] = dict()
    for version in versionsList:
        filetable = f'baserom_{version}/dma_addresses.txt'
        if os.path.exists(filetable):
            dmaAddresses[version] = dict()
            with open(filetable) as f:
                for line in f:
                    filename, *data = line.strip().split(",")
                    dmaAddresses[version][filename] = list(map(int, data))

        codePath = os.path.join("baserom_" + version, "code")
        tableOffset = offset_ActorOverlayTable[version]
        if os.path.exists(codePath) and tableOffset != 0x0:
            codeData = readFileAsBytearray(codePath)
            i = 0
            table = list()
            while i < ACTOR_ID_MAX:
                entry = OverlayTableEntry(codeData[tableOffset + i*0x20 : tableOffset + (i+1)*0x20])
                table.append(entry)
                i += 1
            actorOverlayTable[version] = table

    if not args.noheader:
        # Print csv header
        print("Object name", end="")
        for ver in versionsList:
            print("," + ver, end="")
        print(",Different versions", end="")
        print()

    compareFunction = compareFileAcrossVersions
    if args.overlays:
        compareFunction = compareOverlayAcrossVersions

    numCores = cpu_count() + 1
    p = Pool(numCores)
    for column in p.imap(partial(compareFunction, versionsList=versionsList, dmaAddresses=dmaAddresses, actorOverlayTable=actorOverlayTable, args=args), filesList):
        for row in column:
            # Print csv row
            for cell in row:
                print(cell + ",", end="")
            print(countUnique(row)-1)

if __name__ == "__main__":
    main()
