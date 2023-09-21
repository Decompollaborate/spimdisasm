## [Unreleased]

## [1.17.3] - 2023-09-18

### Uncategorized

- Hardcode a check to avoid disassembling `.vutext`
- Fix `.double` disassembly for little endian

## [1.17.2] - 2023-09-18

### Uncategorized

- Dummy update, sorry

## [1.17.1] - 2023-09-15

### Uncategorized

- Tweak endlabel to be emitted in the same place as the size directive does
- Emit global labels for symbols with no type in the middle of functions
- Add checks for rs and rt registers instead of assuming they are used to avoid crashes
- Option to set the filtering addresses for the symbol finder
- Consider the `j` instruction as a function end if rabbitizer's `--j-branch` option is turned off
- Always migrate mips1 doubles when migrating rodata

## [1.17.0] - 2023-08-27

### Uncategorized

- Allow using `MIPS_NONE` reloc type as a way to avoid symbolizing a reference and use the raw value instead.
- Allow using a different label for symbols in the middle of functions.
  - Useful for setting alternative entry points for handwritten functions.
  - It can be used by setting the `ASM_TEXT_ALT_LABEL`.
- Fix `elfObjDisasm` crashing if a reloc section references an unhandled section like `.pdr`.

## [1.16.5] - 2023-08-22

### Uncategorized

- Do not use iQue symbols by default if user asked for libultra symbols

## [1.16.4] - 2023-08-19

### Uncategorized

- Try to gather the `$gp` register value from non-PIP elfs too
- Various bugfixes related to `j` instructions being used as a way to call another function.
- Fix size directive not being properly emitted for functions with user-declared size that has dangling nops.
- Detect ABI and cpu flags from elf header.
- Do not use mips1 double detection heuristic on non o32 abis
- Avoid warning about LOCAL NOTYPE symbols in elf files

## [1.16.3] - 2023-08-15

### Uncategorized

- Fix hex comment crashing because of doubles when parsing little endian binaries

## [1.16.2] - 2023-08-14

### Uncategorized

- Fix size directive not using the right label when symbols are smaller than a word
- Fix size directive not being properly emitted for symbols with a size smaller than a word
- Generate pad symbols to honor user declared sizes
  - Symbols will be automatically splitted if the user-declared size is smaller than the symbol size (usually due to size not being a multiple of 4, file splits, other symbols not being referenced, etc)
- Add the character `0x1A` to set of special cases for string decoding.
- Fix bug which produced reporting incorrect file splits on strings which their last word was a zero.
- Workaround for big addends when building with modern GAS


## [1.16.0] - 2023-07-23

### Uncategorized

- Add a detector for the redundant function end produced by IDO with some specific flag combinations.
  - It is turned off by default, but it can be turned on globally with `--detect-redundant-function-end`, or globally and per file via the API.
- Fix BSS sections not emitting a first symbol if it isn't referenced anywhere.


## [1.15.4] - 2023-07-14

### Uncategorized

- Avoid taking into account invalid instructions when trying to find function boundaries.
- Properly honor size of user-declared symbols for elf static symbols.

## [1.15.3] - 2023-07-10

### Uncategorized

- Don't use append the section name if it is known when disassembling elfs
  - This special cases the sections `.text`, `.data`, `.rodata` and `.bss`.
  - Avoids the redundant `filename_.text/` naming scheme

## [1.15.2] - 2023-07-04

### Uncategorized

- Fix hardcoded shift value in alignment directive

## [1.15.1] - 2023-07-04

### Uncategorized

- Emit string alignment directives even when the section isn't aligned to a multiple of 8.
  - Some projects can have rodata sections aligned to just a multiple of 4, and not emitting the directive in those cases can break their builds

## [1.15.0] - 2023-07-03

### Uncategorized

