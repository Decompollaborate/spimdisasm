#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import dataclasses
import enum


class sRegister(enum.Enum):
    R0 = 0
    AT = 1
    V0 = 2
    V1 = 3
    A0 = 4
    A1 = 5
    A2 = 6
    A3 = 7
    T0 = 8
    T1 = 9
    T2 = 10
    T3 = 11
    T4 = 12
    T5 = 13
    T6 = 14
    T7 = 15
    S0 = 16
    S1 = 17
    S2 = 18
    S3 = 19
    S4 = 20
    S5 = 21
    S6 = 22
    S7 = 23
    T8 = 24
    T9 = 25
    K0 = 26
    K1 = 27
    GP = 28
    SP = 29
    FP = 30
    RA = 31


class RegisterFloat(enum.Enum):
    F0 = 0
    F1 = 1
    F2 = 2
    F3 = 3
    F4 = 4
    F5 = 5
    F6 = 6
    F7 = 7
    F8 = 8
    F9 = 9
    F10 = 10
    F11 = 11
    F12 = 12
    F13 = 13
    F14 = 14
    F15 = 15
    F16 = 16
    F17 = 17
    F18 = 18
    F19 = 19
    F20 = 20
    F21 = 21
    F22 = 22
    F23 = 23
    F24 = 24
    F25 = 25
    F26 = 26
    F27 = 27
    F28 = 28
    F29 = 29
    F30 = 30
    F31 = 31


