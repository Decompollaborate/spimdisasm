#!/usr/bin/env python3

from setuptools import find_packages, setup # type: ignore

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="spimdisasm",
    version="1.0.5",
    author="Decompollaborate",
    license="MIT",
    description="N64 MIPS disassembler",
    long_description=long_description,
    long_description_content_type="text/markdown",
)
