# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `SectionText.setGpRelHack`: Allows updating the gpRelHack value after anlyzing
  the section.

### Changed

- Change where `.size` directive and end label is emitted on functions with no
  user-declared size.
  - The directives are now emitted before the function's trailing padding.

### Fixed

- `gpRelHack` fix:
  - Avoid emitting multiple `gp_rel:` comments.

## [1.32.4] - 2025-03-03

### Fixed

- `gpRelHack` fix:
  - Fix not emitting `.extern` directives for `$gp` accesses with addends.

## [1.32.3] - 2025-02-18

### Fixed

- Avoid emitting `Handwritten function` comment when comments are disabled.

## [1.32.2] - 2025-02-12

### Fixed

- Fix to avoid incorrectly inferring the symbol's type if the given symbol is
  referenced on complex control flows.
- Avoid symbolizing $gp accesses if the current function set that register to a
  different value.

## [1.32.1] - 2025-02-02

### Changed

- Tweak symbol pairing system a bit.
  - Should reduce the amount of fake pairings emitted in the generated assembly.

### Fixed

- Prevent emitting `jal`s to symbols that aren't functions.

## [1.32.0] - 2024-12-28

### Added

- Add `referencedVrams` member to `SymbolBase`.
  - Contains every vram that is referenced by the current symbol.

### Fixed

- Fix jumptable labels sometimes missing their rom suffix, again.

## [1.31.3] - 2024-12-21

### Fixed

- Fix a crash produced by not handling new `AccessType`s introduced by `rabbitizer`.
- `rabbitizer` 1.12.5 or above is required.

## [1.31.2] - 2024-12-02

### Fixed

- Fix jumptable labels sometimes missing their rom suffix.

## [1.31.1] - 2024-12-01

### Fixed

- Fix some jumptable labels missing their rom address on overlays.

## [1.31.0] - 2024-11-20

### Changed

- Improve comment specifying the reason why an address could not be symbolized
  if it is `$gp` relative.
- Prevent section split suggestions if the selected compiler doesn't follow the
  0x10 boundary rule.
- Rename `MWCC` compiler option to `MWCCPS2`.
- Python 3.9 or later is now required.
  - Nothing really changed. Just the CI tools I was using is refusing to use any
    Python version older than this. Sorry if you were affected by this.

## [1.30.2] - 2024-09-19

### Fixed

- Fix not generating branch labels under some circumstances.

## [1.30.1] - 2024-09-19

### Added

- Add a way to indentate instructions and data.
  - May be desirable to be used with IDEs that support collapsing code by
    looking at the whitespace of each line.
  - This can be controlled by setting the `GlobalConfig.ASM_INDENTATION` option.
  - Defaults to 4.
- Add a way to indentate labels within functions.
  - May be desirable to be used with IDEs that support collapsing code by
    looking at the whitespace of each line.
  - This can be controlled by setting the `GlobalConfig.ASM_INDENTATION_LABELS`
    option.
  - Defaults to 2.

### Changed

- Assembly now gets indentated by default to 4 spaces (or 2 spaces for labels).
  - Use `GlobalConfig.ASM_INDENTATION` and `GlobalConfig.ASM_INDENTATION_LABELS`
    to disable this behavior.

### Fixed

- Prevent generating labels for ignored symbols that are referenced by function
  calls (ie in a `jal`).

## [1.30.0] - 2024-09-10

### Changed

- Avoid migrating rodata symbols to functions if they are referenced by other
  data or rodata symbols.
- Disallow data and rodata symbols from referencing jumptables.
- BREAKING: Change the rodata migration algorithm.
  - This allows for the algorithm to migrate unreferenced symbols that are
    between other symbols that do get migrated to the given function.
  - The algorithm will now stop to migrate symbols as soon as it finds a symbol
    that should not be migrated to the current function (i.e. it should be
    migrated to other function, it is referenced by a data symbol, etc).
    - This could be an abrupt change for projects that were relying on the old
      migration scheme, because some symbols may suddenly disappear, avoiding a
      correct build.
    - This change should also reduce (and hopefully remove) the gaps generated
      between symbols during rodata migration.

### Deprecated

- Depreacte `SymbolBase.isRdata`.

### Fixed

- Fix pointer tracking: fix garbage state of registers after function jumping
  outside of the current function.

## [1.29.0] - 2024-09-09

### Added

- New `ContextSymbol.functionOwnerForMigration` attribute.
  - Allows to override to which function a given rodata symbol should be
    migrated to.
  - Specially useful for unreferenced symbols.
  - WARNING: It is undefined behavior if during rodata migration the listed
    function does not exists on the given text section. For example this symbol
    may get lost in limbo.

### Fixed

- Fix data references to symbols with adends (that have user declared sizes)
  being symbolized instead of just using an addend to the symbol.
  - This bug seems like was only happening if the referenced symbol was in the
    same section as the one who was referencing it.

## [1.28.1] - 2024-08-19

### Changed

- Avoid emitting "global" visibility on labels.
- Avoid emitting a rom offset comment on bss symbols.
- Change on `gpRelHack` behavior:
  - Emit `.extern`s with dummy size at the top of the function for all the
    `%gp_rel`-accessed symbols within the function.

### Fixed

- Fix function symbols and labels not acknowledging their parent file.
  - Used mainly for debugging purposes.

## [1.28.0] - 2024-08-09

### Added

- New `SectionText.gpRelHack` setting.
  - Turning it on changes all instructions that use a `%gp_rel` reloc into macro
    instructions that do not specify the relocation explicitly nor reference the
    `$gp` register.
  - This may even change some instruction mnemonics, like replacing `addiu` into
    `la`.
  - This is required by old assemblers that do not support explicit `%gp_rel`
    relocations, but instead they infer the relocation to be used by checking
    if the symbol was defined in the assembly file and its size fits on the
    passed `-G` parameter.
  - WARNING: It is the user's responsability to provide those symbol definitions
    to the assembler, otherwise those instructions will be expanded into
    multiple instructions and produce a shifted build.
- elfObjDisasm's readelf:
  - Add `MIPS_SCOMMON` and `MIPS_SUNDEFINED` support in symtab.
  - Use the section name in the ndx column instead of a plain number for
    `OBJECT`s and `FUNC`s.

### Changed

- Try to detect function pointers used on tail call optimizations and try to not
  confuse them with detected jumptables.
- rabbitizer 1.12.0 or above is required.

### Fixed

- Fix rodata addresses referenced _only_ by other rodata symbols on the same
  file not being properly symbolized.
