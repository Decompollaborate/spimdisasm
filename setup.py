#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from setuptools import setup # type: ignore

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="spimdisasm",
    version="1.1.6",
    author="Decompollaborate",
    license="MIT",
    url="https://github.com/Decompollaborate/spimdisasm",
    description="N64 MIPS disassembler",
    long_description=long_description,
    long_description_content_type="text/markdown",
)
