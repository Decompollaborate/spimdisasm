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
        super().__init__(context, vromStart, vromEnd, vram, filename, array_of_bytes, common.FileSectionType.Text, segmentVromStart, overlayCategory)


    @property
    def nFuncs(self) -> int:
        return len(self.symbolList)

    @staticmethod
    def wordListToInstructions(wordList: list[int], currentVram: int|None, isRsp: bool=False) -> list[rabbitizer.Instruction]:
        instrsList: list[rabbitizer.Instruction] = list()
        for word in wordList:
            instrCat = rabbitizer.InstrCategory.CPU
            if isRsp:
                instrCat = rabbitizer.InstrCategory.RSP
            instr = rabbitizer.Instruction(word, category=instrCat)

            if currentVram is not None:
                instr.vram = currentVram
                currentVram += 4

            instrsList.append(instr)
        return instrsList

    def analyze(self):
        functionEnded = False
        farthestBranch = 0
        funcsStartsList = [0]
        unimplementedInstructionsFuncList = []

        instrsList = self.wordListToInstructions(self.words, self.getVramOffset(0), self.isRsp)

        instructionOffset = 0
        currentInstructionStart = 0
        currentFunctionSym = self.getSymbol(self.getVramOffset(instructionOffset), tryPlusOffset=False)

        isLikelyHandwritten = self.isHandwritten

        isInstrImplemented = True
        index = 0
        nInstr = len(instrsList)
        while index < nInstr:
            instr = instrsList[index]
            if not instr.isImplemented():
                isInstrImplemented = False

            if functionEnded:
                functionEnded = False

                isLikelyHandwritten = self.isHandwritten
                index += 1
                instructionOffset += 4
                isboundary = False
                # Loop over until we find a instruction that isn't a nop
                while index < nInstr:
                    instr = instrsList[index]
                    if not instr.isNop():
                        if isboundary:
                            self.fileBoundaries.append(self.inFileOffset + index*4)
                        break
                    index += 1
                    instructionOffset += 4
                    isboundary = True

                currentInstructionStart = instructionOffset
                currentFunctionSym = self.getSymbol(self.getVramOffset(instructionOffset), tryPlusOffset=False)

                funcsStartsList.append(index)
                unimplementedInstructionsFuncList.append(not isInstrImplemented)
                if index >= len(instrsList):
                    break
                instr = instrsList[index]
                isInstrImplemented = instr.isImplemented()

            currentVram = self.getVramOffset(instructionOffset)

            if not self.isRsp and not isLikelyHandwritten:
                isLikelyHandwritten = instr.isLikelyHandwritten()

            if instr.isBranch() or instr.isUnconditionalBranch():
                branchOffset = instr.getGenericBranchOffset(currentVram)
                if branchOffset > farthestBranch:
                    # keep track of the farthest branch target
                    farthestBranch = branchOffset
                if branchOffset < 0:
                    if branchOffset + instructionOffset < 0:
                        # Whatever we are reading is not a valid instruction
                        break
                    # make sure to not branch outside of the current function
                    if not isLikelyHandwritten:
                        j = len(funcsStartsList) - 1
                        while j >= 0:
                            if (branchOffset + instructionOffset) < funcsStartsList[j] * 4:
                                vram = self.getVramOffset(funcsStartsList[j]*4)
                                funcSymbol = self.getSymbol(vram, tryPlusOffset=False)
                                if funcSymbol is not None and funcSymbol.isTrustableFunction(self.isRsp):
                                    j -= 1
                                    continue
                                del funcsStartsList[j]
                                del unimplementedInstructionsFuncList[j-1]
                            else:
                                break
                            j -= 1

            elif instr.isJType():
                target = instr.getInstrIndexAsVram()
                if not self.isRsp:
                    if target >= 0x84000000:
                        # RSP address space?
                        isLikelyHandwritten = True
                self.addFunction(target, isAutogenerated=True)

            if not (farthestBranch > 0) and instr.isJump():
                if instr.isJrRa():
                    functionEnded = True
                elif instr.isJrNotRa():
                    pass
                elif not instr.doesLink():
                    if isLikelyHandwritten or self.isRsp:
                        functionEnded = True

            funcSymbol = self.getSymbol(currentVram + 8, tryPlusOffset=False)
            if funcSymbol is not None and funcSymbol.isTrustableFunction(self.isRsp):
                if funcSymbol.vromAddress is None or self.getVromOffset(instructionOffset+8) == funcSymbol.vromAddress:
                    functionEnded = True

            if currentFunctionSym is not None and currentFunctionSym.size is not None:
                if instructionOffset + 8 == currentInstructionStart + currentFunctionSym.getSize():
                    functionEnded = True

            index += 1
            farthestBranch -= 4
            instructionOffset += 4

        unimplementedInstructionsFuncList.append(not isInstrImplemented)

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

            # TODO: wire up back
            # funcName = f"func_{i}"
            # if len(self.context.offsetSymbols[self.sectionType]) > 0:
            #     possibleFuncName = self.context.getOffsetSymbol(start*4, self.sectionType)
            #     if possibleFuncName is not None:
            #         funcName = possibleFuncName.getName()

            localOffset = start*4
            vram = self.getVramOffset(localOffset)

            vrom = self.getVromOffset(localOffset)
            vromEnd = vrom + (end - start)*4

            if common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS or not hasUnimplementedIntrs:
                funcSymbol = self.addFunction(vram, isAutogenerated=True, symbolVrom=vrom)
            elif common.GlobalConfig.ADD_NEW_SYMBOLS:
                self.addSymbol(vram, sectionType=self.sectionType, isAutogenerated=True, symbolVrom=vrom)

            self.symbolsVRams.add(vram)

            func = symbols.SymbolFunction(self.context, vrom, vromEnd, self.inFileOffset + localOffset, vram, instrsList[start:end], self.segmentVromStart, self.overlayCategory)
            func.setCommentOffset(self.commentOffset)
            func.index = i
            func.pointersOffsets |= self.pointersOffsets
            func.hasUnimplementedIntrs = hasUnimplementedIntrs
            func.parent = self
            func.isRsp = self.isRsp
            func.analyze()
            self.symbolList.append(func)
            i += 1


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
