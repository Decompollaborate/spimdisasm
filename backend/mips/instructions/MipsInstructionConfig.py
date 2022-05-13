#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import argparse


class InstructionConfig:
    VR4300_COP0_NAMED_REGISTERS: bool = True
    VR4300_RSP_COP0_NAMED_REGISTERS: bool = True


    @staticmethod
    def addParametersToArgParse(parser: argparse.ArgumentParser):
        mipsInstr = parser.add_argument_group("MIPS instructions configuration")

        mipsInstr.add_argument("--no-cop0-named-registers", help="Disables using the built-in names for registers of the VR4300's Coprocessor 0", action="store_true")
        mipsInstr.add_argument("--no-rsp-cop0-named-registers", help="Disables using the built-in names for registers of the RSP's Coprocessor 0", action="store_true")


    @classmethod
    def parseArgs(cls, args: argparse.Namespace):
        InstructionConfig.VR4300_COP0_NAMED_REGISTERS = not args.no_cop0_named_registers
        InstructionConfig.VR4300_RSP_COP0_NAMED_REGISTERS = not args.no_rsp_cop0_named_registers
