#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022-2024 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import sys

import spimdisasm


if __name__ == "__main__":
    sys.exit(spimdisasm.frontendCommon.FrontendUtilities.cliMain())
