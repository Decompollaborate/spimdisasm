#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse
import enum


class AbiNames(enum.Enum):
    numeric = enum.auto()
    o32 = enum.auto()
    n32 = enum.auto()
    n64 = enum.auto()

    @staticmethod
    def fromStr(x: str) -> AbiNames:
        if x in ("32", "o32"):
            return AbiNames.o32
        if x in ("n32",):
            return AbiNames.n32
        if x in ("64", "n64"):
            return AbiNames.n64
        return AbiNames.numeric


class InstructionConfig:
    NAMED_REGISTERS: bool = True

    GPR_ABI_NAMES: AbiNames = AbiNames.o32
    # FPR_ABI_NAMES: AbiNames = AbiNames.numeric

    VR4300_COP0_NAMED_REGISTERS: bool = True
    VR4300_RSP_COP0_NAMED_REGISTERS: bool = True

    PSEUDO_INSTRUCTIONS: bool = True
    """Produce pseudo instructions (like `move`, `nop` or `b`) whenever those should match the desired original instruction"""

    SN64_DIV_FIX: bool = False
    """Enables a few fixes for SN64's assembler related to div/divu instructions

    - SN64's assembler doesn't like assembling `div $0, a, b` with .set noat active.
    Removing the $0 fixes this issue.

    - SN64's assembler expands div to have break if dividing by zero
    However, the break it generates is different than the one it generates with `break N`
    So we replace break instrutions for SN64 with the exact word that the assembler generates when expanding div
    """

    OPCODE_LJUST: int = 7+4
    """The minimal number of characters to left-align the opcode name"""

    UNKNOWN_INSTR_COMMENT: bool = True

    @staticmethod
    def addParametersToArgParse(parser: argparse.ArgumentParser):
        mipsInstr = parser.add_argument_group("MIPS instructions configuration")

        mipsInstr.add_argument("--no-named-registers", help="Disables named registers for every instruction. This flag takes precedence over other similar flags", action="store_true")

        abi_choices = ["numeric", "32", "o32", "n32", "n64"]
        mipsInstr.add_argument("--Mgpr-names", help=f"Use GPR names according to the specified ABI. Defaults to {InstructionConfig.GPR_ABI_NAMES.name}", choices=abi_choices)

        mipsInstr.add_argument("--no-cop0-named-registers", help="Disables using the built-in names for registers of the VR4300's Coprocessor 0", action="store_true")
        mipsInstr.add_argument("--no-rsp-cop0-named-registers", help="Disables using the built-in names for registers of the RSP's Coprocessor 0", action="store_true")

        mipsInstr.add_argument("--no-pseudo-instr", help=f"Disables producing pseudo instructions. Defaults to {InstructionConfig.PSEUDO_INSTRUCTIONS}", action="store_true")

        mipsInstr.add_argument("--sn64-div-fix", help="Enables a few fixes for SN64's assembler related to div/divu instructions", action="store_true")


        miscOpts = mipsInstr.add_argument_group("Misc options")

        miscOpts.add_argument("--opcode-ljust", help=f"Set the minimal number of characters to left-align the opcode name. Defaults to {InstructionConfig.OPCODE_LJUST}")

        miscOpts.add_argument("--no-unk-instr-comment", help=f"Disables the extra comment produced after unknown instructions. Defaults to {InstructionConfig.UNKNOWN_INSTR_COMMENT}", action="store_true")


    @classmethod
    def parseArgs(cls, args: argparse.Namespace):
        InstructionConfig.NAMED_REGISTERS = not args.no_named_registers

        if args.Mgpr_names:
            InstructionConfig.GPR_ABI_NAMES = AbiNames.fromStr(args.Mgpr_names)

        InstructionConfig.VR4300_COP0_NAMED_REGISTERS = not args.no_cop0_named_registers
        InstructionConfig.VR4300_RSP_COP0_NAMED_REGISTERS = not args.no_rsp_cop0_named_registers

        InstructionConfig.PSEUDO_INSTRUCTIONS = not args.no_pseudo_instr

        InstructionConfig.SN64_DIV_FIX = args.sn64_div_fix

        if args.opcode_ljust is not None:
            InstructionConfig.OPCODE_LJUST = int(args.opcode_ljust)

        InstructionConfig.UNKNOWN_INSTR_COMMENT = not args.no_unk_instr_comment
