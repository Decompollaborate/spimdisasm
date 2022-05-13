#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse


class InstructionConfig:
    VR4300_COP0_NAMED_REGISTERS: bool = True
    VR4300_RSP_COP0_NAMED_REGISTERS: bool = True

    SN64_DIV_FIX: bool = False
    """Enables a few fixes for SN64's assembler related to div/divu instructions

    - SN64's assembler doesn't like assembling `div $0, a, b` with .set noat active.
    Removing the $0 fixes this issue.

    - SN64's assembler expands div to have break if dividing by zero
    However, the break it generates is different than the one it generates with `break N`
    So we replace break instrutions for SN64 with the exact word that the assembler generates when expanding div
    """

    @staticmethod
    def addParametersToArgParse(parser: argparse.ArgumentParser):
        mipsInstr = parser.add_argument_group("MIPS instructions configuration")

        mipsInstr.add_argument("--no-cop0-named-registers", help="Disables using the built-in names for registers of the VR4300's Coprocessor 0", action="store_true")
        mipsInstr.add_argument("--no-rsp-cop0-named-registers", help="Disables using the built-in names for registers of the RSP's Coprocessor 0", action="store_true")

        mipsInstr.add_argument("--sn64-div-fix", help="Enables a few fixes for SN64's assembler related to div/divu instructions", action="store_true")


    @classmethod
    def parseArgs(cls, args: argparse.Namespace):
        InstructionConfig.VR4300_COP0_NAMED_REGISTERS = not args.no_cop0_named_registers
        InstructionConfig.VR4300_RSP_COP0_NAMED_REGISTERS = not args.no_rsp_cop0_named_registers

        InstructionConfig.SN64_DIV_FIX = args.sn64_div_fix
