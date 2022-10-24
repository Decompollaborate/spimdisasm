# spimdisasm

[![PyPI - Downloads](https://img.shields.io/pypi/dm/spimdisasm)](https://pypi.org/project/spimdisasm/)
[![GitHub License](https://img.shields.io/github/license/Decompollaborate/spimdisasm)](https://github.com/Decompollaborate/spimdisasm/releases/latest)
[![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/Decompollaborate/spimdisasm)](https://github.com/Decompollaborate/spimdisasm/releases/latest)
[![PyPI](https://img.shields.io/pypi/v/spimdisasm)](https://pypi.org/project/spimdisasm/)
[![GitHub contributors](https://img.shields.io/github/contributors/Decompollaborate/spimdisasm?logo=purple)](https://github.com/Decompollaborate/spimdisasm/graphs/contributors)

A matching MIPS disassembler API and front-ends with built-in instruction analysis.

Currently supports all the CPU instructions for MIPS I, II, III and IV.

Mainly focused on supporting Nintendo 64 binaries, but it should work with other MIPS platforms too.

## Installing

The recommended way to install is using from the PyPi release, via `pip`:

```bash
pip install spimdisasm
```

In case you want to mess with the latest development version without wanting to clone the repository, then you could use the following command:

```bash
pip uninstall spimdisasm
pip install git+https://github.com/Decompollaborate/spimdisasm.git@develop
```

NOTE: Installing the development version is not recommended. Proceed at your own risk.

## Features

- Produces matching assembly.
- Supports `.text`, `.data`, `.rodata` and `.bss` disassembly.
  - The reloc section from Zelda 64 and some other games is supported too, but no front-end script uses it yet.
- Generates separated files for each section of a file (`.text`, `.data`, `.rodata` and `.bss`).
- Supports multiple files spliting from a single input binary.
- Automatic function detection.
  - Can detect if a function is handwritten too.
- `hi`/`lo` pairing with high success rate.
- Automatic pointer and symbol detection.
- Function spliting with rodata migration.
- Supports floats and doubles in rodata.
- String detection with medium to high success rate.
- Allows to set user-defined function and symbol names.
- Big, little and middle endian support.
- Autogenerated symbols can be named after the section they come from (`RO_` and `B_` for `.rodata` and `.bss` sections) or its type (`STR_`, `FLT_` and `DBL_` for string, floats and doubles respectively).
- Simple file boundary detection.
  - Detects boundaries on .text and .rodata sections
- Lots of features can be turned on and off.
- MIPS instructions features:
  - Named registers for MIPS VR4300's coprocessors.
  - Support for many pseudoinstructions.
  - Properly handle move to/from coprocessor instructions.
  - Support for numeric, o32, n32 and n64 ABI register names.
- Some workarounds for some specific compilers/assemblers:
  - `SN64`/`PSYQ`:
    - `div`/`divu` fix: tweaks a bit the produced `div`, `divu` and `break` instructions.
- N64 RSP instruction disassembly support.
  - RSP decoding has been tested to build back to matching assemblies with [armips](https://github.com/Kingcom/armips/).
- (Experimental) Same VRAM overlay support.
  - Overlays which are able to reference symbols from other overlays in other categories/types is supported too.
  - NOTE: This feature lacks lots of testing and probably has many bugs.

## How to use

This repo can be used either by using the existing front-end scripts or by creating new programs on top of the back-end API.

### Front-end

Every front-end submodule has its own `--help` screen.

The submodules can be executed with `python3 -m spimdisasm.modulename`, for example `python3 -m spimdisasm.disasmdis`

- `singleFileDisasm`: Allows to disassemble a single binary file, producing matching assembly files.

- `disasmdis`: Disassembles raw hex passed to the CLI as a MIPS instruction.

- `elfObjDisasm`: \[EXPERIMENTAL\] Allows to disassemble `.o` elf files. Generated assembly files are not guaranteed to match or be assemblable.

- `rspDisasm`: Disassemblies RSP binaries.

### Back-end

TODO

Check `spimdisasm/__main__.py` for a minimal disassembly working example on how to use the API. Checking the front-ends is recommended too.
