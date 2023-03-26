#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import rabbitizer

from ... import common

from .. import symbols
from ..MipsFileBase import FileBase

from . import SectionBase


class SectionText(SectionBase):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, vram: int, filename: str, array_of_bytes: bytearray, segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, vram, filename, common.Utils.bytesToWords(array_of_bytes, vromStart, vromEnd), common.FileSectionType.Text, segmentVromStart, overlayCategory)

        self.instrCat: rabbitizer.Enum = rabbitizer.InstrCategory.CPU


    @property
    def nFuncs(self) -> int:
        return len(self.symbolList)

    @staticmethod
    def wordListToInstructions(wordList: list[int], currentVram: int|None, instrCat: rabbitizer.Enum) -> list[rabbitizer.Instruction]:
        instrsList: list[rabbitizer.Instruction] = list()
        for word in wordList:
            instr = rabbitizer.Instruction(word, category=instrCat)

            if currentVram is not None:
                instr.vram = currentVram
                currentVram += 4

            instrsList.append(instr)
        return instrsList


    def _findFunctions(self, instrsList: list[rabbitizer.Instruction]):
        if len(instrsList) == 0:
            return [0], [False]

        functionEnded = False
        farthestBranch = 0
        funcsStartsList = [0]
        unimplementedInstructionsFuncList = []

        instructionOffset = 0
        currentInstructionStart = 0
        currentFunctionSym = self.getSymbol(self.getVramOffset(instructionOffset), tryPlusOffset=False, checkGlobalSegment=False)

        isLikelyHandwritten = self.isHandwritten

        isInstrImplemented = True
        index = 0
        nInstr = len(instrsList)

        if instrsList[0].isNop():
            isboundary = False
            # Loop over until we find a instruction that isn't a nop
            while index < nInstr:
                if currentFunctionSym is not None:
                    break

                instr = instrsList[index]
                if not instr.isNop():
                    if isboundary:
                        self.fileBoundaries.append(self.inFileOffset + index*4)
                    break
                index += 1
                instructionOffset += 4
                isboundary |= ((instructionOffset % 16) == 0)

                currentInstructionStart = instructionOffset
                currentFunctionSym = self.getSymbol(self.getVramOffset(instructionOffset), tryPlusOffset=False, checkGlobalSegment=False)

            if index != 0:
                funcsStartsList.append(index)
                unimplementedInstructionsFuncList.append(not isInstrImplemented)

        while index < nInstr:
            instr = instrsList[index]
            if not instr.isImplemented():
                isInstrImplemented = False

            if functionEnded:
                functionEnded = False

                isLikelyHandwritten = self.isHandwritten
                index += 1
                instructionOffset += 4

                auxSym = self.getSymbol(self.getVramOffset(instructionOffset), tryPlusOffset=False, checkGlobalSegment=False)

                isboundary = False
                # Loop over until we find a instruction that isn't a nop
                while index < nInstr:
                    if auxSym is not None:
                        break

                    instr = instrsList[index]
                    if not instr.isNop():
                        if isboundary:
                            self.fileBoundaries.append(self.inFileOffset + index*4)
                        break
                    index += 1
                    instructionOffset += 4
                    isboundary |= ((instructionOffset % 16) == 0)

                    auxSym = self.getSymbol(self.getVramOffset(instructionOffset), tryPlusOffset=False, checkGlobalSegment=False)

                currentInstructionStart = instructionOffset
                currentFunctionSym = auxSym

                funcsStartsList.append(index)
                unimplementedInstructionsFuncList.append(not isInstrImplemented)
                if index >= len(instrsList):
                    break
                instr = instrsList[index]
                isInstrImplemented = instr.isImplemented()

            currentVram = self.getVramOffset(instructionOffset)

            if self.instrCat != rabbitizer.InstrCategory.RSP and not isLikelyHandwritten:
                isLikelyHandwritten = instr.isLikelyHandwritten()

            if instr.isBranch() or instr.isUnconditionalBranch():
                branchOffset = instr.getBranchOffsetGeneric()
                if branchOffset > farthestBranch:
                    # keep track of the farthest branch target
                    farthestBranch = branchOffset
                if branchOffset < 0:
                    if branchOffset + instructionOffset < 0:
                        # Whatever we are reading is not a valid instruction
                        if not instr.isJump(): # Make an exception for `j`
                            break
                    # make sure to not branch outside of the current function
                    if not isLikelyHandwritten:
                        j = len(funcsStartsList) - 1
                        while j >= 0:
                            if branchOffset + instructionOffset < 0:
                                break
                            if (branchOffset + instructionOffset) < funcsStartsList[j] * 4:
                                vram = self.getVramOffset(funcsStartsList[j]*4)
                                funcSymbol = self.getSymbol(vram, tryPlusOffset=False, checkGlobalSegment=False)
                                if funcSymbol is not None and funcSymbol.isTrustableFunction(self.instrCat == rabbitizer.InstrCategory.RSP):
                                    j -= 1
                                    continue
                                del funcsStartsList[j]
                                del unimplementedInstructionsFuncList[j-1]
                            else:
                                break
                            j -= 1

            # Try to find the end of the function
            if currentFunctionSym is not None and currentFunctionSym.userDeclaredSize is not None:
                # If the function has a size set by the user then only use that and ignore the other ways of determining function-ends
                if instructionOffset + 8 == currentInstructionStart + currentFunctionSym.getSize():
                    functionEnded = True
            else:
                if not (farthestBranch > 0) and instr.isJump():
                    if instr.isReturn():
                        # Found a jr $ra and there are no branches outside of this function
                        functionEnded = True
                    elif instr.isJumptableJump():
                        # Usually jumptables, ignore
                        pass
                    elif not instr.doesLink():
                        if isLikelyHandwritten or self.instrCat == rabbitizer.InstrCategory.RSP:
                            # I don't remember the reasoning of this condition...
                            functionEnded = True

                # If there's another function after this then the current function has ended
                funcSymbol = self.getSymbol(currentVram + 8, tryPlusOffset=False, checkGlobalSegment=False)
                if funcSymbol is not None and funcSymbol.isTrustableFunction(self.instrCat == rabbitizer.InstrCategory.RSP):
                    if funcSymbol.vromAddress is None or self.getVromOffset(instructionOffset+8) == funcSymbol.vromAddress:
                        functionEnded = True

            index += 1
            farthestBranch -= 4
            instructionOffset += 4

        unimplementedInstructionsFuncList.append(not isInstrImplemented)
        return funcsStartsList, unimplementedInstructionsFuncList


    def analyze(self):
        instrsList = self.wordListToInstructions(self.words, self.getVramOffset(0), self.instrCat)
        nInstr = len(instrsList)

        funcsStartsList, unimplementedInstructionsFuncList = self._findFunctions(instrsList)

        previousSymbolExtraPadding = 0

        class GpRelativeVariable:
            def __init__(self, var_address : int, earliest_gp_relative_instr_address : int, latest__gp_relative_instr_address : int) -> None:
                self.var_address =var_address
                self.first_gp_access = earliest_gp_relative_instr_address
                self.last_gp_access = latest__gp_relative_instr_address
                pass
        
        var_dict : dict[int, GpRelativeVariable] = {}

        i = 0
        startsCount = len(funcsStartsList)
        for startIndex in range(startsCount):
            start = funcsStartsList[startIndex]
            hasUnimplementedIntrs = unimplementedInstructionsFuncList[startIndex]
            end = nInstr
            if startIndex + 1 < startsCount:
                end = funcsStartsList[startIndex+1]

            if start >= end:
                break

            localOffset = start*4
            vram = self.getVramOffset(localOffset)

            vrom = self.getVromOffset(localOffset)
            vromEnd = vrom + (end - start)*4

            if common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS or not hasUnimplementedIntrs:
                self.addFunction(vram, isAutogenerated=True, symbolVrom=vrom)
            else:
                self.addSymbol(vram, sectionType=self.sectionType, isAutogenerated=True, symbolVrom=vrom)

            self.symbolsVRams.add(vram)

            func = symbols.SymbolFunction(self.context, vrom, vromEnd, self.inFileOffset + localOffset, vram, instrsList[start:end], self.segmentVromStart, self.overlayCategory)
            func.setCommentOffset(self.commentOffset)
            func.index = i
            func.pointersOffsets |= self.pointersOffsets
            func.hasUnimplementedIntrs = hasUnimplementedIntrs
            func.parent = self
            func.isRsp = self.instrCat == rabbitizer.InstrCategory.RSP
            func.analyze()
            self.symbolList.append(func)

            for reloc_key in func.relocs:
                reloc_info = func.relocs[reloc_key]
                instr = func.instructions[reloc_key//4]
                if reloc_info.relocType == common.RelocType.MIPS_GPREL16:
                    address = reloc_info.symbol.address
                    if address not in var_dict:
                        var_dict[address] = GpRelativeVariable(address, func.vram, func.vram + (func.sizew * 4))
                    else:
                        var_dict[address].last_gp_access = func.vram + (func.sizew * 4)


            # File boundaries detection via padding detection
            if func.inFileOffset % 16 == 0:
                # Files are always 0x10 aligned

                if previousSymbolExtraPadding > 0:
                    self.fileBoundaries.append(func.inFileOffset)

            previousSymbolExtraPadding = func.countExtraPadding()
            i += 1
        def overlaps(a, b):
            return a.first_gp_access <= b.last_gp_access and b.first_gp_access <= a.last_gp_access
        
        def find_overlaps(var, global_var_list, ret_list) -> list[GpRelativeVariable]:
            for global_var in global_var_list:
                if overlaps(var, global_var):
                    ret_list.append(global_var)
            
            return ret_list
        overlapping_vars = []      
        for var in var_dict.values():
            # find overlapping address ranges between variables


            overlapping_vars = find_overlaps(var, var_dict.values(), overlapping_vars)
            

        # remove duplicates in overlapping_vars
        overlapping_vars = list(set(overlapping_vars))

        # sort by address
        overlapping_vars.sort(key=lambda x: x.first_gp_access)

        for var in overlapping_vars:
            print(f"Found small-data variable at {hex(var.var_address)} accessed between {hex(var.first_gp_access)} and {hex(var.last_gp_access)}")

        # merge overlapping address ranges in text section
        addressRanges = []
        for var in overlapping_vars:
            addressRanges.append((var.first_gp_access, var.last_gp_access))
        addressRanges = sorted(addressRanges, key=lambda x: x[0])
        mergedRanges = []
        for addressRange in addressRanges:
            if not mergedRanges:
                mergedRanges.append(addressRange)
            else:
                lastRange = mergedRanges[-1]
                if addressRange[0] <= lastRange[1]:
                    mergedRanges[-1] = (lastRange[0], max(lastRange[1], addressRange[1]))
                else:
                    mergedRanges.append(addressRange)
                    
        for addressRange in mergedRanges:
            print(f"Found possible file boundary at {hex(addressRange[0])}")
            
            
        # Filter out repeated values and sort
        self.fileBoundaries = sorted(set(self.fileBoundaries))


    def compareToFile(self, other: FileBase):
        result = super().compareToFile(other)

        if isinstance(other, SectionText):
            result["text"] = {
                "diff_opcode": self.countDiffOpcodes(other),
                "same_opcode_same_args": self.countSameOpcodeButDifferentArguments(other),
            }

        return result

    def countDiffOpcodes(self, other: SectionText) -> int:
        result = 0
        for i in range(min(self.nFuncs, other.nFuncs)):
            func = self.symbolList[i]
            other_func = other.symbolList[i]
            assert isinstance(func, symbols.SymbolFunction)
            assert isinstance(other_func, symbols.SymbolFunction)
            result += func.countDiffOpcodes(other_func)
        return result

    def countSameOpcodeButDifferentArguments(self, other: SectionText) -> int:
        result = 0
        for i in range(min(self.nFuncs, other.nFuncs)):
            func = self.symbolList[i]
            other_func = other.symbolList[i]
            assert isinstance(func, symbols.SymbolFunction)
            assert isinstance(other_func, symbols.SymbolFunction)
            result += func.countSameOpcodeButDifferentArguments(other_func)
        return result

    def blankOutDifferences(self, other_file: FileBase) -> bool:
        if not common.GlobalConfig.REMOVE_POINTERS:
            return False

        if not isinstance(other_file, SectionText):
            return False

        was_updated = False
        for i in range(min(self.nFuncs, other_file.nFuncs)):
            func = self.symbolList[i]
            other_func = other_file.symbolList[i]
            assert isinstance(func, symbols.SymbolFunction)
            assert isinstance(other_func, symbols.SymbolFunction)
            was_updated = func.blankOutDifferences(other_func) or was_updated

        return was_updated

    def removePointers(self) -> bool:
        if not common.GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        for func in self.symbolList:
            assert isinstance(func, symbols.SymbolFunction)
            was_updated = func.removePointers() or was_updated

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False

        if self.nFuncs > 0:
            func = self.symbolList[-1]
            assert isinstance(func, symbols.SymbolFunction)
            func.removeTrailingNops()
            was_updated = True

        return was_updated