@enum.unique
class InstructionId(enum.Enum):
    INVALID   = -1

    SLL       = enum.auto() # Shift word Left Logical

    SRL       = enum.auto() # Shift word Right Logical
    SRA       = enum.auto() # Shift word Right Arithmetic
    SLLV      = enum.auto() # Shift word Left Logical Variable

    SRLV      = enum.auto() # Shift word Right Logical Variable
    SRAV      = enum.auto() # Shift word Right Arithmetic Variable

    JR        = enum.auto() # Jump Register
    JALR      = enum.auto() # Jump And Link Register
    MOVZ      = enum.auto() # MOVe conditional on Zero
    MOVN      = enum.auto() # MOVe conditional on Not zero
    SYSCALL   = enum.auto() # SYStem CALL
    BREAK     = enum.auto() # Break

    SYNC      = enum.auto() # Sync

    MFHI      = enum.auto() # Move From HI register
    MTHI      = enum.auto() # Move To HI register
    MFLO      = enum.auto() # Move From LO register
    MTLO      = enum.auto() # Move To LO register
    DSLLV     = enum.auto() # Doubleword Shift Left Logical Variable

    DSRLV     = enum.auto() # Doubleword Shift Right Logical Variable
    DSRAV     = enum.auto() # Doubleword Shift Right Arithmetic Variable

    MULT      = enum.auto() # MULTtiply word
    MULTU     = enum.auto() # MULTtiply Unsigned word
    DIV       = enum.auto() # DIVide word
    DIVU      = enum.auto() # DIVide Unsigned word
    DMULT     = enum.auto() # Doubleword MULTiply
    DMULTU    = enum.auto() # Doubleword MULTiply Unsigned
    DDIV      = enum.auto() # Doubleword DIVide
    DDIVU     = enum.auto() # Doubleword DIVide Unsigned

    ADD       = enum.auto() # ADD word
    ADDU      = enum.auto() # ADD Unsigned word
    SUB       = enum.auto() # Subtract word
    SUBU      = enum.auto() # SUBtract Unsigned word
    AND       = enum.auto() # AND
    OR        = enum.auto() # OR
    XOR       = enum.auto() # eXclusive OR
    NOR       = enum.auto() # Not OR

    SLT       = enum.auto() # Set on Less Than
    SLTU      = enum.auto() # Set on Less Than Unsigned
    DADD      = enum.auto() # Doubleword Add
    DADDU     = enum.auto() # Doubleword Add Unsigned
    DSUB      = enum.auto() # Doubleword SUBtract
    DSUBU     = enum.auto() # Doubleword SUBtract Unsigned

    TGE       = enum.auto() # Trap if Greater or Equal
    TGEU      = enum.auto() # Trap if Greater or Equal Unsigned
    TLT       = enum.auto() # Trap if Less Than
    TLTU      = enum.auto() # Trap if Less Than Unsigned
    TEQ       = enum.auto() # Trap if EQual

    TNE       = enum.auto() # Trap if Not Equal

    DSLL      = enum.auto() # Doubleword Shift Left Logical

    DSRL      = enum.auto() # Doubleword Shift Right Logical
    DSRA      = enum.auto() # Doubleword Shift Right Arithmetic
    DSLL32    = enum.auto() # Doubleword Shift Left Logical plus 32

    DSRL32    = enum.auto() # Doubleword Shift Right Logical plus 32
    DSRA32    = enum.auto() # Doubleword Shift Right Arithmetic plus 32

    BLTZ      = enum.auto() # Branch on Less Than Zero
    BGEZ      = enum.auto() # Branch on Greater than or Equal to Zero
    BLTZL     = enum.auto() # Branch on Less Than Zero Likely
    BGEZL     = enum.auto() # Branch on Greater than or Equal to Zero Likely

    TGEI      = enum.auto()
    TGEIU     = enum.auto()
    TLTI      = enum.auto()
    TLTIU     = enum.auto()

    BLTZAL    = enum.auto()
    BGEZAL    = enum.auto()
    BLTZALL   = enum.auto()
    BGEZALL   = enum.auto()

    TEQI      = enum.auto()
    TNEI      = enum.auto()

    J         = enum.auto() # Jump
    JAL       = enum.auto() # Jump And Link
    BEQ       = enum.auto() # Branch on EQual
    BNE       = enum.auto() # Branch on Not Equal
    BLEZ      = enum.auto() # Branch on Less than or Equal to Zero
    BGTZ      = enum.auto() # Branch on Greater Than Zero

    ADDI      = enum.auto() # Add Immediate
    ADDIU     = enum.auto() # Add Immediate Unsigned Word
    SLTI      = enum.auto() # Set on Less Than Immediate
    SLTIU     = enum.auto() # Set on Less Than Immediate Unsigned
    ANDI      = enum.auto() # And Immediate
    ORI       = enum.auto() # Or Immediate
    XORI      = enum.auto() # eXclusive OR Immediate
    LUI       = enum.auto() # Load Upper Immediate

    MFC0      = enum.auto() # Move word From CP0
    DMFC0     = enum.auto() # Doubleword Move From CP0
    CFC0      = enum.auto() # Move control word From CP0

    MTC0      = enum.auto() # Move word to CP0
    DMTC0     = enum.auto() # Doubleword Move To CP0
    CTC0      = enum.auto() # Move control word To CP0

    TLBR      = enum.auto() # Read Indexed TLB Entry
    TLBWI     = enum.auto() # Write Indexed TLB Entry
    TLBWR     = enum.auto() # Write Random TLB Entry
    TLBP      = enum.auto() # Probe TLB for Matching Entry
    ERET      = enum.auto() # Return from Exception

    BC0T      = enum.auto() # Branch on FP True
    BC0F      = enum.auto() # Branch on FP False
    BC0TL     = enum.auto() # Branch on FP True Likely
    BC0FL     = enum.auto() # Branch on FP False Likely

    MFC1      = enum.auto() # Move Word From Floating-Point
    DMFC1     = enum.auto() # Doubleword Move From Floating-Point
    CFC1      = enum.auto() # Move Control Word from Floating-Point

    MTC1      = enum.auto() # Move Word to Floating-Point
    DMTC1     = enum.auto() # Doubleword Move To Floating-Point
    CTC1      = enum.auto() # Move Control Word to Floating-Point

    BC1F      = enum.auto()
    BC1T      = enum.auto()
    BC1FL     = enum.auto()
    BC1TL     = enum.auto()
    ADD_S     = enum.auto() # Floating-Point Add
    SUB_S     = enum.auto() # Floating-Point Sub
    MUL_S     = enum.auto() # Floating-Point Multiply
    DIV_S     = enum.auto() # Floating-Point Divide
    SQRT_S    = enum.auto() # Floating-Point Square Root
    ABS_S     = enum.auto() # Floating-Point Absolute Value
    MOV_S     = enum.auto() # Floating-Point Move
    NEG_S     = enum.auto() # Floating-Point Negate
    ROUND_L_S = enum.auto() # Floating-Point Round to Long Fixed-Point
    TRUNC_L_S = enum.auto() # Floating-Point Truncate to Long Fixed-Point
    CEIL_L_S  = enum.auto() # Floating-Point Ceiling Convert to Long Fixed-Point
    FLOOR_L_S = enum.auto() # Floating-Point Floor Convert to Long Fixed-Point
    ROUND_W_S = enum.auto() # Floating-Point Round to Word Fixed-Point
    TRUNC_W_S = enum.auto() # Floating-Point Truncate to Word Fixed-Point
    CEIL_W_S  = enum.auto() # Floating-Point Ceiling Convert to Word Fixed-Point
    FLOOR_W_S = enum.auto() # Floating-Point Floor Convert to Word Fixed-Point
    CVT_D_S   = enum.auto()
    CVT_W_S   = enum.auto()
    CVT_L_S   = enum.auto()
    C_F_S     = enum.auto()
    C_UN_S    = enum.auto()
    C_EQ_S    = enum.auto()
    C_UEQ_S   = enum.auto()
    C_OLT_S   = enum.auto()
    C_ULT_S   = enum.auto()
    C_OLE_S   = enum.auto()
    C_ULE_S   = enum.auto()
    C_SF_S    = enum.auto()
    C_NGLE_S  = enum.auto()
    C_SEQ_S   = enum.auto()
    C_NGL_S   = enum.auto()
    C_LT_S    = enum.auto()
    C_NGE_S   = enum.auto()
    C_LE_S    = enum.auto()
    C_NGT_S   = enum.auto()
    ADD_D     = enum.auto() # Floating-Point Add
    SUB_D     = enum.auto() # Floating-Point Sub
    MUL_D     = enum.auto() # Floating-Point Multiply
    DIV_D     = enum.auto() # Floating-Point Divide
    SQRT_D    = enum.auto() # Floating-Point Square Root
    ABS_D     = enum.auto() # Floating-Point Absolute Value
    MOV_D     = enum.auto() # Floating-Point Move
    NEG_D     = enum.auto() # Floating-Point Negate
    ROUND_L_D = enum.auto() # Floating-Point Round to Long Fixed-Point
    TRUNC_L_D = enum.auto() # Floating-Point Truncate to Long Fixed-Point
    CEIL_L_D  = enum.auto() # Floating-Point Ceiling Convert to Long Fixed-Point
    FLOOR_L_D = enum.auto() # Floating-Point Floor Convert to Long Fixed-Point
    ROUND_W_D = enum.auto() # Floating-Point Round to Word Fixed-Point
    TRUNC_W_D = enum.auto() # Floating-Point Truncate to Word Fixed-Point
    CEIL_W_D  = enum.auto() # Floating-Point Ceiling Convert to Word Fixed-Point
    FLOOR_W_D = enum.auto() # Floating-Point Floor Convert to Word Fixed-Point
    CVT_S_D   = enum.auto()
    CVT_W_D   = enum.auto()
    CVT_L_D   = enum.auto()
    C_F_D     = enum.auto()
    C_UN_D    = enum.auto()
    C_EQ_D    = enum.auto()
    C_UEQ_D   = enum.auto()
    C_OLT_D   = enum.auto()
    C_ULT_D   = enum.auto()
    C_OLE_D   = enum.auto()
    C_ULE_D   = enum.auto()
    C_SF_D    = enum.auto()
    C_NGLE_D  = enum.auto()
    C_SEQ_D   = enum.auto()
    C_NGL_D   = enum.auto()
    C_LT_D    = enum.auto()
    C_NGE_D   = enum.auto()
    C_LE_D    = enum.auto()
    C_NGT_D   = enum.auto()
    CVT_S_W   = enum.auto()
    CVT_D_W   = enum.auto()
    CVT_S_L   = enum.auto()
    CVT_D_L   = enum.auto()

    BEQL      = enum.auto() # Branch on EQual Likely
    BNEL      = enum.auto() # Branch on Not Equal Likely
    BLEZL     = enum.auto() # Branch on Less than or Equal to Zero Likely
    BGTZL     = enum.auto() # Branch on Greater Than Zero Likely

    DADDI     = enum.auto() # Doubleword add Immediate
    DADDIU    = enum.auto() # Doubleword add Immediate Unsigned
    LDL       = enum.auto() # Load Doubleword Left
    LDR       = enum.auto() # Load Doubleword Right

    LB        = enum.auto() # Load Byte
    LH        = enum.auto() # Load Halfword
    LWL       = enum.auto() # Load Word Left
    LW        = enum.auto() # Load Word
    LBU       = enum.auto() # Load Byte Insigned
    LHU       = enum.auto() # Load Halfword Unsigned
    LWR       = enum.auto() # Load Word Right
    LWU       = enum.auto() # Load Word Unsigned

    SB        = enum.auto() # Store Byte
    SH        = enum.auto() # Store Halfword
    SWL       = enum.auto() # Store Word Left
    SW        = enum.auto() # Store Word
    SDL       = enum.auto() # Store Doubleword Left
    SDR       = enum.auto() # Store Doubleword Right
    SWR       = enum.auto() # Store Word Right
    CACHE     = enum.auto() # Cache

    LL        = enum.auto() # Load Linked word
    LWC1      = enum.auto() # Load Word to Coprocessor z
    LWC2      = enum.auto() # Load Word to Coprocessor z
    PREF      = enum.auto() # Prefetch
    LLD       = enum.auto() # Load Linked Doubleword
    LDC1      = enum.auto() # Load Doubleword to Coprocessor z
    LDC2      = enum.auto() # Load Doubleword to Coprocessor z
    LD        = enum.auto() # Load Doubleword

    SC        = enum.auto() # Store Conditional word
    SWC1      = enum.auto() # Store Word from Coprocessor z
    SWC2      = enum.auto() # Store Word from Coprocessor z
    #
    SCD       = enum.auto() # Store Conditional Doubleword
    SDC1      = enum.auto() # Store Doubleword from Coprocessor z
    SDC2      = enum.auto() # Store Doubleword from Coprocessor z
    SD        = enum.auto() # Store Doubleword

    # Pseudo-Instruction Unique IDs
    BEQZ      = enum.auto() # Branch on EQual Zero
    BNEZ      = enum.auto() # Branch on Not Equal Zero
    B         = enum.auto() # Branch (unconditional)
    NOP       = enum.auto() # No OPeration
    MOVE      = enum.auto() # Move
    NEGU      = enum.auto() 
    NOT       = enum.auto() # Not


