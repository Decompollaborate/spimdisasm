#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations


from .ElfObjDisasmInternals import getToolDescription, addOptionsToParser, getArgsParser, applyArgs, applyGlobalConfigurations, getOutputPath, getProcessedSections, changeGlobalSegmentRanges, insertSymtabIntoContext, insertDynsymIntoContext, injectAllElfSymbols, processGlobalOffsetTable, processArguments, addSubparser, elfObjDisasmMain
