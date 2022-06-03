#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

__version_info__ = (1, 1, 6)
__version__ = ".".join(map(str, __version_info__))
__author__ = "Decompollaborate"

from . import common
from . import elf32
from . import mips