- elfObjDisasm's readelf:
  - Fix name column not displaying the section's name.
  - Fix relocation sections not displaying anything on the name columns for
    relocations relative to a section instead of a symbol.

## [1.27.0] - 2024-07-10

### Added

- `gcc_except_table` support.
  - This section type is used by C++ to implement exceptions.
  - There's no automatic detection for this kind of symbol, so it must be
    manually specified by the user.
- Add two new `SymbolSpecialType`: `gccexcepttable` and `gccexcepttablelabel`.
  - `gccexcepttable` symbols will be prefixed with `ehtbl_`, while
    `gccexcepttablelabel` will have a `$LEH_` prefix.
- A new kind of customizable label type for ehtable labels:
  - `GlobalConfig.ASM_EHTBL_LABEL`: Allows to change the macro label for this
    specific kind of label.
  - Defaults to `ehlabel`.

## [1.26.1] - 2024-06-30

### Changed

- `singleFileDisasm`
  - Produce an error if input file does not exist instead of silently doing
    nothing at all.

### Fixed

- `elfObjDisasm`
  - Fix wrong capitalization on elf symbol visibility.
  - Fix symbol visibility not being used on some linked elfs.
- Fix some function pointers not being properly symbolized.
  - Those function pointers may get wrongly identified as jumptables because the
    jumptable pattern and the function pointer tail call pattern is similar.

## [1.26.0] - 2024-05-21

### Added

- Add `visibility` attribute to symbols.
  - Allows to specify custom visibility, like `weak` or `local`, to each symbol.
- Read symbol binding from elf files as `visibility` automatically.

## [1.25.1] - 2024-05-03

### Fixed

- Fix an edge case where strings may not get correctly disassembled when they
  contain the escape character (`\\`).
- Fix some overlay symbols not being properly created.

## [1.25.0] - 2024-04-22

### Added

- R4000 ALLEGREX instruction set compatibility
  - Used by the PlayStation Portable (PSP) CPU.
- `rabbitizer` 1.10.0 or above is required.

## [1.24.3] - 2024-04-04

### Changed

- Instruction analysis now follow all branches during look ahead analysis.
  - Look ahead analysis consists on taking branches and trying to follow the
    control flow without interrumping the main function analysis.
  - Old behavior trigered the look ahead analysis on a branch, but if a second
    branch was found during the look ahead analysis then it was ignored and not
    taken. This lead to some pointers not being properly paired if the codegen
    emitted the `%hi` and `%lo` separated by too many branches.
  - New behavior do follows branches during look ahead analysis, improving
    symbol pairing but making a bit slower the time needed for the overall
    analysis.
- rabbitizer 1.9.4 or above is required.
  - Should fix some issues under Windows.

## [1.24.2] - 2024-03-26

### Added

- `GlobalConfig.SYMBOL_ALIGNMENT_REQUIRES_ALIGNED_SECTION`.
  - Only emit symbol alignment directives if those are not larger than the
    alignment of the disassembled section.

## [1.24.1] - 2024-03-25

### Fixed

- Fix wrong function alignment for `EEGCC` compiler.

## [1.24.0] - 2024-03-25

### Added

- Add `EEGCC` compiler option.
- Add `KMC` compiler option.

### Changed

- Emit `.align 3` directives for strings for `EEGCC` compiler.
- Emit `.align 3` directives for functions for `EEGCC` compiler.
- Refactor `Compiler` to make implementing per-compiler differences more easily.

## [1.23.1] - 2024-03-22

### Fixed

- Fix rodata symbols referencing symbols that it should not reference.
  - Symbols like functions with addends, branch labels (not to be confused with
    jumptable labels), etc.

## [1.23.0] - 2024-03-19

### Added

- Symbols are now quoted if they contain special characters.
  - The symbol will be quote if any of the following characters in contained on
    the symbol name: `@`, `<`, `\\`, `-` or `+`.
- `FileBase.sectionFlags`: allows to specify flags for the specific section.
- `SectionBase.enableStringGuessing`: Allows to toggle the string guessing at
  the section level.
  - This option is ignored if the global string guessing option is disabled.
- `SectionBase.typeForOwnedSymbols`: Allows to specify that all the symbols from
  the current section should have the same type.
  - Does not override user declared types for symbols on the section.
- `SectionBase.sizeForOwnedSymbols`: Allows to specify that all the symbols from
  the current section should have the same size.
  - Does not override user declared sizes for symbols on the section.
- Add `MWCC` compiler option.

### Changed

- `SectionRelocZ64` now emits `"a"` flag as the section flag.
- Make sure to not reference local branch labels on data symbols.
- Minor code cleanups

## [1.22.0] - 2024-03-08

### Added

- New options:
  - Add `ASM_GLOBALIZE_TEXT_LABELS_REFERENCED_BY_NON_JUMPTABLE`.
    - Forces the use of `ASM_JTBL_LABEL` on text labels that are referenced by
      non jumptables.
    - Turned off by default.
      - Not enabled by default since this may be a symptom of something
        misconfigured or fake refences, since nothing besides jumptables should
        be able to reference labels inside a function.
- Add libleo (64DD) hardware registers to the list of known N64 hardware
  registers.

### Changed

- Change the logic for emitting the alignment directives.
  - Now the address of the symbol must be aligned relative to the file where
    the symbol is, instead of requiring both the symbol and the file to be
    aligned independently.
  - This should be specially useful for PSX projects, since the compiler aligns
    jumptables to 8, but it doesn't impose alignment restrictions on files,
    meaning the files (and those jumptables) may end up aligned to a lower
    alignment.
- Avoid filtering out user declared symbols when they are outside the valid
  address space.

### Fixed

- Fix missing type hints

## [1.21.0] - 2024-02-08

### Added

- Add `ContextSymbol.allowedToReferenceSymbols`.
  - Allow or prohibit this symbol to reference other symbols.
- Add `ContextSymbol.allowedToBeReferenced`
  - Allow or prohibit this symbol to be referenced by other symbols.
- Add `FunctionRodataEntry.getName` method.
- Add `FileBase.sectionAlignment`.
  - Allows to customize the alignment of the section.
- New options:
  - `--asm-prelude-use-includes` /
    `GlobalConfig.ASM_PRELUDE_USE_INCLUDES`: Emits the `.include` directive on
    the asm prelude.
  - `--asm-prelude-use-assembler-directives` /
    `GlobalConfig.ASM_PRELUDE_USE_INSTRUCTION_DIRECTIVES`: Emits instruction
    assembler directives.
  - `--asm-prelude-use-section-start` /
    `GlobalConfig.ASM_PRELUDE_USE_SECTION_START`: Emit the start of a section
    and alignment directive.

