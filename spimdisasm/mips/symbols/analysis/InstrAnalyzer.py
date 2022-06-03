#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from .... import common

from ... import instructions

from .RegistersTracker import RegistersTracker


class InstrAnalyzer:
    def __init__(self, funcVram: int) -> None:
        self.funcVram = funcVram

        self.referencedVrams: set[int] = set()
        "Every referenced vram found"
        self.referencedConstants: set[int] = set()
        "Every referenced constant found"

        self.referencedVramsInstrOffset: dict[int, int] = dict()

        # Branches
        self.branchInstrOffsets: dict[int, int] = dict()
        "key: branch instruction offset, value: target vram"

        self.branchTargetInstrOffsets: dict[int, int] = dict()
        "key: branch instruction offset, value: relative branch target"

        self.branchLabelOffsets: set[int] = set()

        # Function calls
        self.funcCallInstrOffsets: dict[int, int] = dict()
        "key: func call instruction offset, value: target vram"
        self.funcCallOutsideRangesOffsets: dict[int, int] = dict()
        "key: func call instruction offset, value: target vram which is outside the [0x80000000, 0x84000000] range"

        # Jump register (jumptables)
        self.jumpRegisterIntrOffset: dict[int, int] = dict()
        self.referencedJumpTableOffsets: dict[int, int] = dict()

        # Constants
        self.constantHiInstrOffset: dict[int, int] = dict()
        "key: offset of instruction which is setting the %hi constant, value: constant"
        self.constantLoInstrOffset: dict[int, int] = dict()
        "key: offset of instruction which is setting the %lo constant, value: constant"

        self.constantInstrOffset: dict[int, int] = dict()

        # Symbols
        self.symbolHiInstrOffset: dict[int, int] = dict()
        "key: offset of instruction which is setting the %hi symbol, value: symbol"
        self.symbolLoInstrOffset: dict[int, int] = dict()
        "key: offset of instruction which is setting the %lo symbol, value: symbol"

        self.symbolGpInstrOffset: dict[int, int] = dict()

        self.symbolInstrOffset: dict[int, int] = dict()

        self.possibleSymbolTypes: dict[int, str] = dict()

        # %hi/%lo pairing
        self.hiToLowDict: dict[int, int] = dict()
        "key: %hi instruction offset, value: %lo instruction offset"
        self.lowToHiDict: dict[int, int] = dict()
        "key: %lo instruction offset, value: %hi instruction offset"

        self.luiInstrs: dict[int, instructions.InstructionBase] = dict()

        self.nonLoInstrOffsets: set[int] = set()


    def processBranch(self, instr: instructions.InstructionBase, instrOffset: int, currentVram: int) -> None:
        if instrOffset in self.branchInstrOffsets:
            # Already processed
            return

        if instr.uniqueId == instructions.InstructionId.J:
            targetBranchVram = instr.getInstrIndexAsVram()
            branch = instrOffset + targetBranchVram - currentVram
        else:
            branch = instrOffset + instr.getBranchOffset()
            targetBranchVram = self.funcVram + branch

        self.referencedVrams.add(targetBranchVram)

        self.branchLabelOffsets.add(branch)
        self.branchInstrOffsets[instrOffset] = targetBranchVram
        self.branchTargetInstrOffsets[instrOffset] = branch

    def processFuncCall(self, instr: instructions.InstructionBase, instrOffset: int) -> None:
        if instrOffset in self.funcCallInstrOffsets:
            # Already processed
            return

        target = instr.getInstrIndexAsVram()
        if target >= 0x84000000 or target < 0x80000000:
            self.funcCallOutsideRangesOffsets[instrOffset] = target

        self.referencedVrams.add(target)
        self.referencedVramsInstrOffset[instrOffset] = target

        self.funcCallInstrOffsets[instrOffset] = target


    def processConstant(self, regsTracker: RegistersTracker, luiInstr: instructions.InstructionBase, luiOffset: int, lowerInstr: instructions.InstructionBase, lowerOffset: int) -> int|None:
        upperHalf = luiInstr.immediate << 16
        lowerHalf = lowerInstr.immediate
        constant = upperHalf | lowerHalf

        self.referencedConstants.add(constant)

        self.constantHiInstrOffset[luiOffset] = constant
        self.constantLoInstrOffset[lowerOffset] = constant
        self.constantInstrOffset[luiOffset] = constant
        self.constantInstrOffset[lowerOffset] = constant

        self.hiToLowDict[luiOffset] = lowerOffset
        self.lowToHiDict[lowerOffset] = luiOffset

        regsTracker.processConstant(constant, lowerInstr, lowerOffset)

        return constant


    def pairHiLo(self, luiInstr: instructions.InstructionBase|None, luiOffset: int|None, lowerInstr: instructions.InstructionBase, lowerOffset: int) -> int|None:
        # lui being None means this symbol is a $gp access
        assert (luiInstr is None and luiOffset is None) or (luiInstr is not None and luiOffset is not None)

        lowerHalf = common.Utils.from2Complement(lowerInstr.immediate, 16)

        if lowerOffset in self.symbolLoInstrOffset:
            # This %lo has been processed already

            if common.GlobalConfig.COMPILER == common.Compiler.IDO:
                # IDO does not pair multiples %hi to the same %lo
                return self.symbolLoInstrOffset[lowerOffset]

            elif common.GlobalConfig.COMPILER in {common.Compiler.GCC, common.Compiler.SN64}:
                if luiOffset is None or luiInstr is None:
                    return None

                if self.hiToLowDict.get(luiOffset, None) == lowerOffset and self.lowToHiDict.get(lowerOffset, None) == luiOffset:
                    # This pair has been already paired
                    return self.symbolLoInstrOffset[lowerOffset]

                # luiInstrPrev = self.instructions[(luiOffset-4)//4]
                # if luiInstrPrev.isBranchLikely() or luiInstrPrev.isUnconditionalBranch():
                #     # This lui will be nullified afterwards, so it is likely for it to be re-used lui
                #     pass
                # elif luiInstrPrev.isBranch():
                #     # I'm not really sure if a lui on any branch slot is enough to believe this is really a symbol
                #     # Let's hope it does for now...
                #     pass
                # elif luiOffset + 4 == lowerOffset:
                if luiOffset + 4 == lowerOffset:
                    # Make an exception if the lower instruction is just after the LUI
                    pass
                else:
                    upperHalf = luiInstr.immediate << 16
                    address = upperHalf + lowerHalf
                    if address == self.symbolLoInstrOffset[lowerOffset]:
                        # Make an exception if the resulting address is the same
                        pass
                    else:
                        return self.symbolLoInstrOffset[lowerOffset]

        if luiInstr is None and common.GlobalConfig.GP_VALUE is None:
            # Trying to pair a gp relative offset, but we don't know the gp address
            return None

        if luiInstr is not None:
            upperHalf = luiInstr.immediate << 16
        else:
            assert common.GlobalConfig.GP_VALUE is not None
            upperHalf = common.GlobalConfig.GP_VALUE

        return upperHalf + lowerHalf


    def processSymbol(self, address: int, luiOffset: int|None, lowerInstr: instructions.InstructionBase, lowerOffset: int) -> int|None:
        # filter out stuff that may not be a real symbol
        filterOut = common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and address < 0x80000000
        filterOut |= common.GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and address >= 0xC0000000
        if filterOut:
            if common.GlobalConfig.SYMBOL_FINDER_FILTERED_ADDRESSES_AS_CONSTANTS:
                # Let's pretend this value is a constant
                constant = address
                self.referencedConstants.add(constant)

                self.constantLoInstrOffset[lowerOffset] = constant
                self.constantInstrOffset[lowerOffset] = constant
                if luiOffset is not None:
                    self.constantHiInstrOffset[luiOffset] = constant
                    self.constantInstrOffset[luiOffset] = constant

                    self.hiToLowDict[luiOffset] = lowerOffset
                    self.lowToHiDict[lowerOffset] = luiOffset
            return None

        self.referencedVrams.add(address)

        if lowerOffset not in self.symbolLoInstrOffset:
            self.symbolLoInstrOffset[lowerOffset] = address
            self.symbolInstrOffset[lowerOffset] = address
            self.referencedVramsInstrOffset[lowerOffset] = address
        if luiOffset is not None:
            if luiOffset not in self.symbolHiInstrOffset:
                self.symbolHiInstrOffset[luiOffset] = address
                self.symbolInstrOffset[luiOffset] = address
                self.referencedVramsInstrOffset[luiOffset] = address

            self.hiToLowDict[luiOffset] = lowerOffset
            self.lowToHiDict[lowerOffset] = luiOffset
        else:
            self.symbolGpInstrOffset[lowerOffset] = address
            self.symbolInstrOffset[lowerOffset] = address
            self.referencedVramsInstrOffset[lowerOffset] = address

        self.processSymbolType(address, lowerInstr)

        return address

    def processSymbolType(self, address: int, instr: instructions.InstructionBase) -> None:
        instrType = instr.mapInstrToType()
        if instrType is None:
            return

        if address not in self.possibleSymbolTypes:
            self.possibleSymbolTypes[address] = instrType

    def processSymbolDereferenceType(self, regsTracker: RegistersTracker, instr: instructions.InstructionBase, instrOffset: int) -> None:
        address = regsTracker.getAddressIfCanSetType(instr, instrOffset)
        if address is None:
            return

        self.processSymbolType(address, instr)


    def symbolFinder(self, regsTracker: RegistersTracker, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase|None, instrOffset: int) -> None:
        if instr.uniqueId == instructions.InstructionId.LUI:
            regsTracker.processLui(instr, prevInstr, instrOffset)
            self.luiInstrs[instrOffset] = instr
            return

        if instr.uniqueId == instructions.InstructionId.ORI:
            # Constants
            luiOffset = regsTracker.getLuiOffsetForConstant(instr)
            if luiOffset is None:
                return
            luiInstr = self.luiInstrs.get(luiOffset, None)
            if luiInstr is None:
                return
            self.processConstant(regsTracker, luiInstr, luiOffset, instr, instrOffset)
            return

        if instr.uniqueId in {instructions.InstructionId.ANDI, instructions.InstructionId.XORI, instructions.InstructionId.CACHE, instructions.InstructionId.SLTI, instructions.InstructionId.SLTIU}:
            return

        if instrOffset in self.nonLoInstrOffsets:
            return

        luiOffset, shouldProcess = regsTracker.getLuiOffsetForLo(instr, instrOffset)
        if not shouldProcess:
            state = regsTracker.registers[instr.rs]
            if state.hasLoValue and not state.hasLuiValue:
                self.nonLoInstrOffsets.add(instrOffset)
            return

        luiInstr = None
        if luiOffset is not None:
            luiInstr = self.luiInstrs.get(luiOffset, None)
            if luiInstr is None:
                return

        address = self.pairHiLo(luiInstr, luiOffset, instr, instrOffset)
        if address is None:
            return

        address = self.processSymbol(address, luiOffset, instr, instrOffset)
        if address is not None:
            regsTracker.processLo(instr, address, instrOffset)


    def processJumpRegister(self, regsTracker: RegistersTracker, instr: instructions.InstructionBase, instrOffset: int) -> None:
        jrInfo = regsTracker.getJrInfo(instr)
        if jrInfo is not None:
            offset, address = jrInfo

            self.referencedJumpTableOffsets[offset] = address
            self.jumpRegisterIntrOffset[instrOffset] = address
            self.referencedVrams.add(address)


    def processInstr(self, regsTracker: RegistersTracker, instr: instructions.InstructionBase, instrOffset: int, currentVram: int, prevInstr: instructions.InstructionBase|None=None) -> None:
        if instr.isBranch() or instr.isUnconditionalBranch():
            self.processBranch(instr, instrOffset, currentVram)

        elif instr.isJType():
            self.processFuncCall(instr, instrOffset)

        elif instr.isIType():
            self.symbolFinder(regsTracker, instr, prevInstr, instrOffset)
            self.processSymbolDereferenceType(regsTracker, instr, instrOffset)

        elif instr.isJrNotRa():
            self.processJumpRegister(regsTracker, instr, instrOffset)

        regsTracker.overwriteRegisters(instr, instrOffset, currentVram)


    def processPrevFuncCall(self, regsTracker: RegistersTracker, instr: instructions.InstructionBase, prevInstr: instructions.InstructionBase, currentVram: int | None = None) -> None:
        regsTracker.unsetRegistersAfterFuncCall(instr, prevInstr, currentVram)



    def printAnalisisDebugInfo_IterInfo(self, regsTracker: RegistersTracker, instr: instructions.InstructionBase, currentVram: int):
        if not common.GlobalConfig.PRINT_FUNCTION_ANALYSIS_DEBUG_INFO:
            return

        print("_printAnalisisDebugInfo_IterInfo")
        print()
        print(f"vram: {currentVram:X}")
        print(instr)
        print(instr.rs, instr.getRegisterName(instr.rs))
        print(instr.rt, instr.getRegisterName(instr.rt))
        print(instr.rd, instr.getRegisterName(instr.rd))
        print(regsTracker.registers)
        print({instr.getRegisterName(x): y for x, y in regsTracker.registers.items()})
        # _t is shorthand of temp
        print({instr.getRegisterName(register_t): f"{state_t.value:X},{state_t.loOffset:X},{state_t.dereferenced}" for register_t, state_t in regsTracker.registers.items() if state_t.hasLoValue})
        print()

    def printSymbolFinderDebugInfo_UnpairedLuis(self):
        if not common.GlobalConfig.PRINT_UNPAIRED_LUIS_DEBUG_INFO:
            return

        firstNotePrinted = False

        for instructionOffset, luiInstr in self.luiInstrs.items():
            # if instructionOffset in self.nonPointerLuiSet:
            #     continue
            if instructionOffset in self.constantInstrOffset:
                # print(f"{currentVram:06X} ", end="")
                # print(f"C  {self.constantsPerInstruction[instructionOffset]:8X}", luiInstr)
                pass
            else:
                if common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and luiInstr.immediate < 0x8000: # filter out stuff that may not be a real symbol
                    continue
                if common.GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and luiInstr.immediate >= 0xC000: # filter out stuff that may not be a real symbol
                    continue

                # print(f"{currentVram:06X} ", end="")
                # if instructionOffset in self.pointersPerInstruction:
                #     print(f"P  {self.pointersPerInstruction[instructionOffset]:8X}", luiInstr)
                # else:
                #     print("NO         ", luiInstr)

                if instructionOffset not in self.symbolInstrOffset:
                    if not firstNotePrinted:
                        print("_printSymbolFinderDebugInfo_UnpairedLuis")
                        print(f"funcVram: {self.funcVram:08X}")
                        firstNotePrinted = True

                    print(f"{luiInstr.vram:06X} ", "NO         ", luiInstr)

        if firstNotePrinted:
            print()