@enum.unique
class InstructionVectorId(enum.Enum):
    INVALID   = -1

    VMULF     = enum.auto()
    VMULU     = enum.auto()
    VRNDP     = enum.auto()
    VMULQ     = enum.auto()
    VMUDL     = enum.auto()
    VMUDM     = enum.auto()
    VMUDN     = enum.auto()
    VMUDH     = enum.auto()
    VMACF     = enum.auto()
    VMACU     = enum.auto()
    VRNDN     = enum.auto()
    VMACQ     = enum.auto()
    VMADL     = enum.auto()
    VMADM     = enum.auto()
    VMADN     = enum.auto()
    VMADH     = enum.auto()
    VADD      = enum.auto()
    VSUB      = enum.auto()
    VABS      = enum.auto()
    VADDC     = enum.auto()
    VSUBC     = enum.auto()
    VSAR      = enum.auto()
    VAND      = enum.auto()
    VNAND     = enum.auto()
    VOR       = enum.auto()
    VNOR      = enum.auto()
    VXOR      = enum.auto()
    VNXOR     = enum.auto()

    VLT       = enum.auto()
    VEQ       = enum.auto()
    VNE       = enum.auto()
    VGE       = enum.auto()
    VCL       = enum.auto()
    VCH       = enum.auto()
    VCR       = enum.auto()
    VMRG      = enum.auto()

    VRCP      = enum.auto()
    VRCPL     = enum.auto()
    VRCPH     = enum.auto()
    VMOV      = enum.auto()
    VRSQ      = enum.auto()
    VRSQL     = enum.auto()
    VRSQH     = enum.auto()
    VNOP      = enum.auto()

    MFC2      = enum.auto()
    MTC2      = enum.auto()
    CFC2      = enum.auto()
    CTC2      = enum.auto()

    SBV       = enum.auto()
    SSV       = enum.auto()
    SLV       = enum.auto()
    SDV       = enum.auto()

    SQV       = enum.auto()
    SRV       = enum.auto()

    SPV       = enum.auto()

    SUV       = enum.auto()
    SWV       = enum.auto()

    SHV       = enum.auto()

    SFV       = enum.auto()
    STV       = enum.auto()

    LBV       = enum.auto()
    LSV       = enum.auto()
    LLV       = enum.auto()
    LDV       = enum.auto()

    LQV       = enum.auto()
    LRV       = enum.auto()

    LPV       = enum.auto()

    LUV       = enum.auto()

    LHV       = enum.auto()

    LFV       = enum.auto()
    LTV       = enum.auto()