- Change the string guesser to work with multiple levels instead of plainly enabled/disabled and the aggressive toggle.
  - The new option is used via the API `GlobalConfig.RODATA_STRING_GUESSER_LEVEL` or via the CLI `--rodata-string-guesser level`.
  - The old `GlobalConfig.STRING_GUESSER` and `GlobalConfig.AGGRESSIVE_STRING_GUESSER` options are now deprecated, same as the CLI options `--string-guesser` and `--aggressive-string-guesser`.
    - The old `GlobalConfig.STRING_GUESSER = True` is equivalent to the new `GlobalConfig.RODATA_STRING_GUESSER_LEVEL = 1`
    - The old `GlobalConfig.AGGRESSIVE_STRING_GUESSER = True` is equivalent to the new `GlobalConfig.RODATA_STRING_GUESSER_LEVEL = 4`
  - Meaning of the new levels:
    - level 0: Completely disable the guessing feature.
    - level 1: The most conservative guessing level. Imposes the following restrictions:
      - Do not try to guess if the user provided a type for the symbol.
      - Do no try to guess if type information for the symbol can be inferred by other means.
      - A string symbol must be referenced only once.
      - Strings must not be empty.
    - level 2: A string no longer needs to be referenced only once to be considered a possible string. This can happen because of a deduplication optimization.
    - level 3: Empty strings are allowed.
    - level 4: Symbols with autodetected type information but no user type information can still be guessed as strings.
  - The level defaults to 1.
- Implement string guesser for the data section.
  - Controlled by the API `GlobalConfig.DATA_STRING_GUESSER_LEVEL` or via the CLI `--data-string-guesser level`.
  - Decodes strings with the `ASCII` encoding by default.
  - The meaning of each level are the same as the rodata string guesser.
  - The level defaults to 2.
- Add experimental Pascal string guesser.
  - Works for both rodata and data sections.
  - Follows the same level logic as the C string guesser.
  - It is disabled by default.
- Start emitting `.size` directives by default.
- Emit `jlabel` instead of `dlabel` for jumptable labels by default
- Emit `dlabel` instead of `dlabel` for data, rodata and bss symbols by default

## [1.14.3] - 2023-06-19

### Uncategorized

- Failcheck for non aligned doubles
- (Hopefully) Fix same-vram overlays using symbols from other overlays
- `elfObjDisasm`: Can now disassemble sections with arbitrary names
- `disasmdis`: Disable pseudo instructions by default

## [1.14.2] - 2023-06-10

### Uncategorized

- Actually add `py.typed` to `pyproject.toml`
- Use `bytearray` as little as possible
- `writeBytearrayToFile` is now deprecated, use `writeBytesToFile` instead

## [1.14.1] - 2023-06-10

### Uncategorized

- Emit a previous alignment directive for strings.
  - Ensures strings are always word aligned
- Purge `.balign` directive in favor of `.align` directive
- Add `py.typed` file. Whoops

## [1.14.0] - 2023-05-10

### Uncategorized

- Try to support better N32 PIC programs.
  - The current issue was spimdisasm was not able to properly generate symbol references for `$gp` accesses.
  - GOT table now gets its own address from the reginfo instead of the dynamic table.
  - Accesses pointing outside the GOT table are tried to be redirected to `sdata`, `srdata` and `sbss` sections.
- Implement `--dyn-syms` on readelf-like mode.
- Minor improvements to readelf output format

## [1.13.3] - 2023-05-05

### Uncategorized

- Fix not writing to subfolders properly when a csv filesplit entry has a slash on its name.

## [1.13.2] - 2023-05-01

### Uncategorized

- Add support for `.dummy` section in csv file split format
- Add readelf's `--section-headers` flag to elfObjDisasm

## [1.13.1] - 2023-04-30

### Uncategorized

- Fix a possible `None` case in ` getInstrCategoryFromStr`
- Add note about R3000GTE and R5900 instruction set support in the README


## [1.13.0] - 2023-04-30

### Uncategorized

- Add support for R3000GTE


## [1.12.5] - 2023-04-28

### Uncategorized

- Fix jumptable end detection algorithm on vram ranges different than `0x80XXXXXX`
- Add `--function-info` flag to `elfObjDisasm`
- Option for emitting size directives in the generated assembly
- Add `--asm-emit-size-directive` flag to emit size directives on generated assembly

## [1.12.4] - 2023-04-19

### Uncategorized

- Fix user-declared relocs having an incorrect addend

## [1.12.3] - 2023-04-18

### Uncategorized

- Fix conflicting `-V` flag

## [1.12.2] - 2023-04-18

### Uncategorized

