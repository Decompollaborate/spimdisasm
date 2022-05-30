#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ... import common

from .. import instructions

from . import SymbolText


class SymbolFunction(SymbolText):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, inFileOffset: int, vram: int, instrsList: list[instructions.InstructionBase], segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, inFileOffset, vram, list(), segmentVromStart, overlayCategory)
        self.instructions: list[instructions.InstructionBase] = list(instrsList)

        self.pointersRemoved: bool = False

        self.localLabels: dict[int, str] = dict()
        """Branch labels found on this function.

        The key is the offset relative to the start of the function and the value is the name of the label

        If VRAM is available, then it is preferred to use `context.getSymbol(self.vram + branch, tryPlusOffset=False)` to get the name of a label instead.
        """

        # TODO: this needs a better name
        self.pointersPerInstruction: dict[int, int] = dict()
        self.constantsPerInstruction: dict[int, int] = dict()
        self.branchInstructions: list[int] = list()

        # key: %hi (lui) instruction offset, value: %lo instruction offset
        self.hiToLowDict: dict[int, int] = dict()
        # key: %lo instruction offset, value: %hi (lui) instruction offset
        self.lowToHiDict: dict[int, int] = dict()

        self.luiInstructions: dict[int, instructions.InstructionBase] = dict()
        self.nonPointerLuiSet: set[int] = set()
        self.gpInstructions: dict[int, instructions.InstructionBase] = dict()

        self.pointersOffsets: set[int] = set()
        self.referencedJumpTableOffsets: set[int] = set()

        self.referencedVRams: set[int] = set()
        self.referencedConstants: set[int] = set()

        self.hasUnimplementedIntrs: bool = False

        self.isRsp: bool = False

        self.isLikelyHandwritten: bool = False

    @property
    def nInstr(self) -> int:
        return len(self.instructions)

    @property
    def sizew(self) -> int:
        return self.nInstr

    def _printAnalisisDebugInfo_IterInfo(self, instr: instructions.InstructionBase, register1: int|None, register2: int|None, register3: int|None, currentVram: int, trackedRegisters: dict, registersValues: dict, registersDereferencedValues: dict):
        if not common.GlobalConfig.PRINT_FUNCTION_ANALYSIS_DEBUG_INFO:
            return

        print("_printAnalisisDebugInfo_IterInfo")
        print()
        print(f"vram: {currentVram:X}")
        print(instr)
        if register1 is not None:
            print(register1, instr.getRegisterName(register1))
        if register2 is not None:
            print(register2, instr.getRegisterName(register2))
        if register3 is not None:
            print(register3, instr.getRegisterName(register3))
        print(trackedRegisters)
        print({instr.getRegisterName(x): y for x, y in trackedRegisters.items()})
        # _t is shorthand of temp
        print({instr.getRegisterName(register_t): f"{vram_t:X},{offset_t:X}" for register_t, (vram_t, offset_t) in registersValues.items()})
        print({instr.getRegisterName(register_t): f"{vram_t:X},{offset_t:X}" for register_t, (vram_t, offset_t) in registersDereferencedValues.items()})
        print()

    def _printSymbolFinderDebugInfo_DelTrackedRegister(self, instr: instructions.InstructionBase, register: int, currentVram: int|None, trackedRegisters: dict):
        if not common.GlobalConfig.PRINT_SYMBOL_FINDER_DEBUG_INFO:
            return

        if currentVram is None:
            return

        print("_printSymbolFinderDebugInfo_DelTrackedRegister")
        print()
        print(f"vram: {currentVram:X}")
        print(instr)
        print(trackedRegisters)
        print(f"deleting {register} / {instr.getRegisterName(register)}")
        print()

    def _printSymbolFinderDebugInfo_UnpairedLuis(self):
        if not common.GlobalConfig.PRINT_UNPAIRED_LUIS_DEBUG_INFO:
            return

        firstNotePrinted = False

        for instructionOffset, luiInstr in self.luiInstructions.items():
            # inFileOffset = self.inFileOffset + instructionOffset
            currentVram = self.getVramOffset(instructionOffset)
            if instructionOffset in self.nonPointerLuiSet:
                continue
            if instructionOffset in self.constantsPerInstruction:
                # print(f"{currentVram:06X} ", end="")
                # print(f"C  {self.constantsPerInstruction[instructionOffset]:8X}", luiInstr)
                pass
            else:
                if common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and luiInstr.immediate < 0x4000: # filter out stuff that may not be a real symbol
                    continue
                if common.GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and luiInstr.immediate >= 0xC000: # filter out stuff that may not be a real symbol
                    continue

                # print(f"{currentVram:06X} ", end="")
                # if instructionOffset in self.pointersPerInstruction:
                #     print(f"P  {self.pointersPerInstruction[instructionOffset]:8X}", luiInstr)
                # else:
                #     print("NO         ", luiInstr)

                if instructionOffset not in self.pointersPerInstruction:
                    if not firstNotePrinted:
                        print("_printSymbolFinderDebugInfo_UnpairedLuis")
                        print(f"func: {self.getName()}")
                        print(f"vram: {self.vram:08X}")
                        firstNotePrinted = True

                    print(f"{currentVram:06X} ", "NO         ", luiInstr)

        if firstNotePrinted:
            print()


    def _processBranch(self, instr: instructions.InstructionBase, instructionOffset: int, currentVram: int) -> None:
        if instr.uniqueId == instructions.InstructionId.J:
            targetBranchVram = instr.getInstrIndexAsVram()
            branch = instructionOffset + targetBranchVram - currentVram
        else:
            branch = instructionOffset + instr.getBranchOffset()
            targetBranchVram = self.getVramOffset(branch)

        self.referencedVRams.add(targetBranchVram)

        labelSym = self.addBranchLabel(targetBranchVram, isAutogenerated=True)
        labelSym.referenceCounter += 1
        self.localLabels[branch] = labelSym.getName()
        self.branchInstructions.append(instructionOffset)

    def _processSymbol(self, luiInstr: instructions.InstructionBase|None, luiOffset: int|None, lowerInstr: instructions.InstructionBase, lowerOffset: int) -> int|None:
        # lui being None means this symbol is a $gp access
        assert (luiInstr is None and luiOffset is None) or (luiInstr is not None and luiOffset is not None)

        lowerHalf = common.Utils.from2Complement(lowerInstr.immediate, 16)

        if lowerOffset in self.pointersPerInstruction:
            # This %lo has been processed already
            if luiOffset is None or luiInstr is None:
                return None
            luiInstrPrev = self.instructions[(luiOffset-4)//4]
            if luiInstrPrev.isBranchLikely() or luiInstrPrev.isUnconditionalBranch():
                # This lui will be nullified afterwards, so it is likely for it to be re-used lui
                pass
            elif luiInstrPrev.isBranch():
                # I'm not really sure if a lui on any branch slot is enough to believe this is really a symbol
                # Let's hope it does for now...
                pass
            elif luiOffset + 4 == lowerOffset:
                # Make an exception if the lower instruction is just after the LUI
                pass
            else:
                upperHalf = luiInstr.immediate << 16
                address = upperHalf + lowerHalf
                if address == self.pointersPerInstruction[lowerOffset]:
                    # Make an exception if the resulting address is the same
                    pass
                else:
                    return self.pointersPerInstruction[lowerOffset]

        if luiInstr is None and common.GlobalConfig.GP_VALUE is None:
            return None

        if luiInstr is not None:
            upperHalf = luiInstr.immediate << 16
        else:
            assert common.GlobalConfig.GP_VALUE is not None
            upperHalf = common.GlobalConfig.GP_VALUE

        address = upperHalf + lowerHalf
        if address in self.context.bannedSymbols:
            return None

        # filter out stuff that may not be a real symbol
        filterOut = common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and upperHalf < 0x40000000
        filterOut |= common.GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and upperHalf >= 0xC0000000
        if filterOut:
            if common.GlobalConfig.SYMBOL_FINDER_FILTERED_ADDRESSES_AS_CONSTANTS:
                # Let's pretend this value is a constant
                constant = address
                self.referencedConstants.add(constant)

                self.constantsPerInstruction[lowerOffset] = constant
                if luiOffset is not None:
                    self.constantsPerInstruction[luiOffset] = constant

                    self.hiToLowDict[luiOffset] = lowerOffset
                    self.lowToHiDict[lowerOffset] = luiOffset
            return None

        patchedAddress = address
        patch = self.getLoPatch(lowerInstr.vram)
        if patch is not None:
            patchedAddress = patch

        self.referencedVRams.add(patchedAddress)
        contextSym = self.getSymbol(patchedAddress)
        if contextSym is None:
            if common.GlobalConfig.ADD_NEW_SYMBOLS:
                contextSym = self.addSymbol(patchedAddress, isAutogenerated=True)
                instrType = lowerInstr.mapInstrToType()
                if instrType is not None:
                    contextSym.setTypeIfUnset(instrType)
                contextSym.referenceCounter += 1
        else:
            contextSym.referenceCounter += 1

        if lowerOffset not in self.pointersPerInstruction:
            self.pointersPerInstruction[lowerOffset] = address
        if luiOffset is not None:
            if luiOffset not in self.pointersPerInstruction:
                self.pointersPerInstruction[luiOffset] = address

            self.hiToLowDict[luiOffset] = lowerOffset
            self.lowToHiDict[lowerOffset] = luiOffset
        else:
            self.gpInstructions[lowerOffset] = lowerInstr

        return address

    def _processConstant(self, luiInstr: instructions.InstructionBase, luiOffset: int, lowerInstr: instructions.InstructionBase, lowerOffset: int) -> int|None:
        upperHalf = luiInstr.immediate << 16
        lowerHalf = lowerInstr.immediate
        constant = upperHalf | lowerHalf

        self.referencedConstants.add(constant)

        self.constantsPerInstruction[lowerOffset] = constant
        self.constantsPerInstruction[luiOffset] = constant

        self.hiToLowDict[luiOffset] = lowerOffset
        self.lowToHiDict[lowerOffset] = luiOffset

        return constant

    def _moveRegisterInTrackers(self, instr: instructions.InstructionBase, trackedRegisters: dict, trackedRegistersAll: dict, registersValues: dict, registersDereferencedValues: dict[int, tuple[int, int]]) -> bool:
        if instr.uniqueId not in (instructions.InstructionId.MOVE, instructions.InstructionId.OR, instructions.InstructionId.ADDU):
            return False

        if instr.rt == 0 and instr.rs == 0:
            return False

        if instr.rt == 0:
            register = instr.rs
        elif instr.rs == 0:
            register = instr.rt
        else:
            # Check stuff like  `addu   $3, $3, $2`
            if instr.rd == instr.rs:
                register = instr.rt
            elif instr.rd == instr.rt:
                register = instr.rs
            else:
                return False

            updated = False
            if register in registersValues:
                registersValues[instr.rd] = registersValues[register]
                updated = True
            if register in registersDereferencedValues:
                registersDereferencedValues[instr.rd] = registersDereferencedValues[register]
                updated = True
            return updated

        updated = False
        if register in trackedRegistersAll:
            trackedRegistersAll[instr.rd] = trackedRegistersAll[register]
            updated = True
        if register in trackedRegisters:
            trackedRegisters[instr.rd] = trackedRegisters[register]
            updated = True
        if register in registersValues:
            registersValues[instr.rd] = registersValues[register]
            updated = True
        if register in registersDereferencedValues:
            registersDereferencedValues[instr.rd] = registersDereferencedValues[register]
            updated = True
        return updated

    def _removeRegisterFromTrackers(self, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase|None, currentVram: int|None, trackedRegisters: dict, trackedRegistersAll: dict, registersValues: dict, registersDereferencedValues: dict[int, tuple[int, int]], wasRegisterValuesUpdated: bool):
        shouldRemove = False
        register = 0

        if self._moveRegisterInTrackers(instr, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues):
            return

        if not instr.isFloatInstruction():
            if instr.isRType() or (instr.isBranch() and isinstance(instr, instructions.InstructionNormal)):
                # $at is a one-use register
                at = -1
                if instr.rs == 1:
                    at = instr.rs
                elif instr.rt == 1:
                    at = instr.rt

                if at in trackedRegistersAll:
                    otherInstrIndex = trackedRegistersAll[at]
                    otherInstr = self.instructions[otherInstrIndex]
                    if otherInstr.uniqueId == instructions.InstructionId.LUI:
                        self.nonPointerLuiSet.add(otherInstrIndex*4)
                    shouldRemove = True
                    register = at

            # if instr.uniqueId != instructions.InstructionId.LUI and instr.modifiesRt():
            if instr.modifiesRt():
                if instr.uniqueId == instructions.InstructionId.LUI:
                    register = instr.rt
                    if register in registersValues:
                        del registersValues[register]
                    if register in registersDereferencedValues:
                        del registersDereferencedValues[register]
                else:
                    shouldRemove = True
                    register = instr.rt

            if instr.modifiesRd():
                shouldRemove = True
                register = instr.rd

                # Usually array offsets use an ADDU to add the index of the array
                if instr.uniqueId == instructions.InstructionId.ADDU:
                    if instr.rd != instr.rs and instr.rd != instr.rt:
                        shouldRemove = True
                    else:
                        shouldRemove = False

        else:
            if instr.uniqueId in (instructions.InstructionId.MTC1, instructions.InstructionId.DMTC1, instructions.InstructionId.CTC1):
                # IDO usually use a register as a temp when loading a constant value
                # into the float coprocessor, after that IDO never re-uses the value
                # in that register for anything else
                shouldRemove = True
                register = instr.rt

            if instr.modifiesRt():
                shouldRemove = True
                register = instr.rt

            if instr.modifiesRd():
                shouldRemove = True
                register = instr.rd

        if shouldRemove:
            if register in trackedRegisters:
                self._printSymbolFinderDebugInfo_DelTrackedRegister(instr, register, currentVram, trackedRegisters)
                del trackedRegisters[register]
            if register in trackedRegistersAll:
                del trackedRegistersAll[register]
            if not wasRegisterValuesUpdated:
                if register in registersValues:
                    del registersValues[register]
                if register in registersDereferencedValues:
                    del registersDereferencedValues[register]

    def _invalidateRegistersInTrackersAfterFunctionCall(self, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase, currentVram: int|None, trackedRegisters: dict, trackedRegistersAll: dict, registersValues: dict, registersDereferencedValues: dict[int, tuple[int, int]]) -> None:
        if prevInstr.uniqueId != instructions.InstructionId.JAL:
            return

        # Happens $at, $v0 and $v1 have the same raw values for both o32 and n32 ABIs, so no need to worry about it for now...
        registersToInvalidate = (
            1, # $at
            2, # $v0
            3, # $v1
        )
        # TODO: should we worry about $a and $t registers?

        for register in registersToInvalidate:
            if register in trackedRegisters:
                self._printSymbolFinderDebugInfo_DelTrackedRegister(instr, register, currentVram, trackedRegisters)
                del trackedRegisters[register]
            if register in trackedRegistersAll:
                del trackedRegistersAll[register]
            if register in registersValues:
                del registersValues[register]
            if register in registersDereferencedValues:
                del registersDereferencedValues[register]

    def _tryToSetSymbolType(self, instr: instructions.InstructionBase, instructionOffset: int, registersValues: dict[int, tuple[int, int]], registersDereferencedValues: dict[int, tuple[int, int]]):
        instrType = instr.mapInstrToType()
        if instrType is None:
            return

        if instr.rs in registersValues:
            address, loInstructionOffset = registersValues[instr.rs]
            contextSym = self.getSymbol(address, tryPlusOffset=False)
            if contextSym is not None:
                contextSym.setTypeIfUnset(instrType)

        elif instr.rs in registersDereferencedValues:
            address, loInstructionOffset = registersDereferencedValues[instr.rs]
            if instructionOffset != loInstructionOffset:
                loInstr = self.instructions[loInstructionOffset//4]
                if loInstr.uniqueId != instructions.InstructionId.ADDIU:
                    # if the instruction used to load this value wasn't an ADDIU
                    # then the register has the value pointed by this address
                    return

            contextSym = self.getSymbol(address, tryPlusOffset=False)
            if contextSym is not None:
                contextSym.setTypeIfUnset(instrType)


    def _symbolFinder(self, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase|None, instructionOffset: int, trackedRegisters: dict[int, int], trackedRegistersAll: dict[int, int], registersValues: dict[int, tuple[int, int]], registersDereferencedValues: dict[int, tuple[int, int]]):
        if instr.uniqueId == instructions.InstructionId.LUI:
            if prevInstr is None or (not prevInstr.isBranchLikely() and not prevInstr.isUnconditionalBranch()):
                # If the previous instructions is a branch likely, then nulify
                # the effects of this instruction for future analysis
                trackedRegisters[instr.rt] = instructionOffset//4
            trackedRegistersAll[instr.rt] = instructionOffset//4
            return False

        if instr.uniqueId == instructions.InstructionId.ORI:
            # Constants
            rs = instr.rs
            if rs in trackedRegistersAll:
                luiOffset = trackedRegistersAll[rs] * 4
                luiInstr = self.instructions[luiOffset//4]
                constant = self._processConstant(luiInstr, luiOffset, instr, instructionOffset)
                if constant is not None:
                    registersValues[instr.rt] = (constant, instructionOffset)
                    return True
            return False

        if instr.uniqueId not in (instructions.InstructionId.ANDI, instructions.InstructionId.XORI, instructions.InstructionId.CACHE, instructions.InstructionId.SLTI, instructions.InstructionId.SLTIU):
            rs = instr.rs
            if rs in trackedRegisters:
                luiOffset = trackedRegisters[rs]*4
                luiInstr = self.instructions[luiOffset//4]
            elif rs == 28: # $gp
                luiOffset = None
                luiInstr = None
            else:
                if instr.uniqueId != instructions.InstructionId.ADDIU and instr.modifiesRt():
                    if rs in registersValues:
                        # Simulate a dereference
                        registersDereferencedValues[instr.rt] = registersValues[rs]
                        return True
                return False

            address = self._processSymbol(luiInstr, luiOffset, instr, instructionOffset)
            if address is not None:
                if instr.uniqueId == instructions.InstructionId.ADDIU:
                    registersValues[instr.rt] = (address, instructionOffset)
                else:
                    registersDereferencedValues[instr.rt] = (address, instructionOffset)
                return True

        return False


    def _lookAheadSymbolFinder(self, instr: instructions.InstructionBase, instructionOffset: int, trackedRegistersOriginal: dict[int, int], trackedRegistersAllOriginal: dict[int, int], registersValuesOriginal: dict[int, tuple[int, int]], registersDereferencedValuesOriginal: dict[int, tuple[int, int]]):
        trackedRegisters = dict(trackedRegistersOriginal)
        trackedRegistersAll = dict(trackedRegistersAllOriginal)
        registersValues = dict(registersValuesOriginal)
        registersDereferencedValues = dict(registersDereferencedValuesOriginal)

        lastInstr = self.instructions[instructionOffset//4 - 1]
        if not lastInstr.isBranch() and not lastInstr.isUnconditionalBranch():
            return

        if lastInstr.uniqueId == instructions.InstructionId.J:
            targetBranchVram = lastInstr.getInstrIndexAsVram()
            branchOffset = targetBranchVram - self.getVramOffset(instructionOffset)
        else:
            branchOffset = lastInstr.getBranchOffset() - 4
        branch = instructionOffset + branchOffset

        if instr.uniqueId == instructions.InstructionId.LUI:
            self._symbolFinder(instr, None, instructionOffset, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues)

        pairedLoFound = False
        i = 0
        while True:
            if branch//4 >= len(self.instructions):
                return

            if i >= 10:
                if instr.uniqueId == instructions.InstructionId.LUI:
                    # Continue searching until we find the corresponding lo instruction for this LUI
                    if pairedLoFound:
                        return
                else:
                    # Only check the 5 next instructions in the target branch for non LUI instructions
                    return

            wasRegisterValuesUpdated = False
            prevTargetInstr = self.instructions[branch//4 - 1]
            targetInstr = self.instructions[branch//4]

            # Usually array offsets use an ADDU to add the index of the array
            if targetInstr.uniqueId == instructions.InstructionId.ADDU and not prevTargetInstr.isBranchLikely() and not prevTargetInstr.isUnconditionalBranch():
                if targetInstr.rd == targetInstr.rs or targetInstr.rd == targetInstr.rt:
                    branch += 4
                    i += 1
                    continue

            if targetInstr.isIType():
                if self._symbolFinder(targetInstr, prevTargetInstr, branch, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues):
                    wasRegisterValuesUpdated = True
                    if instr.uniqueId == instructions.InstructionId.LUI and targetInstr.rs == instr.rt:
                        pairedLoFound = True
                self._tryToSetSymbolType(targetInstr, branch, registersValues, registersDereferencedValues)

            if prevTargetInstr.isUnconditionalBranch():
                # TODO: Consider following branches
                # self._lookAheadSymbolFinder(targetInstr, branch, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues)
                return
            if prevTargetInstr.isJType():
                return
            if prevTargetInstr.isJump():
                return

            self._removeRegisterFromTrackers(targetInstr, prevTargetInstr, None, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues, wasRegisterValuesUpdated)

            self._invalidateRegistersInTrackersAfterFunctionCall(targetInstr, prevTargetInstr, None, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues)

            branch += 4
            i += 1

    def _processElfRelocSymbols(self):
        if len(self.context.relocSymbols[common.FileSectionType.Text]) == 0:
            return

        # Process reloc symbols (probably from a .elf file)
        instructionOffset = 0
        inFileOffset = self.inFileOffset
        for instr in self.instructions:
            relocSymbol = self.context.getRelocSymbol(inFileOffset, common.FileSectionType.Text)
            if relocSymbol is not None:
                if relocSymbol.name is not None and relocSymbol.name.startswith("."):
                    sectType = common.FileSectionType.fromStr(relocSymbol.name)

                    if instructionOffset in self.pointersPerInstruction:
                        if instructionOffset in self.referencedJumpTableOffsets:
                            # Jump tables
                            addressOffset = self.pointersPerInstruction[instructionOffset]
                            if relocSymbol.name != ".rodata":
                                common.Utils.eprint(f"Warning. Jumptable referenced in reloc does not have '.rodata' as its name")
                            contextOffsetSym = self.context.addOffsetJumpTable(addressOffset, sectType)
                            contextOffsetSym.referenceCounter += 1
                            relocSymbol.name = contextOffsetSym.name
                            self.pointersPerInstruction[instructionOffset] = 0
                            if instructionOffset in self.lowToHiDict:
                                luiOffset = self.lowToHiDict[instructionOffset]
                                otherReloc = self.context.getRelocSymbol(self.inFileOffset+luiOffset, common.FileSectionType.Text)
                                if otherReloc is not None:
                                    otherReloc.name = relocSymbol.name
                                    self.pointersPerInstruction[luiOffset] = 0
                        else:
                            addressOffset = self.pointersPerInstruction[instructionOffset]
                            relocName = f"{relocSymbol.name}_{addressOffset:06X}"
                            # print(relocName, addressOffset, instr)
                            contextOffsetSym = common.ContextOffsetSymbol(addressOffset, relocName, sectType)
                            self.context.offsetSymbols[sectType][addressOffset] = contextOffsetSym
                            relocSymbol.name = relocName
                            self.pointersPerInstruction[instructionOffset] = 0
            inFileOffset += 4
            instructionOffset += 4


    def analyze(self):
        if not common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS and self.hasUnimplementedIntrs:
            offset = 0
            for instr in self.instructions:
                currentVram = self.getVramOffset(offset)
                contextSym = self.getSymbol(currentVram, False)
                if contextSym is not None:
                    contextSym.isDefined = True

                offset += 4
            return

        # Search for LUI instructions first
        instructionOffset = 0
        for instr in self.instructions:
            if instr.uniqueId == instructions.InstructionId.LUI:
                self.luiInstructions[instructionOffset] = instr
            if instructionOffset > 0:
                prevInstr = self.instructions[instructionOffset//4 - 1]
                if prevInstr.isJType() or prevInstr.isJump():
                    self.nonPointerLuiSet.add(instructionOffset)
            instructionOffset += 4

        trackedRegisters: dict[int, int] = dict()
        trackedRegistersAll: dict[int, int] = dict()
        # key: register, value: (vram, offset of instruction which set this value)
        registersValues: dict[int, tuple[int, int]] = dict()
        registersDereferencedValues: dict[int, tuple[int, int]] = dict()

        instructionOffset = 0
        for instr in self.instructions:
            currentVram = self.getVramOffset(instructionOffset)
            wasRegisterValuesUpdated = False
            self.isLikelyHandwritten |= instr.uniqueId in instructions.InstructionsNotEmitedByIDO
            prevInstr = self.instructions[instructionOffset//4 - 1]

            self._printAnalisisDebugInfo_IterInfo(instr, instr.rs, instr.rt, instr.rd, currentVram, trackedRegisters, registersValues, registersDereferencedValues)

            if not self.isLikelyHandwritten:
                self.isLikelyHandwritten = instr.isLikelyHandwritten()

            if not common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS and not instr.isImplemented():
                # Abort analysis
                self.hasUnimplementedIntrs = True
                return

            if instr.isBranch() or instr.isUnconditionalBranch():
                self._processBranch(instr, instructionOffset, currentVram)

            elif instr.isJType():
                target = instr.getInstrIndexAsVram()
                if not self.isRsp:
                    if target >= 0x84000000:
                        # RSP address space?
                        self.isLikelyHandwritten = True

                funcSym = self.addFunction(target, isAutogenerated=True)
                funcSym.referenceCounter += 1
                self.pointersPerInstruction[instructionOffset] = target

            # symbol finder
            elif instr.isIType():
                if self._symbolFinder(instr, prevInstr, instructionOffset, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues):
                    wasRegisterValuesUpdated = True
                self._tryToSetSymbolType(instr, instructionOffset, registersValues, registersDereferencedValues)

            elif instr.uniqueId == instructions.InstructionId.JR:
                rs = instr.rs
                if rs != 31: # $ra
                    if rs in registersDereferencedValues:
                        # print(instructionOffset, rs, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues, self.pointersPerInstruction)
                        address, jmptblSeterOffset = registersDereferencedValues[rs]
                        self.referencedJumpTableOffsets.add(jmptblSeterOffset)
                        self.referencedVRams.add(address)
                        self.addJumpTable(address, isAutogenerated=True)

            self._removeRegisterFromTrackers(instr, prevInstr, currentVram, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues, wasRegisterValuesUpdated)

            # look-ahead symbol finder
            self._lookAheadSymbolFinder(instr, instructionOffset, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues)

            self._invalidateRegistersInTrackersAfterFunctionCall(instr, prevInstr, currentVram, trackedRegisters, trackedRegistersAll, registersValues, registersDereferencedValues)

            instructionOffset += 4

        self._printSymbolFinderDebugInfo_UnpairedLuis()

        self._processElfRelocSymbols()

        for instr in self.instructions:
            instr.inHandwrittenFunction = self.isLikelyHandwritten


    def countDiffOpcodes(self, other: SymbolFunction) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            if not self.instructions[i].sameOpcode(other.instructions[i]):
                result += 1
        return result

    def countSameOpcodeButDifferentArguments(self, other: SymbolFunction) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                result += 1
        return result

    def blankOutDifferences(self, other_func: SymbolFunction) -> bool:
        if not common.GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False

        for i in range(min(self.nInstr, other_func.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other_func.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                instr1.blankOut()
                instr2.blankOut()
                was_updated = True

        return was_updated

    def removePointers(self) -> bool:
        if not common.GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False

        for instructionOffset in self.pointersPerInstruction:
            self.instructions[instructionOffset//4].blankOut()
        was_updated = len(self.pointersPerInstruction) > 0 or was_updated

        for fileOffset in self.pointersOffsets:
            index = (fileOffset - self.inFileOffset)//4
            if index < 0:
                continue
            if index >= self.nInstr:
                continue
            self.instructions[index].blankOut()

        if common.GlobalConfig.IGNORE_BRANCHES:
            for instructionOffset in self.branchInstructions:
                self.instructions[instructionOffset//4].blankOut()
            was_updated = len(self.branchInstructions) > 0 or was_updated

        self.pointersRemoved = True

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False
        first_nop = self.nInstr

        for i in range(self.nInstr-1, 0-1, -1):
            instr = self.instructions[i]
            if instr.uniqueId != instructions.InstructionId.NOP:
                if instr.uniqueId == instructions.InstructionId.JR and instr.rs == 31: #$ra
                    first_nop += 1
                break
            first_nop = i

        if first_nop < self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated


    def generateHiLoStr(self, instr: instructions.InstructionBase, symName: str) -> str:
        if instr.uniqueId == instructions.InstructionId.LUI:
            return f"%hi({symName})"

        # $gp
        if instr.rs == 28:
            # $gp
            if instr.rt != 28 or not instr.modifiesRt():
                return f"%gp_rel({symName})"

        return f"%lo({symName})"

    def getImmOverrideForInstruction(self, instr: instructions.InstructionBase, instructionOffset: int) -> str|None:
        if len(self.context.relocSymbols[self.sectionType]) > 0:
            # Check possible symbols using reloc information (probably from a .o elf file)
            possibleImmOverride = self.context.getRelocSymbol(self.inFileOffset + instructionOffset, self.sectionType)
            if possibleImmOverride is not None:
                auxOverride = possibleImmOverride.getName()
                if instr.isIType():
                    if instructionOffset in self.pointersPerInstruction:
                        addressOffset = self.pointersPerInstruction[instructionOffset]
                        auxOverride = possibleImmOverride.getNamePlusOffset(addressOffset)

                    auxOverride = self.generateHiLoStr(instr, auxOverride)
                return auxOverride

        if instr.isBranch() or instr.isUnconditionalBranch():
            if not common.GlobalConfig.IGNORE_BRANCHES:
                if instr.uniqueId == instructions.InstructionId.J:
                    targetBranchVram = instr.getInstrIndexAsVram()
                    branch = instructionOffset + targetBranchVram - self.getVramOffset(instructionOffset)
                else:
                    branch = instructionOffset + instr.getBranchOffset()
                    targetBranchVram = self.getVramOffset(branch)
                labelSymbol = self.getSymbol(targetBranchVram, tryPlusOffset=False)
                if labelSymbol is not None:
                    return labelSymbol.getName()

                # in case we don't have access to vram or this label was not in context
                if branch in self.localLabels:
                    return self.localLabels[branch]

        elif instr.isIType():
            if not self.pointersRemoved and instructionOffset in self.pointersPerInstruction:
                address = self.pointersPerInstruction[instructionOffset]

                instrVram = instr.vram
                if instr.uniqueId == instructions.InstructionId.LUI:
                    # we need to get the address of the lo instruction to get the patch
                    if instructionOffset in self.hiToLowDict:
                        loInstr = self.instructions[self.hiToLowDict[instructionOffset] // 4]
                        instrVram = loInstr.vram

                # Check for user-defined symbol patches
                patchedAddress = self.getLoPatch(instrVram)
                if patchedAddress is not None:
                    symbol = self.getSymbol(patchedAddress, tryPlusOffset=True, checkUpperLimit=False)
                else:
                    symbol = self.getSymbol(address, tryPlusOffset=True)

                if symbol is not None:
                    return self.generateHiLoStr(instr, symbol.getSymbolPlusOffset(address))

            elif instructionOffset in self.constantsPerInstruction:
                constant = self.constantsPerInstruction[instructionOffset]

                symbol = self.getConstant(constant)
                if symbol is not None:
                    return self.generateHiLoStr(instr, symbol.getName())

                # Pretend this pair is a constant
                if instr.uniqueId == instructions.InstructionId.LUI:
                    loInstr = self.instructions[self.hiToLowDict[instructionOffset] // 4]
                    if loInstr.uniqueId == instructions.InstructionId.ORI:
                        return f"(0x{constant:X} >> 16)"
                elif instr.uniqueId == instructions.InstructionId.ORI:
                    return f"(0x{constant:X} & 0xFFFF)"

                if common.GlobalConfig.SYMBOL_FINDER_FILTERED_ADDRESSES_AS_HILO:
                    return self.generateHiLoStr(instr, f"0x{constant:X}")

            elif instr.uniqueId == instructions.InstructionId.LUI:
                # Unpaired LUI
                return f"(0x{instr.immediate<<16:X} >> 16)"

        elif instr.isJType():
            possibleOverride = self.getSymbol(instr.getInstrIndexAsVram(), tryPlusOffset=False)
            if possibleOverride is not None:
                return possibleOverride.getName()

        return None

    def getLabelForOffset(self, instructionOffset: int) -> str:
        if not common.GlobalConfig.IGNORE_BRANCHES and instructionOffset != 0:
            # Skip over this function to avoid duplication

            currentVram = self.getVramOffset(instructionOffset)
            labelSym = self.getSymbol(currentVram, tryPlusOffset=False)
            if labelSym is None and len(self.context.offsetJumpTablesLabels) > 0:
                labelSym = self.context.getOffsetGenericLabel(self.inFileOffset+instructionOffset, common.FileSectionType.Text)
            if labelSym is None and len(self.context.offsetSymbols[self.sectionType]) > 0:
                labelSym = self.context.getOffsetSymbol(self.inFileOffset+instructionOffset, common.FileSectionType.Text)

            if labelSym is not None:
                labelSym.isDefined = True
                labelSym.sectionType = self.sectionType
                if labelSym.type == common.SymbolSpecialType.function or labelSym.type == common.SymbolSpecialType.jumptablelabel:
                    label = labelSym.getSymbolLabel() + common.GlobalConfig.LINE_ENDS
                    if common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL:
                        label += f"{labelSym.getName()}:{common.GlobalConfig.LINE_ENDS}"
                    return label
                return labelSym.getName() + ":" + common.GlobalConfig.LINE_ENDS

            if instructionOffset in self.localLabels:
                return self.localLabels[instructionOffset] + ":" + common.GlobalConfig.LINE_ENDS
        return ""


    def disassemble(self) -> str:
        output = ""

        if not common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS:
            if self.hasUnimplementedIntrs:
                return self.disassembleAsData()

        if self.isLikelyHandwritten:
            output += "/* Handwritten function */" + common.GlobalConfig.LINE_ENDS

        output += self.getLabel()

        if common.GlobalConfig.ASM_TEXT_ENT_LABEL:
            output += f"{common.GlobalConfig.ASM_TEXT_ENT_LABEL} {self.getName()}" + common.GlobalConfig.LINE_ENDS

        if common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL:
            output += f"{self.getName()}:" + common.GlobalConfig.LINE_ENDS

        wasLastInstABranch = False
        instructionOffset = 0
        for instr in self.instructions:
            immOverride = self.getImmOverrideForInstruction(instr, instructionOffset)
            comment = self.generateAsmLineComment(instructionOffset, instr.instr)

            if wasLastInstABranch:
                instr.extraLjustWidthOpcode -= 1
                comment += " "

            line = instr.disassemble(immOverride)

            if wasLastInstABranch:
                instr.extraLjustWidthOpcode += 1

            label = self.getLabelForOffset(instructionOffset)
            output += f"{label}{comment}  {line}" + common.GlobalConfig.LINE_ENDS

            wasLastInstABranch = instr.isBranch() or instr.isJump()
            instructionOffset += 4

        if common.GlobalConfig.ASM_TEXT_END_LABEL:
            output += f"{common.GlobalConfig.ASM_TEXT_END_LABEL} {self.getName()}" + common.GlobalConfig.LINE_ENDS

        return output

    def disassembleAsData(self) -> str:
        self.words = [instr.instr for instr in self.instructions]
        return super().disassembleAsData()