### Changed

- Data symbols: Emit the original word inside the emitted comment.
- Force `SectionRelocZ64`'s generated symbols to always be disassembled as
  `.word`s.
- Disallow `SectionRelocZ64`'s generated symbols to reference other symbols or
  to be referenced by other symbols.
- Autogenerated symbols from `SectionRelocZ64` will only use the stem of the
  section's filename.
- Assembler directives that only have meaning for instructions are only emitted
  on `.text` sections.

### Fixed

- Fix default string encoding for `.data` section.

## [1.20.1] - 2024-01-28

### Added

- Perform a range check on all `.data` symbols that have a user declared size.
  - This checks for the size of the symbol to not be smaller than the user
    declared size.
  - If the size doesn't match then a warning is printed to `stderr` or halt
    execution if `GlobalConfig.PANIC_RANGE_CHECK` is turned on.
- Add issue templates for bug reports and feature suggestions.
- Add `--rodata-string-encoding` and `--data-string-encoding` CLI options.

### Changed

- Format the range check message of `.bss` symbols similarly to how the `.data`
  range check is formatted.

### Fixed

- Fix missing comment terminator on assembly prelude.

## [1.20.0] - 2023-12-25

### Added

- Add `--name-vars-by-file` option to rename data symbols based on their offset
  and in their containing file.
- Add `--name-vars-by-section` considers the `.text` and `.ovl`/`.reloc` sections.
  - For unknown `.text` symbols (that are not functions, or any kind of label)
    the `T_` prefix will be used.
  - For `.ovl`/`.reloc` symbols the `REL_` prefix will be used

### Changed

- File splits can now contain reloc sections (`.ovl` or `.reloc`).
- Type-based name generation (`--name-vars-by-type`) can now be mixed with
  other kinds of name generations, allowing to give extra information on the
  symbol name.
- Autogenerated symbol names will not use a suffix if said symbol is generated
  after another symbol or file.
- Do not try to symbolize an address as a symbol plus addend if the original
  address is a banned address.
- Recognize `volatile` type variants as aliases for the non `volatile`
  versions.
  - For example, `vs16` is recognized as a `.short`.
- Use C style comments over `#` since that is more widely supperted.
  - For example Clang doesn't seem to support `#` comments.

## [1.19.0] - 2023-12-04

### Added

- Try to detect when instructions are setting the `$gp` register with the
  passed `GP_VALUE` and use the `_gp` symbol instead of symbolizing an
  unrelated symbol.
  - This check is only applied on non PIC mode.