- Fix data symbols not using local reloc overrides
- Fix `.word`s not being updated after clearing pointers
- Some pointer clearing fixes
- Add `--version` flag to every cli tool
- Fix data not being properly disassembled on `singleFileDisasm`
- Enforce UTF-8 encoding on generated asm files (PR #111)
  - Thanks to @1superchip 
- Emit a comment saying if a reloc is a global one when the emit relocs flag is passed
- Fix incorrect addends on non static symbols from elf files.
  - Fixes issue #110 
- Fix a regression where some `%lo` symbols weren't being properly paired because of the `%got` being reused on PIC code
- Fix sizes for inferred types
- Properly detect `-mips1` `double` literals
  - Fixes issue #57 

## [1.12.1] - 2023-03-28

### Uncategorized

- Fix addends bigger than `0x7FFF` and smaller than `0x10000`

## [1.12.0] - 2023-03-21

### Uncategorized

- Now exposes known types to spimdisasm via `common.gKnownTypes`
- Prevents referencing labels and jumptable labels with addends
- Prevents referencing labels and jumptable labels in non jumptable symbols
- `static` (local) symbol handling of non relocated elf object files were improved
- Fake/non used symbols are not longer being emitted when disassembling elf .o files

## [1.11.6] - 2023-03-10

### Uncategorized

- Add flag to specify instruction category in `elfObjDisasm` and `singleFileDisasm`
- Remove `ContextSymbol.type` and add `ContextSymbol.userDeclaredType` and `ContextSymbol.autodetectedType`
  - A property named `.type` is available to provide backwards compatibility

## [1.11.5] - 2023-03-07

### Uncategorized

- Sort detected file boundaries and remove duplicates

## [1.11.4] - 2023-02-20

### Uncategorized

- Fix `--data-start` not processing hex correctly
- Add function vrom to `--function-info` and tweak its input a bit

## [1.11.3] - 2023-02-15

### Uncategorized

- Allow specifying a custom suffix to every autogenerated symbol with `--custom-suffix`
- Add "referenced functions" information to the `--function-info` flag

## [1.11.2] - 2023-02-13

### Uncategorized

- Add flag to emit inline relocs
- Do not report extra padding in functions if user declared size matches the size of the function
- Rename `ContextSymbol.size` to `ContextSymbol.userDeclaredSize`
- Add `--function-info` flag
- `FuncRodataEntry`: Fix migrate parameter if function has no rodata to be migrated

## [1.11.1] - 2023-01-30

### Uncategorized

- Allow `None` in `FunctionRodataEntry` methods
- `FuncRodataEntry`: Do not write `.section .text` if the function is `None`

## [1.11.0] - 2023-01-30

### Uncategorized

- CLI changes:
  - Install CLI tools as actual terminal programs
  - Allow invoking the CLI tools from spimdisasm as subparsers
  - The old way of invoking the CLI tools (`python3 -m spimdisasm.clitool`) is now deprecated, but still works
- `disasmdis`: Fix crash if the input isn't a multiple of a word
- Report with a comment which instruction made spimdisasm detected as a handwritten instruction
- New in the API: `FunctionRodataEntry`
  - Cleaner interface for rodata migration and similar functions
  - Provides method for intermixing functions and non-migrated rodata symbols in a way the correct order is still preserved
  - Old functions from `FileHandlers` which provided rodata migration functionalities are now deprecated

## [1.10.6] - 2023-01-28

### Uncategorized

- Fix some `.text` boundaries not being properly detected.
- Add hardware registers as constants so they are used by `lui`/`ori` pairs
- Check for bss symbol size to match user declared size
- Warn if the globalsegment vrom start and end is the same
- Identify 32bitsmode elf flag
- Avoid reporting leading zeroes as padding in rodata symbols if the size of the symbol matches the user declared one

## [1.10.5] - 2023-01-28

### Uncategorized

- Emit a comment on invalid instructions disassembled as words
- Remove redundant `.noreorder`
- Fix `disasmdis` ignoring endian parameter

## [1.10.4] - 2023-01-20

### Uncategorized

- Avoid trashing function analysis for `j` jumps outside of the function
- Add `EGCS` compiler
- `nop`s at the beginning of the files are now skipped.
- Fix `disasmdis` not properly accepting spaces
- Add iQue-specific libultra syms and hardware regs
- Add `--data-start` and `--data-end` flags to `singleFileDisasm`

## [1.10.3] - 2023-01-08

### Uncategorized

- Fix OoB for automatic type-based naming

## [1.10.2] - 2023-01-08

### Uncategorized

- Fix a small typo on `osAppNMIBuffer`

## [1.10.1] - 2023-01-05

### Uncategorized

- Adds a workaround for addends which does not fit on a 16 bits value

## [1.10.0] - 2023-01-05

### Uncategorized

- Rework system to allow/disallow addend references on data
- Add support for splat's symbol_addrs format for standalone invocations

## [1.9.2] - 2023-01-02

### Uncategorized

- Fix emitting `.align` directives on unnaligned jumptables
- Fix rodata split detection not properly considering special jumptable alignment
- Add `nameEnd` member to `ContextSymbol` to allow emitting a closing user-declared label

## [1.9.1] - 2022-12-29

### Uncategorized

- Emit a `.align 3` directive for every jumptable on non-IDO compilers

## [1.9.0] - 2022-12-28

### Uncategorized

- Reloc system re-worked. Users can now provide their own relocs to improve the automatic disassembly
- loPatch system has been removed and superseded by the global reloc system
- `GlobalConfig` variables can now be set via environment variables. 
  - Parameters passed by cli take priority over environment variables.
  - Options configured via code (when using this as a library) take priority over environment variables.


## [1.8.2] - 2022-12-19

### Uncategorized

- Check for banned symbols on addends references

## [1.8.1] - 2022-12-19

### Uncategorized

- New interface for allowing banning ranges of symbols, instead of having to add them one by one

## [1.8.0] - 2022-12-16

### Uncategorized

- Require [`rabbitizer` 1.4.0](https://github.com/Decompollaborate/rabbitizer/releases/tag/1.4.0)
- Allow to type-hint strings with `asciz`
- Allow disassembling `.data` symbols as strings
  - This won't be automatically guessed as with `.rodata`, this only will happen with type-hints
- `disasmdis` now accepts spaces and input from `stdin`

Meta:
- `setup.cfg` was removed and all its info was moved to `pyproject.toml`

## [1.7.12] - 2022-12-05

### Uncategorized

- Allow symbol references on rodata (for non jump-tables)
- Output version on disassembled files
- Add option to show which symbols reference the disassembled symbol
- Add `--file-splits` option to `elfObjDisasm`

## [1.7.11] - 2022-11-29

### Uncategorized

Check size of floats and doubles before migrating them

## [1.7.10] - 2022-11-26

### Uncategorized

- Allow changing the label used for jumptables labels with `GlobalConfig.ASM_JTBL_LABEL`
- Allow forcing (and forcing not to) migrate  a symbol on rodata migration

- Elf fixes:
  - Reference `NOTYPE` symbols
  - Various GOT fixes
  - Show isAutogeneratedPad in the context file #79 
  - [Show the first %lo reference for each symbol in the context #80
  

## [1.7.9] - 2022-11-09

### Uncategorized

- Fix an OoB issue when trying to post-process the GOT analyzis on non-PIC mode
- Emit a comment for automatically generated bss pads.
  - Those pads are created mainly to properly adjust the `.space` of a bss symbol if said symbol had an user-declared size

## [1.7.8] - 2022-11-04

### Uncategorized

- Fix function pointers being incorrectly tagged as `%call16` instead of `%got`
- Avoid crashing when trying to migrate functions when there's no rodata section
- Improve logic to disassemble `.byte`s and `.short`s
- Fix wrong migrated rodata on PIC programs
- Avoid using addends on function references
- Improve logic to find the jumptable ends (again)

## [1.7.7] - 2022-11-02

### Uncategorized

- Improve detection of the end of jumptables
- Refactor REL handling. It has been simplified
  - This should improve disassembling `.o` files
- Symbols from elfs are checked to be in the correct vram range before adding them to the context.
- Allow disassembling data symbols as floats and doubles


## [1.7.6] - 2022-10-31

### Uncategorized

- Refactor GOT handling
  - Should fix IDO 5.3 disassembly
-  Use glabels for jumptable labels when the functions are not being migrated
- Support `MIPS_GOT_HI16`, `MIPS_GOT_LO16`, `MIPS_CALL_HI16` and `MIPS_CALL_LO16` reloc types from `.rel` elf sections
- `disasmdis` now ignores non hex characters
- Negative addresses are considered as GOT accesses in PIC mode
- Add special handling for the GOT lazy resolver

## [1.7.5] - 2022-10-30

### Uncategorized

- Use `.gpword` on PIC jumptables
- Fix showing the got table in a few niche cases
- Use `glabel` on migrated rodata again until we figure out why it messes matching
- Do not use `glabel` on jumptable labels

## [1.7.4] - 2022-10-28

1.7.4: GOT fixes

### Uncategorized

GOT fixes:
- Fix using GOT local addresses as functions
- Fix `.data` symbols incorrectly referencing GOT local addresses
- Fix `elfObjDisasm` using N64 specific symbols by default
- Migrated rodata will no longer use glabels
  - This change was made to accommodate GOT global/local references

New features:
- The autodetected size of functions can now be queried from a `ContextSymbol` with `getSize()`
- The arch level can now be specified when disassembling
  - It is detected automatically when parsing an elf file
  - `.set gp=64` is disabled on MIPS1 and MIPS2 arch levels
- `elfObjDisasm` will now produce a list of functions and non migrated rodata when requesting to migrate functions.
  - This can be useful to bootstrap newly generated C files so the rodata order is easier to preserve

Misc changes:
- `elfObjDisasm` now display progress to stdout when disassembling
  - This behavior can be disabled with `-q`
- Rodata migration has been slightly tweaked: migration will be performed if only one function references the symbol, contrary to the old "only one reference in the whole codebase must reference the symbol"
  - Seems to be a common pattern on PIC programs compiled with IDO


## [1.7.3] - 2022-10-24

1.7.3: `--aggressive-string-guesser`

### Uncategorized

- Do not infer the type of a variable if the access types are heterogeneous, which may imply a struct
- New `--aggressive-string-guesser` flag
  - Tries to decode string even if the string is empty, the symbol may have type information or it is referenced more than once
- Add `PSYQ` compiler option
  - Currently it enables the same options as `SN64`

## [1.7.2] - 2022-10-24

### Uncategorized

- Allow passing context flags to `elfObjDisasm`
- Fix type inference if the user declared a type for the variable
- Fix some strings not being properly detected on elf files (again)

## [1.7.1] - 2022-10-23

1.7.1: elf fixing: got and dynamic programs

### Uncategorized

- New flags in `elfObjDisasm`:
  - Flags which try to mimic `readelf`: `--file-header`, `--syms`, `--relocs` and `-display-got`
  - `--split-functions`, has the same behavior as `singleFileDisasm`
- `elfObjDisasm` changes:
  - Fix undefined symbols handling in 
  - Fix addends of got global symbols
  - Use rel types types from the elf file if they are available instead of trying to infer them
  - Warn when trying to disassemble an `abi2` (N32) elf 
  - Warn for negative GOT accesses instead of crashing 
  - Warn if unhandled flags are found in an elf file
  - Fix gp value on N32 abi
  - `$gp` access are no longer symbolized if the address is not found in the got table
- Fix rodata pointer detection in data on elf files (fixes #63)

## [1.7.0] - 2022-10-18

### Uncategorized

- Add `leoBootID` to libultra syms
- Fix `--help` screen. whoops
- `--no-emit-cpload` flag to disable emitting the `.cpload` directive in PIC programs
  - `_gp_disp` is emitted instead of the raw immediate values
  - The `_gp_disp` value is emitted as a comment
- Updates `rabbitizer` requirement to 1.3.1
- Fix `.byte` and `.short` in little endian
  - Thanks @Xeeynamo (#62)
- Fix boundaries detection reporting in `elfObjDisasm`
  - The output of this report is csv-friendly. Thanks @EllipticEllipsis (#65)
- Tweak string disassembly
  - If a symbol is in the middle of a string then the string is aborted and disassembled as `.word`s
  - Check the next bytes (until a word boundary) after the nul terminator of a string are zero. Thanks @Xeeynamo (#64)
  - Strings with '\a' are no longer treated as real strings
- Fix labels not being emitted in rodata if they were not being word-aligned.
  - Fixes #59 
- Remove `GlobalConfig.ADD_NEW_SYMBOLS`
- Avoid pairing `%gp_got` symbols on non PIC code
- General cleanups

## [1.6.5] - 2022-10-07

1.6.5: elf fixes

### Uncategorized

- Avoid crashing if a `%got` access' address is not in the global table. Prints a warning instead.
- Fix symbol usage on relocatable files, kinda
- Ignore `.rel` sections if the file is not of REL type
- Avoid emitting `%got` rels for non PIC disassemblies

## [1.6.4] - 2022-10-06

 1.6.4: Fix symtab parsing from relocated elfs

### Uncategorized



## [1.6.3] - 2022-10-04

1.6.3: Speedup for overlay disassembly

### Uncategorized

- Moves the `globalSegment` check above all the other checks in `getSymbol`, providing a faster lookup since most of the time overlays usually reference a symbol from the `globalSegment` over a symbol from another overlay segment from a different category 

## [1.6.2] - 2022-10-03

1.6.2: hotfix

### Uncategorized

Should address issue #55 

## [1.6.1] - 2022-10-03

1.6.1: SN64 strikes again

### Uncategorized

- Add align directive for doubles for SN64
- Track which functions references each symbol
  - Used to improve rodata migration on non-IDO compilers
- Use `g` format specifier when formatting disassembled floats and doubles

## [1.6.0] - 2022-10-01

1.6.0: Refactor front-end scripts

### Uncategorized

- Deleted `singleFileDisasm.py`, `simpleFileDisasm.py`, `disasmdis.py`, `rspDisasm.py`, `elfObjDisasm.py`
- The frontend scripts were converted to submodules, now those can be executed with the `python3 -m spimdisasm.submodulename` syntax, allowing to execute them even in the pip installations of spimdisasm.
- Refactored raw path handling to use `pathlib`. 
- Add installation instructions to readme.
- SN64 tweaks:
  - Use `.align` directive for string disassembly on SN64
  - Migrate const variables to functions on SN64
  - Remove `.rdata` check on migrated rodata for SN64

## [1.5.7] - 2022-09-30

1.5.7: Fix 0 size bss variables

### Uncategorized

- Fixes an issue where 0 size bss variables where being outputted if the last bss variable of a file had a size which filled its size up until the file boundary



## [1.5.6] - 2022-09-27

1.5.6: More SN64 tweaks and data/rodata endianess

### Uncategorized

- Fixes data analyzis. It was ignoring banned symbols
- Use `.rdata` on rodata migration for SN64
- Fix `ASM_DATA_SYM_AS_LABEL` on bss generation
- Fix rodata symbols searching during migration
- Add option to disassemble data/rodata with different endianess than the global one
- Allow changing the string encoding per rodata segment
- Return the created segment by `addOverlaySegment`

## [1.5.5] - 2022-09-24

1.5.5: Fix `ASM_DATA_SYM_AS_LABEL`

### Uncategorized

- Fix `ASM_DATA_SYM_AS_LABEL`: It was outputing the data label

## [1.5.4] - 2022-09-24

1.5.4: More SN64 tweaks

### Uncategorized

- `ASM_DATA_SYM_AS_LABEL`: Allow adding a data symbol as a simple label.
- `late_rodata` logic has been tweaked to only be applied when compiler is set to IDO
- Allow range checks on symbols referenced by data symbols
- Add option to allow all addends referenced by data symbols
- Fix use of user-declared sizes on functions and bss symbols

## [1.5.3] - 2022-09-23

1.5.3: More default banned symbols

### Uncategorized

- Add `0x7FFFFFFF` to the list of default banned symbols

## [1.5.2] - 2022-09-21

1.5.2: elf endianess fixes and rodata migration fixes

### Uncategorized

- Read endianess from elf file
- Use `.section` directive on migrated rodata

## [1.5.1] - 2022-09-19

1.5.1: symbol detection fixes

### Uncategorized

- Check for data pointers in data itself and do a recheck in case the pointer is in the same section but behind the current symbol
- Properly update original symbol type when creating symbols for sub-4 sizes

## [1.5.0] - 2022-09-17

1.5.0: dynamic elfs

### Uncategorized

- Add compatibility for dynamic elf files
  - `.dynsym`, `.dynstr` and `.dynamic` section parsing
  - `.got` table parsing
  - Use `%got` and `%call16` syntax for `$gp` relative symbols.
  - `.cpload REG` detection
- Minor changes on string splitting functions to improve its use as an api

## [1.4.2] - 2022-09-09

1.4.2: PS2 addresses hotfix

### Uncategorized

Fix 0 and negative addresses being treated as real symbols

## [1.4.1] - 2022-09-03

1.4.1: Overlay function start bugfix

### Uncategorized

Fixes a bug in which a function start was wrongly detected on an overlay because the symbol existed for said address on the global segment

## [1.4.0] - 2022-08-27

1.4.0: RSP and R5900 support

### Uncategorized

- Adds proper support for N64's RSP
- Allows passing any `rabbitizer.InstrCategory` to SectionText to allow using any instruction set supported by rabbitizer

## [1.3.0] - 2022-07-08

1.3.0: rabbitizer 1.0.0

### Uncategorized

- Updated to use rabbitizer 1.0.0
- Rodata boundaries detection

## [1.2.4] - 2022-07-05

### Uncategorized

- Explicitly require a `rabbitizer` version minor than the next major version to avoid possible compatibility issues.
- Set up automatic discovery on setup.cfg


## [1.2.3] - 2022-06-11

1.2.3: Fix Python 3.7 compatibility (again)

### Uncategorized

This time really fixes 3.7 compat, hopefully

## [1.2.2] - 2022-06-11

1.2.2: %hi/%lo symbol filtering fix

### Uncategorized

Prevents filtering out LUI/ADDIU combos from being real symbols

## [1.2.1] - 2022-06-10

1.2.1: Packaging fix

### Uncategorized

Hopefully fixes a packaging problem on PyPi

## [1.2.0] - 2022-06-10

1.2.0: Faster disassembly from rabbitizer

### Uncategorized

- Use [rabbitizer](https://pypi.org/project/rabbitizer/) as the instruction decoder to speed up instruction analysis  and disassembly
- Other minor changes to try to be compatible with Python 3.7

## [1.1.7] - 2022-06-08

### Uncategorized

- Allow changing the file offset width comment
- `SortedDict`: New class which abstracts away the logic for keeping a sorted dictionary

## [1.1.6] - 2022-06-03

1.1.6: %hi reuse fix

### Uncategorized



## [1.1.5] - 2022-06-03

1.1.5: More overlay fixes

### Uncategorized

- Move most of the instruction analyzing code to a new independent class
- Try to track instructions which are likely to not be a %lo and avoid pairing it.
- Extra checks to avoid using labels from other overlay segments
- Only stop a look ahead search until we find either an unconditional branch or a `jr` instruction

## [1.1.4] - 2022-06-02

1.1.4: Overlay handling fixes

### Uncategorized

- Overall adds more vrom checks for handling shared-vram overlays.
- The autogenerated symbol size now adjusts itself considering the symbol type and the address alignment

## [1.1.3] - 2022-06-01

 1.1.3: Minor patch

### Uncategorized

Adds a way to register a name getter callback for symbols

## [1.1.2] - 2022-06-01

1.1.2: More symbol finding cleanups

### Uncategorized

- General cleanups and fixes related to symbol finding.
- New `COMPILER` option in `GlobalConfig`, used to enable some compiler specific tweaks

## [1.1.1] - 2022-05-30

1.1.1: Symbol finding fixes

### Uncategorized

Various fixes:
- Multiples workarounds for pairing multiples %hi to the same %lo
- Fix `J` target calculation for the look ahead symbol finder
- Special cases for `LUI`s on delay slots
- Track moving registers by using `MOVE`, `OR` and `ADDU`
- Invalidate some registers after function calls
- Check negative branches
- Fix jump table detector for SN64

## [1.1.0] - 2022-05-29

 1.1.0: Overlay support (hopefully)

### Uncategorized

- Add support for overlays which share VRAMs and overlays which may communicate to overlays from other categories.
  - Each new category and overlay segment must be registered in `Context`
- `ElementBase#vram` is now just an `int` as opposed to old `int|None`.
- The `vrom` of elements is tracked and stored. It is mainly used to autogenerate overlay's names.
- The differentiation in symbol categories (labels, functions, symbols, etc) is removed. Everything now is stored in only one big dictionary to avoid duplication and lower memory usage.
- Remove the concept of "fake functions"
- `addSymbol`, `getSymbol` and similar methods were moved to `ElementBase` because of the need for the respective element overlay info.
- Symbol name generation is delayed as much as possible and autogenerated on the fly.
- Some minor symbol finder fixes
- Bugfix symbols disappearing for no reason
- General cleanups

## [1.0.6] - 2022-05-26

1.0.6: Even more SN64 fixes

### Uncategorized

- Add more checks for J as unconditional branch
- Allow disabling %hi/%lo syntax for constants
- Add extra global label for other symbols (usually labels) in functions when `GlobalConfig.ASM_TEXT_FUNC_AS_LABEL` is enabled.

## [1.0.5] - 2022-05-26

### Uncategorized

- Allow setting a .ent and the function name as a label
- Float register `$31` fix for `NAMED_REGISTERS=False`
- Fix CI builds (2)
- Only apply the SN64 DIV fix for non handwritten functions
- Finer control over pseudo instructions


## [1.0.4] - 2022-05-26

### Uncategorized

Fix the CI wheel

## [1.0.3] - 2022-05-26

### Uncategorized

Add CI which builds a wheel and publishes it to PyPI

## [1.0.2] - 2022-05-26

### Uncategorized

- Fix missing `f` on float registers when `GlobalConfig.NAMED_REGISTERS` was set to `False`
- Allow changing the line ends to anything via `GlobalConfig.LINE_ENDS`

## [1.0.1] - 2022-05-26

### Uncategorized

Changes:
- Improve RAM usage by removing an almost unused dictionary from the Instruction classes

## [1.0.0] - 2022-05-26

### Uncategorized



[unreleased]: https://github.com/Decompollaborate/spimdisasm/compare/master...develop
[1.17.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.17.2...1.17.3
[1.17.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.17.1...1.17.2
[1.17.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.17.0...1.17.1
[1.17.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.16.5...1.17.0
[1.16.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.16.4...1.16.5
[1.16.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.16.3...1.16.4
[1.16.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.16.2...1.16.3
[1.16.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.16.0...1.16.2
[1.16.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.15.4...1.16.0
[1.15.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.15.3...1.15.4
[1.15.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.15.2...1.15.3
[1.15.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.15.1...1.15.2
[1.15.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.15.0...1.15.1
[1.15.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.14.3...1.15.0
[1.14.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.14.2...1.14.3
[1.14.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.14.1...1.14.2
[1.14.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.14.0...1.14.1
[1.14.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.13.3...1.14.0
[1.13.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.13.2...1.13.3
[1.13.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.13.1...1.13.2
[1.13.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.13.0...1.13.1
[1.13.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.12.5...1.13.0
[1.12.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.12.4...1.12.5
[1.12.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.12.3...1.12.4
[1.12.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.12.2...1.12.3
[1.12.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.12.1...1.12.2
[1.12.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.12.0...1.12.1
[1.12.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.6...1.12.0
[1.11.6]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.5...1.11.6
[1.11.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.4...1.11.5
[1.11.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.3...1.11.4
[1.11.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.2...1.11.3
[1.11.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.1...1.11.2
[1.11.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.11.0...1.11.1
[1.11.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.6...1.11.0
[1.10.6]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.5...1.10.6
[1.10.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.4...1.10.5
[1.10.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.3...1.10.4
[1.10.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.2...1.10.3
[1.10.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.1...1.10.2
[1.10.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.10.0...1.10.1
[1.10.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.9.2...1.10.0
[1.9.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.9.1...1.9.2
[1.9.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.9.0...1.9.1
[1.9.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.8.2...1.9.0
[1.8.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.8.1...1.8.2
[1.8.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.8.0...1.8.1
[1.8.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.12...1.8.0
[1.7.12]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.11...1.7.12
[1.7.11]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.10...1.7.11
[1.7.10]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.9...1.7.10
[1.7.9]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.8...1.7.9
[1.7.8]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.7...1.7.8
[1.7.7]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.6...1.7.7
[1.7.6]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.5...1.7.6
[1.7.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.4...1.7.5
[1.7.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.3...1.7.4
[1.7.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.2...1.7.3
[1.7.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.1...1.7.2
[1.7.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.7.0...1.7.1
[1.7.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.6.5...1.7.0
[1.6.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.6.4...1.6.5
[1.6.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.6.3...1.6.4
[1.6.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.6.2...1.6.3
[1.6.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.6.1...1.6.2
[1.6.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.6.0...1.6.1
[1.6.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.7...1.6.0
[1.5.7]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.6...1.5.7
[1.5.6]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.5...1.5.6
[1.5.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.4...1.5.5
[1.5.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.3...1.5.4
[1.5.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.2...1.5.3
[1.5.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.1...1.5.2
[1.5.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.5.0...1.5.1
[1.5.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.4.2...1.5.0
[1.4.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.4.1...1.4.2
[1.4.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.4.0...1.4.1
[1.4.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.3.0...1.4.0
[1.3.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.2.4...1.3.0
[1.2.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.2.3...1.2.4
[1.2.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.2.2...1.2.3
[1.2.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.2.1...1.2.2
[1.2.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.2.0...1.2.1
[1.2.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.7...1.2.0
[1.1.7]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.6...1.1.7
[1.1.6]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.5...1.1.6
[1.1.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.4...1.1.5
[1.1.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.3...1.1.4
[1.1.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.2...1.1.3
[1.1.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.1...1.1.2
[1.1.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.1.0...1.1.1
[1.1.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.6...1.1.0
[1.0.6]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.5...1.0.6
[1.0.5]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.4...1.0.5
[1.0.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.3...1.0.4
[1.0.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.2...1.0.3
[1.0.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.1...1.0.2
[1.0.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/Decompollaborate/spimdisasm/releases/tag/1.0.0