@enum.unique
class InstrType(enum.Enum):
    typeUnknown = -1

    typeJ       = enum.auto()
    typeI       = enum.auto()
    typeR       = enum.auto()

    typeRegimm  = enum.auto()

@dataclasses.dataclass
class InstrDescriptor:
    operand1: str|None
    operand2: str|None
    operand3: str|None

    instrType: InstrType

    isBranch: bool
    isBranchLikely: bool
    isTrap: bool

    isFloat: bool
    isDouble: bool

    isUnsigned: bool

    modifiesRt: bool
    modifiesRd: bool

    mipsVersion: int|None = None
    "Version in which this instruction was introduced. `None` means unknown"
    isRsp: bool = False


instructionDescriptorDict: dict[InstructionId|InstructionVectorId, InstrDescriptor] = {
    # InstructionId.SLL       : InstrDescriptor(),

    # InstructionId.SRL       : InstrDescriptor(),
    # InstructionId.SRA       : InstrDescriptor(),
    # InstructionId.SLLV      : InstrDescriptor(),

    # InstructionId.SRLV      : InstrDescriptor(),
    # InstructionId.SRAV      : InstrDescriptor(),

    # InstructionId.JR        : InstrDescriptor(),
    # InstructionId.JALR      : InstrDescriptor(),
    # InstructionId.MOVZ      : InstrDescriptor(),
    # InstructionId.MOVN      : InstrDescriptor(),
    # InstructionId.SYSCALL   : InstrDescriptor(),
    # InstructionId.BREAK     : InstrDescriptor(),

    # InstructionId.SYNC      : InstrDescriptor(),

    # InstructionId.MFHI      : InstrDescriptor(),
    # InstructionId.MTHI      : InstrDescriptor(),
    # InstructionId.MFLO      : InstrDescriptor(),
    # InstructionId.MTLO      : InstrDescriptor(),
    # InstructionId.DSLLV     : InstrDescriptor(),

    # InstructionId.DSRLV     : InstrDescriptor(),
    # InstructionId.DSRAV     : InstrDescriptor(),

    # InstructionId.MULT      : InstrDescriptor(),
    # InstructionId.MULTU     : InstrDescriptor(),
    # InstructionId.DIV       : InstrDescriptor(),
    # InstructionId.DIVU      : InstrDescriptor(),
    # InstructionId.DMULT     : InstrDescriptor(),
    # InstructionId.DMULTU    : InstrDescriptor(),
    # InstructionId.DDIV      : InstrDescriptor(),
    # InstructionId.DDIVU     : InstrDescriptor(),

    # InstructionId.ADD       : InstrDescriptor(),
    # InstructionId.ADDU      : InstrDescriptor(),
    # InstructionId.SUB       : InstrDescriptor(),
    # InstructionId.SUBU      : InstrDescriptor(),
    # InstructionId.AND       : InstrDescriptor(),
    # InstructionId.OR        : InstrDescriptor(),
    # InstructionId.XOR       : InstrDescriptor(),
    # InstructionId.NOR       : InstrDescriptor(),

    # InstructionId.SLT       : InstrDescriptor(),
    # InstructionId.SLTU      : InstrDescriptor(),
    # InstructionId.DADD      : InstrDescriptor(),
    # InstructionId.DADDU     : InstrDescriptor(),
    # InstructionId.DSUB      : InstrDescriptor(),
    # InstructionId.DSUBU     : InstrDescriptor(),

    # InstructionId.TGE       : InstrDescriptor(),
    # InstructionId.TGEU      : InstrDescriptor(),
    # InstructionId.TLT       : InstrDescriptor(),
    # InstructionId.TLTU      : InstrDescriptor(),
    # InstructionId.TEQ       : InstrDescriptor(),

    # InstructionId.TNE       : InstrDescriptor(),

    # InstructionId.DSLL      : InstrDescriptor(),

    # InstructionId.DSRL      : InstrDescriptor(),
    # InstructionId.DSRA      : InstrDescriptor(),
    # InstructionId.DSLL32    : InstrDescriptor(),

    # InstructionId.DSRL32    : InstrDescriptor(),
    # InstructionId.DSRA32    : InstrDescriptor(),

    # OP  rs, IMM
    InstructionId.BLTZ      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BGEZ      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BLTZL     : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BGEZL     : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.TGEI      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=False, isBranchLikely=False, isTrap=True, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.TGEIU     : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=False, isBranchLikely=False, isTrap=True, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.TLTI      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=False, isBranchLikely=False, isTrap=True, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.TLTIU     : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=False, isBranchLikely=False, isTrap=True, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BLTZAL    : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BGEZAL    : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BLTZALL   : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BGEZALL   : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.TEQI      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=False, isBranchLikely=False, isTrap=True, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.TNEI      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=False, isBranchLikely=False, isTrap=True, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),

    # OP LABEL
    # InstructionId.J         : InstrDescriptor(),
    # InstructionId.JAL       : InstrDescriptor(),

    InstructionId.BEQ       : InstrDescriptor("{rs}, ", "{rt}, ", "{IMM}", InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BNE       : InstrDescriptor("{rs}, ", "{rt}, ", "{IMM}", InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BLEZ      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BGTZ      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BEQL      : InstrDescriptor("{rs}, ", "{rt}, ", "{IMM}", InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BNEL      : InstrDescriptor("{rs}, ", "{rt}, ", "{IMM}", InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BLEZL     : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BGTZL     : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=True, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),

    # InstructionId.ADDI      : InstrDescriptor(),
    # InstructionId.ADDIU     : InstrDescriptor(),
    # InstructionId.SLTI      : InstrDescriptor(),
    # InstructionId.SLTIU     : InstrDescriptor(),
    # InstructionId.ANDI      : InstrDescriptor(),
    # InstructionId.ORI       : InstrDescriptor(),
    # InstructionId.XORI      : InstrDescriptor(),
    # InstructionId.LUI       : InstrDescriptor(),

    # InstructionId.MFC0      : InstrDescriptor(),
    # InstructionId.DMFC0     : InstrDescriptor(),
    # InstructionId.CFC0      : InstrDescriptor(),

    # InstructionId.MTC0      : InstrDescriptor(),
    # InstructionId.DMTC0     : InstrDescriptor(),
    # InstructionId.CTC0      : InstrDescriptor(),

    # InstructionId.TLBR      : InstrDescriptor(),
    # InstructionId.TLBWI     : InstrDescriptor(),
    # InstructionId.TLBWR     : InstrDescriptor(),
    # InstructionId.TLBP      : InstrDescriptor(),
    # InstructionId.ERET      : InstrDescriptor(),

    # InstructionId.BC0T      : InstrDescriptor(),
    # InstructionId.BC0F      : InstrDescriptor(),
    # InstructionId.BC0TL     : InstrDescriptor(),
    # InstructionId.BC0FL     : InstrDescriptor(),

    # InstructionId.MFC1      : InstrDescriptor(),
    # InstructionId.DMFC1     : InstrDescriptor(),
    # InstructionId.CFC1      : InstrDescriptor(),

    # InstructionId.MTC1      : InstrDescriptor(),
    # InstructionId.DMTC1     : InstrDescriptor(),
    # InstructionId.CTC1      : InstrDescriptor(),

    # InstructionId.BC1F      : InstrDescriptor(),
    # InstructionId.BC1T      : InstrDescriptor(),
    # InstructionId.BC1FL     : InstrDescriptor(),
    # InstructionId.BC1TL     : InstrDescriptor(),
    # InstructionId.ADD_S     : InstrDescriptor(),
    # InstructionId.SUB_S     : InstrDescriptor(),
    # InstructionId.MUL_S     : InstrDescriptor(),
    # InstructionId.DIV_S     : InstrDescriptor(),
    # InstructionId.SQRT_S    : InstrDescriptor(),
    # InstructionId.ABS_S     : InstrDescriptor(),
    # InstructionId.MOV_S     : InstrDescriptor(),
    # InstructionId.NEG_S     : InstrDescriptor(),
    # InstructionId.ROUND_L_S : InstrDescriptor(),
    # InstructionId.TRUNC_L_S : InstrDescriptor(),
    # InstructionId.CEIL_L_S  : InstrDescriptor(),
    # InstructionId.FLOOR_L_S : InstrDescriptor(),
    # InstructionId.ROUND_W_S : InstrDescriptor(),
    # InstructionId.TRUNC_W_S : InstrDescriptor(),
    # InstructionId.CEIL_W_S  : InstrDescriptor(),
    # InstructionId.FLOOR_W_S : InstrDescriptor(),
    # InstructionId.CVT_D_S   : InstrDescriptor(),
    # InstructionId.CVT_W_S   : InstrDescriptor(),
    # InstructionId.CVT_L_S   : InstrDescriptor(),
    # InstructionId.C_F_S     : InstrDescriptor(),
    # InstructionId.C_UN_S    : InstrDescriptor(),
    # InstructionId.C_EQ_S    : InstrDescriptor(),
    # InstructionId.C_UEQ_S   : InstrDescriptor(),
    # InstructionId.C_OLT_S   : InstrDescriptor(),
    # InstructionId.C_ULT_S   : InstrDescriptor(),
    # InstructionId.C_OLE_S   : InstrDescriptor(),
    # InstructionId.C_ULE_S   : InstrDescriptor(),
    # InstructionId.C_SF_S    : InstrDescriptor(),
    # InstructionId.C_NGLE_S  : InstrDescriptor(),
    # InstructionId.C_SEQ_S   : InstrDescriptor(),
    # InstructionId.C_NGL_S   : InstrDescriptor(),
    # InstructionId.C_LT_S    : InstrDescriptor(),
    # InstructionId.C_NGE_S   : InstrDescriptor(),
    # InstructionId.C_LE_S    : InstrDescriptor(),
    # InstructionId.C_NGT_S   : InstrDescriptor(),
    # InstructionId.ADD_D     : InstrDescriptor(),
    # InstructionId.SUB_D     : InstrDescriptor(),
    # InstructionId.MUL_D     : InstrDescriptor(),
    # InstructionId.DIV_D     : InstrDescriptor(),
    # InstructionId.SQRT_D    : InstrDescriptor(),
    # InstructionId.ABS_D     : InstrDescriptor(),
    # InstructionId.MOV_D     : InstrDescriptor(),
    # InstructionId.NEG_D     : InstrDescriptor(),
    # InstructionId.ROUND_L_D : InstrDescriptor(),
    # InstructionId.TRUNC_L_D : InstrDescriptor(),
    # InstructionId.CEIL_L_D  : InstrDescriptor(),
    # InstructionId.FLOOR_L_D : InstrDescriptor(),
    # InstructionId.ROUND_W_D : InstrDescriptor(),
    # InstructionId.TRUNC_W_D : InstrDescriptor(),
    # InstructionId.CEIL_W_D  : InstrDescriptor(),
    # InstructionId.FLOOR_W_D : InstrDescriptor(),
    # InstructionId.CVT_S_D   : InstrDescriptor(),
    # InstructionId.CVT_W_D   : InstrDescriptor(),
    # InstructionId.CVT_L_D   : InstrDescriptor(),
    # InstructionId.C_F_D     : InstrDescriptor(),
    # InstructionId.C_UN_D    : InstrDescriptor(),
    # InstructionId.C_EQ_D    : InstrDescriptor(),
    # InstructionId.C_UEQ_D   : InstrDescriptor(),
    # InstructionId.C_OLT_D   : InstrDescriptor(),
    # InstructionId.C_ULT_D   : InstrDescriptor(),
    # InstructionId.C_OLE_D   : InstrDescriptor(),
    # InstructionId.C_ULE_D   : InstrDescriptor(),
    # InstructionId.C_SF_D    : InstrDescriptor(),
    # InstructionId.C_NGLE_D  : InstrDescriptor(),
    # InstructionId.C_SEQ_D   : InstrDescriptor(),
    # InstructionId.C_NGL_D   : InstrDescriptor(),
    # InstructionId.C_LT_D    : InstrDescriptor(),
    # InstructionId.C_NGE_D   : InstrDescriptor(),
    # InstructionId.C_LE_D    : InstrDescriptor(),
    # InstructionId.C_NGT_D   : InstrDescriptor(),
    # InstructionId.CVT_S_W   : InstrDescriptor(),
    # InstructionId.CVT_D_W   : InstrDescriptor(),
    # InstructionId.CVT_S_L   : InstrDescriptor(),
    # InstructionId.CVT_D_L   : InstrDescriptor(),

    # InstructionId.DADDI     : InstrDescriptor(),
    # InstructionId.DADDIU    : InstrDescriptor(),
    # InstructionId.LDL       : InstrDescriptor(),
    # InstructionId.LDR       : InstrDescriptor(),

    # InstructionId.LB        : InstrDescriptor(),
    # InstructionId.LH        : InstrDescriptor(),
    # InstructionId.LWL       : InstrDescriptor(),
    # InstructionId.LW        : InstrDescriptor(),
    # InstructionId.LBU       : InstrDescriptor(),
    # InstructionId.LHU       : InstrDescriptor(),
    # InstructionId.LWR       : InstrDescriptor(),
    # InstructionId.LWU       : InstrDescriptor(),

    # InstructionId.SB        : InstrDescriptor(),
    # InstructionId.SH        : InstrDescriptor(),
    # InstructionId.SWL       : InstrDescriptor(),
    # InstructionId.SW        : InstrDescriptor(),
    # InstructionId.SDL       : InstrDescriptor(),
    # InstructionId.SDR       : InstrDescriptor(),
    # InstructionId.SWR       : InstrDescriptor(),
    # InstructionId.CACHE     : InstrDescriptor(),

    # InstructionId.LL        : InstrDescriptor(),
    # InstructionId.LWC1      : InstrDescriptor(),
    # InstructionId.LWC2      : InstrDescriptor(),
    # InstructionId.PREF      : InstrDescriptor(),
    # InstructionId.LLD       : InstrDescriptor(),
    # InstructionId.LDC1      : InstrDescriptor(),
    # InstructionId.LDC2      : InstrDescriptor(),
    # InstructionId.LD        : InstrDescriptor(),

    # InstructionId.SC        : InstrDescriptor(),
    # InstructionId.SWC1      : InstrDescriptor(),
    # InstructionId.SWC2      : InstrDescriptor(),
    #
    # InstructionId.SCD       : InstrDescriptor(),
    # InstructionId.SDC1      : InstrDescriptor(),
    # InstructionId.SDC2      : InstrDescriptor(),
    # InstructionId.SD        : InstrDescriptor(),

    # Pseudo-Instruction Unique IDs
    InstructionId.BEQZ      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.BNEZ      : InstrDescriptor("{rs}, ", "{IMM}", None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),
    InstructionId.B         : InstrDescriptor("{IMM}", None, None, InstrType.typeRegimm, isBranch=True, isBranchLikely=False, isTrap=False, isFloat=False, isDouble=False, isUnsigned=False, modifiesRt=False, modifiesRd=False),

    # InstructionId.NOP       : InstrDescriptor(),
    # InstructionId.MOVE      : InstrDescriptor(),
    # InstructionId.NEGU      : InstrDescriptor(),
    # InstructionId.NOT       : InstrDescriptor(),
}


InstructionsNotEmitedByIDO = {
    InstructionId.ADD,
    InstructionId.ADDI,
    InstructionId.MTC0,
    InstructionId.MFC0,
    InstructionId.ERET,
    InstructionId.TLBP,
    InstructionId.TLBR,
    InstructionId.TLBWI,
    InstructionId.CACHE,
}