- Track if a symbol is accessed with a `%gp_rel`.
- Add `--sequential-label-names` option to rename branch and jump table labels
  after their containing function (#142).
  - Thanks @cadmic

### Changed

- Emit which symbol produced the generation of an automatically generated and
  unused pad.
- Change how the variables csv file is parsed: If the size column is either
  empty, a dash (`-`) or zero then that column is ignored.
- Autogenerated symbol names are now padded up to 8 hex digits, contrary to the
  old 6 hex digits.
  - Since this may be a breaking change for some people the old functionallity
    can be restored by enabling the `GlobalConfig.LEGACY_SYM_ADDR_ZERO_PADDING`
    setting, or by passing `--legacy-sym-addr-zero-padding` when invoking a
    command line front-end.

### Deprecated

- `GlobalConfig.LEGACY_SYM_ADDR_ZERO_PADDING` (and the CLI version
  `--legacy-sym-addr-zero-padding`) are deprecated.
  - There's no replacement for this option.

### Fixed

- Avoid using the wrong immediate for gp-relative references if the address
  could not be symbolized.
  - In example, if said address is outside of the known address range.
- Fix some niche cases where spimdisasm may emit duplicated data symbol labels
  but without their data.

## [1.18.0] - 2023-10-29

### Added

- Add option to rename the sections generated by `FunctionRodataEntry`
  - `FunctionRodataEntry.sectionText` to rename the `.text` section
  - `FunctionRodataEntry.sectionRodata` to rename the `.rodata` section
  - `FunctionRodataEntry.sectionLateRodata` to rename the `.late_rodata` section

## [1.17.4] - 2023-10-07

### Added

- Add `CHANGELOG.md`

### Fixed

- Fix not decoding some valid Japense strings on `decodeBytesToStrings`
  - Specifically SHIFT-JIS ones with raw characters `0x8C` and `0x8D`

## [1.17.3] - 2023-09-18

### Changed

- Hardcodes a check to avoid disassembling `.vutext`. This will be changed in a
  future release.
  - Fixes `.vutext` sections from PS2 elfs messing with symbol analyzis

### Fixed

- Fix `.double` disassembly for little endian

## [1.17.2] - 2023-09-18

### Uncategorized

- Dummy update, sorry

## [1.17.1] - 2023-09-15

### Added

- Option to set the filtering addresses for the symbol finder

### Changed

- Tweak endlabel to be emitted in the same place as the size directive does
- Emit global labels for symbols with no type in the middle of functions
- Consider the `j` instruction as a function end if rabbitizer's `--j-branch`
  option is turned off
- Always migrate mips1 doubles when migrating rodata

### Fixed

- Add checks for `$rs` and `$rt` registers instead of assuming they are used to
  avoid crashing during runtime.

## [1.17.0] - 2023-08-27

### Changed

- Allow using `MIPS_NONE` reloc type as a way to avoid symbolizing a reference
  and use the raw value instead.
- Allow using a different label for symbols in the middle of functions.
  - Useful for setting alternative entry points for handwritten functions.
  - It can be used by setting the `ASM_TEXT_ALT_LABEL`.

### Fixed

- Fix `elfObjDisasm` crashing if a reloc section references an unhandled section
  like `.pdr`.

## [1.16.5] - 2023-08-22

### Changed

- Do not use iQue symbols by default if user asked only for libultra symbols

## [1.16.4] - 2023-08-19

### Added

- Try to gather the `$gp` register value from non-PIP elfs too
- Detect ABI and cpu flags from elf header.

### Changed

- Do not use mips1 double detection heuristic on non o32 abis

### Fixed

- Various bugfixes related to `j` instructions being used as a way to call
  another function.
- Fix size directive not being properly emitted for functions with user-declared
  size that has dangling nops.
- Avoid warning about `LOCAL` `NOTYPE` symbols in elf files

## [1.16.3] - 2023-08-15

### Fixed

- Fix hex comment crashing because of doubles when parsing little endian binaries

## [1.16.2] - 2023-08-14

### Added

- Generate pad symbols to honor user declared sizes
  - Symbols will be automatically splitted if the user-declared size is smaller
  than the symbol size (usually due to size not being a multiple of 4, file
  splits, other symbols not being referenced, etc)
- Add the character `0x1A` to set of special cases for string decoding.

### Changed

- Workaround for big addends when building with modern GAS

### Fixed

- Fix size directive not using the right label when symbols are smaller than a word
- Fix size directive not being properly emitted for symbols with a size smaller
  than a word
- Fix bug which produced reporting incorrect file splits on strings which their
  last word was a zero.

## [1.16.0] - 2023-07-23

### Added

- Add a detector for the redundant function end produced by IDO with some
  specific flag combinations.
  - It is turned off by default, but it can be turned on globally with
    `--detect-redundant-function-end`, or globally and per file via the API.

### Fixed

- Fix BSS sections not emitting a first symbol if it isn't referenced anywhere.

## [1.15.4] - 2023-07-14

### Changed

- Avoid taking into account invalid instructions when trying to find function boundaries.

### Fixed

- Properly honor size of user-declared symbols for elf static symbols.

## [1.15.3] - 2023-07-10

### Changed

- Don't append the section name if it is known when disassembling elfs
  - This special cases the sections `.text`, `.data`, `.rodata` and `.bss`.
  - Avoids the redundant `filename_.text/` naming scheme

## [1.15.2] - 2023-07-04

### Fixed

- Fix hardcoded shift value in alignment directive

## [1.15.1] - 2023-07-04

### Changed

- Emit string alignment directives even when the section isn't aligned to a
  multiple of 8.
  - Some projects can have rodata sections aligned to just a multiple of 4, and
    not emitting the directive in those cases can break their builds

## [1.15.0] - 2023-07-03

### Added

- Implement string guesser for the data section.
  - Controlled by the API `GlobalConfig.DATA_STRING_GUESSER_LEVEL` or via the
    CLI `--data-string-guesser level`.
  - Decodes strings with the `ASCII` encoding by default.
  - The meaning of each level are the same as the rodata string guesser.
  - The level defaults to 2.
- Add experimental Pascal string guesser.
  - Works for both rodata and data sections.
  - Follows the same level logic as the C string guesser.
  - It is disabled by default.

### Changed

- Change the string guesser to work with multiple levels instead of plainly
  enabled/disabled and the aggressive toggle.
  - The new option is used via the API
    `GlobalConfig.RODATA_STRING_GUESSER_LEVEL` or via the CLI
    `--rodata-string-guesser level`.
  - Meaning of the new levels:
    - level 0: Completely disable the guessing feature.
    - level 1: The most conservative guessing level. Imposes the following restrictions:
      - Do not try to guess if the user provided a type for the symbol.
      - Do no try to guess if type information for the symbol can be inferred by
        other means.
      - A string symbol must be referenced only once.
      - Strings must not be empty.
    - level 2: A string no longer needs to be referenced only once to be
      considered a possible string. This can happen because of a deduplication optimization.
    - level 3: Empty strings are allowed.
    - level 4: Symbols with autodetected type information but no user type
      information can still be guessed as strings.
  - The level defaults to 1.
- Start emitting `.size` directives by default.
- Emit `jlabel` instead of `dlabel` for jumptable labels by default
- Emit `dlabel` instead of `dlabel` for data, rodata and bss symbols by default

### Deprecated

- The old `GlobalConfig.STRING_GUESSER` and
  `GlobalConfig.AGGRESSIVE_STRING_GUESSER` options are now deprecated, same as
  the CLI options `--string-guesser` and `--aggressive-string-guesser`.
  - The old `GlobalConfig.STRING_GUESSER = True` is equivalent to the new
    `GlobalConfig.RODATA_STRING_GUESSER_LEVEL = 1`
  - The old `GlobalConfig.AGGRESSIVE_STRING_GUESSER = True` is equivalent to
    the new `GlobalConfig.RODATA_STRING_GUESSER_LEVEL = 4`

## [1.14.3] - 2023-06-19

### Added

- Failcheck for non aligned doubles

### Changed

- `elfObjDisasm`: Can now disassemble sections with arbitrary names
- `disasmdis`: Disable pseudo instructions by default

### Fixed

- (Hopefully) Fix same-vram overlays using symbols from other overlays

## [1.14.2] - 2023-06-10

### Changed

- Actually add `py.typed` to `pyproject.toml`
- Use `bytearray` as little as possible

### Deprecated

- `writeBytearrayToFile` is now deprecated, use `writeBytesToFile` instead

## [1.14.1] - 2023-06-10

### Added

- Emit a previous alignment directive for strings.
  - Ensures strings are always word aligned
- Add `py.typed` file. Whoops

### Changed

- Purge `.balign` directive in favor of `.align` directive

## [1.14.0] - 2023-05-10

### Added

- Implement `--dyn-syms` on readelf-like mode.

### Changed

- Improve a bit support for N32 PIC programs.
  - The current issue was spimdisasm was not able to properly generate symbol
    references for `$gp` accesses.
  - GOT table now gets its own address from the reginfo instead of the dynamic table.
  - Accesses pointing outside the GOT table are tried to be redirected to
    `sdata`, `srdata` and `sbss` sections.
- Minor improvements to readelf output format

## [1.13.3] - 2023-05-05

### Fixed

- Fix not writing to subfolders properly when a csv filesplit entry has a slash
  on its name.

## [1.13.2] - 2023-05-01

### Added

- Add support for `.dummy` section in csv file split format
- Add readelf's `--section-headers` flag to elfObjDisasm

## [1.13.1] - 2023-04-30

### Added

- Add note about R3000GTE and R5900 instruction set support in the README

### Fixed

- Fix a possible `None` case in `getInstrCategoryFromStr`

## [1.13.0] - 2023-04-30

### Added

- Add support for R3000GTE

## [1.12.5] - 2023-04-28

### Added

- Add `--function-info` flag to `elfObjDisasm`
- Option for emitting `.size`` directives in the generated assembly
  - Add `--asm-emit-size-directive` flag to emit size directives on generated assembly

### Fixed

- Fix jumptable end detection algorithm on vram ranges different than `0x80XXXXXX`

## [1.12.4] - 2023-04-19

### Fixed

- Fix user-declared relocs having an incorrect addend

## [1.12.3] - 2023-04-18

### Fixed

- Fix conflicting `-V` flag

## [1.12.2] - 2023-04-18

### Added

- Add `--version` flag to every cli tool
- Properly detect `-mips1` `double` literals
  - Fixes issue #57

### Changed

- Emit a comment saying if a reloc is a global one when the emit relocs flag is passed

### Fixed

- Fix data symbols not using local reloc overrides
- Fix `.word`s not being updated after clearing pointers
- Some pointer clearing fixes
- Fix data not being properly disassembled on `singleFileDisasm`
- Enforce UTF-8 encoding on generated asm files (PR #111)
  - Thanks to @1superchip
- Fix incorrect addends on non static symbols from elf files.
  - Fixes issue #110
- Fix a regression where some `%lo` symbols weren't being properly paired
  because of the `%got` being reused on PIC code
- Fix sizes for inferred types

## [1.12.1] - 2023-03-28

### Fixed

- Fix addends bigger than `0x7FFF` and smaller than `0x10000`

## [1.12.0] - 2023-03-21

### Added

- Expose known types to spimdisasm via `common.gKnownTypes`

### Changed

- Improve handling `static` (local) symbols for non relocated elf object files
- Fake/non used symbols are not longer emitted when disassembling elf `.o` files

### Fixed

- Prevents referencing labels and jumptable labels with addends
- Prevents referencing labels and jumptable labels in non jumptable symbols

## [1.11.6] - 2023-03-10

### Added

- Add flag to specify instruction category in `elfObjDisasm` and `singleFileDisasm`
- Add `ContextSymbol.userDeclaredType` and `ContextSymbol.autodetectedType`

### Deprecated

- Deprecate `ContextSymbol.type`
  - `.type` is kept as a property to provide backwards compatibility

## [1.11.5] - 2023-03-07

### Changed

- Sort detected file boundaries and remove duplicates

## [1.11.4] - 2023-02-20

### Changed

- Add function vrom to `--function-info` and tweak its input a bit

### Fixed

- Fix `--data-start` not processing hex correctly

## [1.11.3] - 2023-02-15

### Added

- Allow specifying a custom suffix to every autogenerated symbol with `--custom-suffix`
- Add "referenced functions" information to the `--function-info` flag

## [1.11.2] - 2023-02-13

### Added

- Add flag to emit inline relocs
- Add `ContextSymbol.userDeclaredSize`
- Add `--function-info` flag

### Changed

- Do not report extra padding in functions if user declared size matches the
  size of the function
- `FuncRodataEntry`: Fix migrate parameter if function has no rodata to be migrated

### Deprecated

- Deprecate `ContextSymbol.size`
  - It is kept as a property that wraps `ContextSymbol.userDeclaredSize`

## [1.11.1] - 2023-01-30

### Changed

- Allow `None` in `FunctionRodataEntry` methods
- `FuncRodataEntry`: Do not write `.section .text` if the function is `None`

## [1.11.0] - 2023-01-30

### Added

- Install CLI tools as actual terminal programs
- Allow invoking the CLI tools from spimdisasm as subparsers
- Report, with a comment, which instruction made spimdisasm detect a function as
  handwritten function
- New in the API: `FunctionRodataEntry`
  - Cleaner interface for rodata migration and similar functions
  - Provides method for intermixing functions and non-migrated rodata symbols in
    a way the correct order is still preserved

### Deprecated

- The old way of invoking the CLI tools (`python3 -m spimdisasm.clitool`) is now
  deprecated, but still works
- Old functions from `FileHandlers` which provided rodata migration
  functionalities are now deprecated in favour of the new `FunctionRodataEntry`.

### Fixed

- `disasmdis`: Fix crash if the input isn't a multiple of a word

## [1.10.6] - 2023-01-28

### Added

- Add hardware registers as constants so they are used by `lui`/`ori` pairs
- Check for bss symbol size to match user declared size
  - If the size doesn't match then a warning is printed to `stderr`
- Warn if the globalsegment's vrom start and end is the same
- Identify 32bitsmode elf flag

### Changed

- Avoid reporting leading zeroes as padding in rodata symbols if the size of the
  symbol matches the user declared one

### Fixed

- Fix some `.text` boundaries not being properly detected.

## [1.10.5] - 2023-01-28

### Added

- Emit a comment on invalid instructions disassembled as words

### Removed

- Remove redundant `.noreorder`

### Fixed

- Fix `disasmdis` ignoring endian parameter

## [1.10.4] - 2023-01-20

### Added

- Add `EGCS` compiler
- Add iQue-specific libultra syms and hardware regs
- Add `--data-start` and `--data-end` flags to `singleFileDisasm`

### Changed

- `nop`s at the beginning of the files are now skipped.

### Fixed

- Avoid trashing function analysis for `j` jumps outside of the function
- Fix `disasmdis` not properly accepting spaces

## [1.10.3] - 2023-01-08

### Fixed

- Fix OoB for automatic type-based naming

## [1.10.2] - 2023-01-08

### Fixed

- Fix a small typo on `osAppNMIBuffer`

## [1.10.1] - 2023-01-05

### Added

- Add a workaround for addends which does not fit on a 16 bits value

## [1.10.0] - 2023-01-05

### Added

- Add support for splat's symbol_addrs format for standalone invocations

### Changed

- Rework system to allow/disallow addend references on data

## [1.9.2] - 2023-01-02

### Added

- Add `nameEnd` member to `ContextSymbol` to allow emitting a closing
  user-declared label

### Fixed

- Fix emitting `.align` directives on unnaligned jumptables
- Fix rodata split detection not properly considering special jumptable alignment

## [1.9.1] - 2022-12-29

### Changed

- Emit a `.align 3` directive for every jumptable on non-IDO compilers

## [1.9.0] - 2022-12-28

### Added

- `GlobalConfig` variables can now be set via environment variables.
  - Parameters passed by cli take priority over environment variables.
  - Options configured via code (when using this as a library) take priority
    over environment variables.

### Changed

- Reloc system re-worked. Users can now provide their own relocs to improve the
  automatic disassembly

### Removed

- loPatch system has been removed and superseded by the global reloc system

## [1.8.2] - 2022-12-19

### Added

- Check for banned symbols on addends references

## [1.8.1] - 2022-12-19

### Added

- New interface for allowing banning ranges of symbols, instead of having to add
  them one by one

## [1.8.0] - 2022-12-16

### Added

- Allow to type-hint strings with `asciz`
- Allow disassembling `.data` symbols as strings
  - This won't be automatically guessed as with `.rodata`, this only will happen
    with type-hints

### Changed

- Require [`rabbitizer` 1.4.0](https://github.com/Decompollaborate/rabbitizer/releases/tag/1.4.0)
- `disasmdis` now accepts spaces and input from `stdin`

### Removed

- `setup.cfg` was removed and all its info was moved to `pyproject.toml`

## [1.7.12] - 2022-12-05

### Added

- Output version on disassembled files
- Add option to show which symbols reference the disassembled symbol
- Add `--file-splits` option to `elfObjDisasm`

### Changed

- Allow symbol references on rodata (for non jump-tables)

## [1.7.11] - 2022-11-29

### Changed

- Check size of floats and doubles before migrating them

## [1.7.10] - 2022-11-26

### Added

- Allow changing the label used for jumptables labels with `GlobalConfig.ASM_JTBL_LABEL`
- Allow forcing (and forcing not to) migrate a symbol on rodata migration
- Show `isAutogeneratedPad` in the context file (#79)
  - Thanks @simonlindholm
- Show the first `%lo` reference for each symbol in the context (#80)
  - Thanks @simonlindholm

### Fixed

- Reference `NOTYPE` symbols
- Various GOT fixes

## [1.7.9] - 2022-11-09

### Added

- Emit a comment for automatically generated bss pads.
  - Those pads are created mainly to properly adjust the `.space` of a bss
    symbol if said symbol had an user-declared size

### Fixed

- Fix an OoB issue when trying to post-process the GOT analyzis on non-PIC mode

## [1.7.8] - 2022-11-04

### Changed

- Improve logic to disassemble `.byte`s and `.short`s
- Improve logic to find the jumptable ends (again)

### Fixed

- Fix function pointers being incorrectly tagged as `%call16` instead of `%got`
- Avoid crashing when trying to migrate functions when there's no rodata section
- Fix wrong migrated rodata on PIC programs
- Avoid using addends on function references

## [1.7.7] - 2022-11-02

### Added

- Allow disassembling data symbols as floats and doubles

### Changed

- Improve detection of the end of jumptables
- Refactor REL handling. It has been simplified
  - This should improve disassembling `.o` files
- Symbols from elfs are checked to be in the correct vram range before adding
  them to the context.

## [1.7.6] - 2022-10-31

### Added

- Add special handling for the GOT lazy resolver

### Changed

- Refactor GOT handling
  - Should fix IDO 5.3 disassembly
- Use `glabel`s for jumptable labels when the functions are not being migrated
- Support `MIPS_GOT_HI16`, `MIPS_GOT_LO16`, `MIPS_CALL_HI16` and
  `MIPS_CALL_LO16` reloc types from `.rel` elf sections
- `disasmdis` now ignores non hex characters
- Negative addresses are considered as GOT accesses in PIC mode

## [1.7.5] - 2022-10-30

### Changed

- Use `.gpword` on PIC jumptables
- Use `glabel` on migrated rodata again until we figure out why it messes matching
- Do not use `glabel` on jumptable labels

### Fixed

- Fix showing the got table in a few niche cases

## [1.7.4] - 2022-10-28

1.7.4: GOT fixes

### Added

- The autodetected size of functions can now be queried from a `ContextSymbol`
  with `getSize()`
- The arch level can now be specified when disassembling
  - It is detected automatically when parsing an elf file
  - `.set gp=64` is disabled on MIPS1 and MIPS2 arch levels
- `elfObjDisasm` will now produce a list of functions and non migrated rodata
  when requesting to migrate functions.
  - This can be useful to bootstrap newly generated C files so the rodata order
    is easier to preserve

### Changed

- `elfObjDisasm` now display progress to stdout when disassembling
  - This behavior can be disabled with `-q`
- Rodata migration has been slightly tweaked: migration will be performed if
  only one function references the symbol, contrary to the old "only one
  reference in the whole codebase must reference the symbol"
  - Seems to be a common pattern on PIC programs compiled with IDO

### Fixed

- Fix using GOT local addresses as functions
- Fix `.data` symbols incorrectly referencing GOT local addresses
- Fix `elfObjDisasm` using N64 specific symbols by default
- Migrated rodata will no longer use glabels
  - This change was made to accommodate GOT global/local references

## [1.7.3] - 2022-10-24

1.7.3: `--aggressive-string-guesser`

### Added

- New `--aggressive-string-guesser` flag
  - Tries to decode string even if the string is empty, the symbol may have type
    information or it is referenced more than once
- Add `PSYQ` compiler option
  - Currently it enables the same options as `SN64`

### Changed

- Do not try to infer the type of a variable if the access types are heterogeneous,
  which may imply a struct

## [1.7.2] - 2022-10-24

### Added

- Allow passing context flags to `elfObjDisasm`

### Fixed

- Fix type inference if the user declared a type for the variable
- Fix some strings not being properly detected on elf files (again)

## [1.7.1] - 2022-10-23

1.7.1: elf fixing: got and dynamic programs

### Added

- `elfObjDisasm`: Flags which try to mimic `readelf`: `--file-header`, `--syms`,
  `--relocs` and `-display-got`
- `elfObjDisasm`: Add `--split-functions`. It has the same behavior as `singleFileDisasm`
- `elfObjDisasm`: Warn when trying to disassemble an `abi2` (N32) elf
- `elfObjDisasm`: Warn for negative GOT accesses instead of crashing
- `elfObjDisasm`: Warn if unhandled flags are found in an elf file

### Fixed

- `elfObjDisasm`. Fix undefined symbols handling in
- `elfObjDisasm`. Fix addends of got global symbols
- `elfObjDisasm`. Use rel types types from the elf file if they are available
  instead of trying to infer them
- `elfObjDisasm`: Fix gp value detection on N32 abi
- `elfObjDisasm`: `$gp` access are no longer symbolized if the address is not
  found in the got table
- Fix rodata pointer detection in data on elf files (fixes #63)

## [1.7.0] - 2022-10-18

### Added

- Add `leoBootID` to libultra syms
- `--no-emit-cpload` flag to disable emitting the `.cpload` directive in PIC programs
  - `_gp_disp` is emitted instead of the raw immediate values
  - The `_gp_disp` value is emitted as a comment

### Changed

- Updates `rabbitizer` requirement to 1.3.1
- Tweak string disassembly
  - If a symbol is in the middle of a string then the string is aborted and
    disassembled as `.word`s
  - Check the next bytes (until a word boundary) after the nul terminator of a
    string are zero. Thanks @Xeeynamo (#64)
  - Strings with '\a' are no longer treated as real strings
- Avoid pairing `%gp_got` symbols on non PIC code
- General cleanups

### Removed

- Remove `GlobalConfig.ADD_NEW_SYMBOLS`

### Fixed

- Fix `--help` screen. whoops
- Fix `.byte` and `.short` in little endian (#62)
  - Thanks @Xeeynamo
- Fix boundaries detection reporting in `elfObjDisasm` (#65)
  - The output of this report is csv-friendly.
  - Thanks @EllipticEllipsis
- Fix labels not being emitted in rodata if they were not being word-aligned.
  - Fixes #59

## [1.6.5] - 2022-10-07

1.6.5: elf fixes

### Fixed

- Avoid crashing if a `%got` access' address is not in the global table. Prints
  a warning instead.
- Fix symbol usage on relocatable files, kinda
- Ignore `.rel` sections if the file is not of REL type
- Avoid emitting `%got` rels for non PIC disassemblies

## [1.6.4] - 2022-10-06

1.6.4: Fix symtab parsing from relocated elfs

### Fixed

- Fix symtab parsing from relocated elfs

## [1.6.3] - 2022-10-04

1.6.3: Speedup for overlay disassembly

### Changed

- Moves the `globalSegment` check above all the other checks in `getSymbol`,
  providing a faster lookup since most of the time overlays usually reference a
  symbol from the `globalSegment` over a symbol from another overlay segment
  from a different category

## [1.6.2] - 2022-10-03

1.6.2: hotfix

### Fixed

- Fix unhashable type crash
  - Fixes #55

## [1.6.1] - 2022-10-03 [YANKED]

1.6.1: SN64 strikes again

### Added

- Add align directive for doubles for SN64
- Track which functions references each symbol
  - Used to improve rodata migration on non-IDO compilers

### Changed

- Use `g` format specifier when formatting disassembled floats and doubles

## [1.6.0] - 2022-10-01

1.6.0: Refactor front-end scripts

### Changed

- The frontend scripts were converted to submodules, now those can be executed
  with the `python3 -m spimdisasm.submodulename` syntax, allowing to execute
  them even in the pip installations of spimdisasm.
- Refactored raw path handling to use `pathlib`.
- Add installation instructions to readme.
- SN64 tweaks:
  - Use `.align` directive for string disassembly on SN64
  - Migrate const variables to functions on SN64
  - Remove `.rdata` check on migrated rodata for SN64

### Removed

- Deleted `singleFileDisasm.py`, `simpleFileDisasm.py`, `disasmdis.py`,
  `rspDisasm.py`, `elfObjDisasm.py`

## [1.5.7] - 2022-09-30

1.5.7: Fix 0 size bss variables

### Fixed

- Fixes an issue where 0 size bss variables where being outputted if the last
  bss variable of a file had a size which filled its size up until the file boundary

## [1.5.6] - 2022-09-27

1.5.6: More SN64 tweaks and data/rodata endianess

### Added

- Add option to disassemble data/rodata with different endianess than the global
  one
- Allow changing the string encoding per rodata segment

### Changed

- Use `.rdata` on rodata migration for SN64

### Fixed

- Fixes data analyzis.
  - It was ignoring banned symbols
- Fix `ASM_DATA_SYM_AS_LABEL` on bss generation
- Fix rodata symbols searching during migration
- Return the created segment by `addOverlaySegment`

## [1.5.5] - 2022-09-24

1.5.5: Fix `ASM_DATA_SYM_AS_LABEL`

### Fixed

- Fix `ASM_DATA_SYM_AS_LABEL`: It was outputing the data label

## [1.5.4] - 2022-09-24

1.5.4: More SN64 tweaks

### Added

- `ASM_DATA_SYM_AS_LABEL`: Allow adding a data symbol as a simple label.
- Allow range checks on symbols referenced by data symbols
- Add option to allow all addends referenced by data symbols

### Changed

- `late_rodata` logic has been tweaked to only be applied when compiler is set
  to IDO

### Fixed

- Fix use of user-declared sizes on functions and bss symbols

## [1.5.3] - 2022-09-23

1.5.3: More default banned symbols

### Changed

- Add `0x7FFFFFFF` to the list of default banned symbols

## [1.5.2] - 2022-09-21

1.5.2: elf endianess fixes and rodata migration fixes

### Changed

- Read endianess from elf file
- Use `.section` directive on migrated rodata

## [1.5.1] - 2022-09-19

1.5.1: symbol detection fixes

### Changed

- Check for data pointers in data itself and do a recheck in case the pointer is
  in the same section but behind the current symbol
- Properly update original symbol type when creating symbols for sub-4 sizes

## [1.5.0] - 2022-09-17

1.5.0: dynamic elfs

### Added

- Add compatibility for dynamic elf files
  - `.dynsym`, `.dynstr` and `.dynamic` section parsing
  - `.got` table parsing
  - Use `%got` and `%call16` syntax for `$gp` relative symbols.
  - `.cpload REG` detection

### Changed

- Minor changes on string splitting functions to improve its use as an api

## [1.4.2] - 2022-09-09

1.4.2: PS2 addresses hotfix

### Fixed

- Fix 0 and negative addresses being treated as real symbols

## [1.4.1] - 2022-09-03

1.4.1: Overlay function start bugfix

### Fixed

- Fixes a bug in which a function start was wrongly detected on an overlay because
  the symbol existed for said address on the global segment

## [1.4.0] - 2022-08-27

1.4.0: RSP and R5900 support

### Added

- Adds proper support for N64's RSP
- Allows passing any `rabbitizer.InstrCategory` to SectionText to allow using
  any instruction set supported by `rabbitizer`

## [1.3.0] - 2022-07-08

1.3.0: rabbitizer 1.0.0

### Added

- Rodata boundaries detection

### Changed

- Updated to use rabbitizer 1.0.0

## [1.2.4] - 2022-07-05

### Changed

- Explicitly require a `rabbitizer` version smaller than the next major version to
  avoid possible compatibility issues.
- Set up automatic discovery on `setup.cfg`

## [1.2.3] - 2022-06-11

1.2.3: Fix Python 3.7 compatibility (again)

### Fixed

- This time really fixes 3.7 compat, hopefully

## [1.2.2] - 2022-06-11

1.2.2: %hi/%lo symbol filtering fix

### Fixed

- Prevents filtering out `lui`/`addiu` combos from being real symbols

## [1.2.1] - 2022-06-10

1.2.1: Packaging fix

### Fixed

- Hopefully fixes a packaging problem on PyPi

## [1.2.0] - 2022-06-10 [YANKED]

1.2.0: Faster disassembly from rabbitizer

### Changed

- Use [rabbitizer](https://pypi.org/project/rabbitizer/) as the instruction
  decoder to speed up instruction analysis and disassembly
- Other minor changes to try to be compatible with Python 3.7

## [1.1.7] - 2022-06-08

### Changed

- Allow changing the file offset width comment
- `SortedDict`: New class which abstracts away the logic for keeping a sorted dictionary

## [1.1.6] - 2022-06-03

1.1.6: %hi reuse fix

### Fixed

- Fix `%hi` reuse with wrong values

## [1.1.5] - 2022-06-03

1.1.5: More overlay fixes

### Changed

- Move most of the instruction analyzing code to a new independent class
- Try to track instructions which are likely to not be a `%lo` and avoid pairing
  it.
- Extra checks to avoid using labels from other overlay segments
- Only stop a look ahead search until we find either an unconditional branch or
  a `jr` instruction

## [1.1.4] - 2022-06-02

1.1.4: Overlay handling fixes

### Changed

- Overall adds more vrom checks for handling shared-vram overlays.
- The autogenerated symbol size now adjusts itself considering the symbol type
  and the address alignment

## [1.1.3] - 2022-06-01

1.1.3: Minor patch

### Added

- Adds a way to register a name getter callback for symbols

## [1.1.2] - 2022-06-01

1.1.2: More symbol finding cleanups

### Added

- New `COMPILER` option in `GlobalConfig`, used to enable some compiler
  specific tweaks

### Fixed

- General cleanups and fixes related to symbol finding.

## [1.1.1] - 2022-05-30

1.1.1: Symbol finding fixes

### Fixed

- Multiples workarounds for pairing multiples %hi to the same %lo
- Fix `j` target calculation for the look ahead symbol finder
- Special cases for `lui`s on delay slots
- Track moving registers by using `move`, `or` and `addu`
- Invalidate some registers after function calls
- Check negative branches
- Fix jump table detector for SN64

## [1.1.0] - 2022-05-29

1.1.0: Overlay support (hopefully)

### Added

- Add support for overlays which share VRAMs and overlays which may communicate
  to overlays from other categories.
  - Each new category and overlay segment must be registered in `Context`
- The `vrom` of elements is tracked and stored. It is mainly used to
  autogenerate overlay's names.
- The categorization in symbol categories (labels, functions, symbols, etc) is
  removed. Everything now is stored in only one big dictionary to avoid
  duplication and lower memory usage.

### Changed

- `ElementBase#vram` is now just an `int` as opposed to old `int|None`.
- `addSymbol`, `getSymbol` and similar methods were moved to `ElementBase`
  because of the need for the respective element overlay info.
- Symbol name generation is delayed as much as possible and autogenerated on
  the fly.
- General cleanups

### Removed

- Remove the concept of "fake functions"

### Fixed

- Some minor symbol finder fixes
- Gix symbols disappearing for no reason

## [1.0.6] - 2022-05-26

1.0.6: Even more SN64 fixes

### Added

- Add more checks for `j` as unconditional branch
- Allow disabling `%hi`/`%lo` syntax for constants
- Add extra global label for other symbols (usually labels) in functions when
  `GlobalConfig.ASM_TEXT_FUNC_AS_LABEL` is enabled.

## [1.0.5] - 2022-05-26

### Added

- Allow setting a `.ent` and the function name as a label
- Float register `$31` fix for `NAMED_REGISTERS=False`

### Changed

- Only apply the SN64 DIV fix for non handwritten functions
- Finer control over pseudo instructions

### Fixed

- Fix CI builds (2)

## [1.0.4] - 2022-05-26 [YANKED]

### Fixed

- Fix the CI wheel

## [1.0.3] - 2022-05-26 [YANKED]

### Changed

- Add CI which builds a wheel and publishes it to PyPI

## [1.0.2] - 2022-05-26

### Added

- Allow changing the line ends to anything via `GlobalConfig.LINE_ENDS`

### Fixed

- Fix missing `f` on float registers when `GlobalConfig.NAMED_REGISTERS` was set
  to `False`

## [1.0.1] - 2022-05-26

### Changed

- Improve RAM usage by removing an almost unused dictionary from the Instruction
  classes

## [1.0.0] - 2022-05-26

### Added

- Version 1.0.0

[unreleased]: https://github.com/Decompollaborate/spimdisasm/compare/master...develop
[1.32.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.32.3...1.32.4
[1.32.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.32.2...1.32.3
[1.32.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.32.1...1.32.2
[1.32.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.32.0...1.32.1
[1.32.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.31.3...1.32.0
[1.31.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.31.2...1.31.3
[1.31.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.31.1...1.31.2
[1.31.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.31.0...1.31.1
[1.31.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.30.2...1.31.0
[1.30.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.30.1...1.30.2
[1.30.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.30.0...1.30.1
[1.30.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.29.0...1.30.0
[1.29.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.28.1...1.29.0
[1.28.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.28.0...1.28.1
[1.28.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.27.0...1.28.0
[1.27.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.26.1...1.27.0
[1.26.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.26.0...1.26.1
[1.26.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.25.1...1.26.0
[1.25.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.25.0...1.25.1
[1.25.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.24.3...1.25.0
[1.24.3]: https://github.com/Decompollaborate/spimdisasm/compare/1.24.2...1.24.3
[1.24.2]: https://github.com/Decompollaborate/spimdisasm/compare/1.24.1...1.24.2
[1.24.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.24.0...1.24.1
[1.24.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.23.1...1.24.0
[1.23.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.23.0...1.23.1
[1.23.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.22.0...1.23.0
[1.22.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.21.1...1.22.0
[1.21.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.20.1...1.21.0
[1.20.1]: https://github.com/Decompollaborate/spimdisasm/compare/1.20.0...1.20.1
[1.20.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.19.0...1.20.0
[1.19.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.18.0...1.19.0
[1.18.0]: https://github.com/Decompollaborate/spimdisasm/compare/1.17.4...1.18.0
[1.17.4]: https://github.com/Decompollaborate/spimdisasm/compare/1.17.3...1.17.4
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
