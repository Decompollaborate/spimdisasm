#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig, printVerbose
from ..common.Context import Context
from ..common.FileSectionType import FileSectionType

from .MipsFileBase import FileBase
from .MipsSection import Section
from .Instructions import InstructionBase, wordToInstruction, wordToInstructionRsp, InstructionId, InstructionCoprocessor0, InstructionCoprocessor2
from .MipsFunction import Function


class Text(Section):
    def __init__(self, context: Context, vram: int|None, filename: str, array_of_bytes: bytearray):
        super().__init__(context, vram, filename, array_of_bytes, FileSectionType.Text)

        # TODO: do something with this information
        self.fileBoundaries: List[int] = list()

    @property
    def nFuncs(self) -> int:
        return len(self.symbolList)

    def analyze(self):
        functionEnded = False
        farthestBranch = 0
        funcsStartsList = [0]
        unimplementedInstructionsFuncList = []

        instructions: List[InstructionBase] = list()
        for word in self.words:
            if self.isRsp:
                instr = wordToInstructionRsp(word)
            else:
                instr = wordToInstruction(word)
            instructions.append(instr)

        trackedRegisters: Dict[int, int] = dict()
        registersValues: Dict[int, int] = dict()
        instructionOffset = 0

        isLikelyHandwritten = self.isHandwritten
        newFunctions = list()

        isInstrImplemented = True
        index = 0
        nInstr = len(instructions)
        while index < nInstr:
            instr = instructions[index]
            if not instr.isImplemented():
                isInstrImplemented = False

            if functionEnded:
                functionEnded = False

                if not isLikelyHandwritten or self.isRsp:
                    for isFake, targetVram, targetFuncName in newFunctions:
                        if isFake:
                            self.context.addFakeFunction(targetVram, targetFuncName)
                        else:
                            self.context.addFunction(targetVram, targetFuncName)

                newFunctions.clear()

                isLikelyHandwritten = self.isHandwritten
                index += 1
                instructionOffset += 4
                isboundary = False
                while index < nInstr:
                    instr = instructions[index]
                    if instr.uniqueId != InstructionId.NOP:
                        if isboundary:
                            self.fileBoundaries.append(self.inFileOffset + index*4)
                        break
                    index += 1
                    instructionOffset += 4
                    isboundary = True

                trackedRegisters.clear()
                registersValues.clear()

                funcsStartsList.append(index)
                unimplementedInstructionsFuncList.append(not isInstrImplemented)
                if index >= len(instructions):
                    break
                instr = instructions[index]
                isInstrImplemented = instr.isImplemented()

            if not self.isRsp and not isLikelyHandwritten:
                if isinstance(instr, InstructionCoprocessor2):
                    isLikelyHandwritten = True
                elif isinstance(instr, InstructionCoprocessor0):
                    isLikelyHandwritten = True
                elif instr.getRegisterName(instr.rs) in ("$k0", "$k1"):
                    isLikelyHandwritten = True
                elif instr.getRegisterName(instr.rt) in ("$k0", "$k1"):
                    isLikelyHandwritten = True

            if instr.isBranch():
                branch = from2Complement(instr.immediate, 16) + 1
                if branch > farthestBranch:
                    # keep track of the farthest branch target
                    farthestBranch = branch
                if branch < 0:
                    if branch + index < 0:
                        # Whatever we are reading is not a valid instruction
                        break
                    # make sure to not branch outside of the current function
                    if not isLikelyHandwritten:
                        j = len(funcsStartsList) - 1
                        while j >= 0:
                            if index + branch < funcsStartsList[j]:
                                if GlobalConfig.TRUST_USER_FUNCTIONS or (GlobalConfig.DISASSEMBLE_RSP and self.isRsp):
                                    vram = self.getVramOffset(funcsStartsList[j]*4)
                                    if self.context.getFunction(vram) is not None:
                                        j -= 1
                                        continue
                                del funcsStartsList[j]
                                del unimplementedInstructionsFuncList[j-1]
                            else:
                                break
                            j -= 1

            elif instr.isIType():
                isLui = instr.uniqueId == InstructionId.LUI
                if isLui:
                    if instr.immediate >= 0x4000: # filter out stuff that may not be a real symbol
                        trackedRegisters[instr.rt] = instructionOffset//4
                elif instr.isIType() and instr.uniqueId not in (InstructionId.ANDI, InstructionId.ORI, InstructionId.XORI, InstructionId.CACHE):
                    rs = instr.rs
                    if rs in trackedRegisters:
                        luiInstr = instructions[trackedRegisters[rs]]
                        upperHalf = luiInstr.immediate << 16
                        lowerHalf = from2Complement(instr.immediate, 16)
                        registersValues[instr.rt] = upperHalf + lowerHalf

            elif instr.isJType():
                target = instr.getInstrIndexAsVram()
                if instr.uniqueId == InstructionId.J and not self.isRsp:
                    # newFunctions.append((True, target, f"fakefunc_{target:08X}"))
                    newFunctions.append((True, target, f".L{target:08X}"))
                else:
                    newFunctions.append((False, target, f"func_{target:08X}"))

            if not (farthestBranch > 0):
                if instr.uniqueId == InstructionId.JR:
                    if instr.getRegisterName(instr.rs) == "$ra":
                        functionEnded = True
                    else:
                        if instr.rs in registersValues:
                            functionEnded = True
                elif instr.uniqueId == InstructionId.J and (isLikelyHandwritten or (GlobalConfig.DISASSEMBLE_RSP and self.isRsp)):
                    functionEnded = True

            if self.vram is not None:
                if GlobalConfig.TRUST_USER_FUNCTIONS or (GlobalConfig.DISASSEMBLE_RSP and self.isRsp):
                    vram = self.getVramOffset(instructionOffset) + 8
                    funcContext = self.context.getFunction(vram)
                    if funcContext is not None:
                        if funcContext.isUserDefined or (GlobalConfig.DISASSEMBLE_RSP and self.isRsp):
                            functionEnded = True

            index += 1
            farthestBranch -= 1
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

            funcName = f"func_{i}"
            possibleFuncName = self.context.getOffsetSymbol(start*4, FileSectionType.Text)
            if possibleFuncName is not None:
                funcName = possibleFuncName.name

            vram = None
            if self.vram is not None:
                vram = self.getVramOffset(start*4)
                funcSymbol = self.context.getFunction(vram)
                if funcSymbol is not None:
                    funcName = funcSymbol.name
                else:
                    funcName = "func_" + toHex(vram, 6)[2:]

                if GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS or not hasUnimplementedIntrs:
                    self.context.addFunction(vram, funcName)
                    funcSymbol = self.context.getFunction(vram)
                    if funcSymbol is not None:
                        funcSymbol.isDefined = True
                else:
                    if vram in self.context.symbols:
                        self.context.symbols[vram].isDefined = True
                    elif GlobalConfig.ADD_NEW_SYMBOLS:
                        contextSym = self.context.addSymbol(vram, None)
                        contextSym.isAutogenerated = True
                        contextSym.isDefined = True

            func = Function(self.context, self.inFileOffset + start*4, vram, funcName, instructions[start:end])
            func.index = i
            func.pointersOffsets |= self.pointersOffsets
            func.hasUnimplementedIntrs = hasUnimplementedIntrs
            func.parent = self
            func.isRsp = self.isRsp
            func.analyze()
            self.symbolList.append(func)
            i += 1

    def printAnalyzisResults(self):
        super().printAnalyzisResults()
        if not GlobalConfig.VERBOSE:
            return


        printVerbose(f"Found {self.nFuncs} functions.")

        nBoundaries = len(self.fileBoundaries)
        if nBoundaries > 0:
            printVerbose(f"Found {nBoundaries} file boundaries.")

            for i in range(len(self.fileBoundaries)-1):
                start = self.fileBoundaries[i]
                end = self.fileBoundaries[i+1]

                functionsInBoundary = 0
                for func in self.symbolList:
                    if func.vram is not None and self.vram is not None:
                        funcOffset = func.vram - self.vram
                        if start <= funcOffset < end:
                            functionsInBoundary += 1
                fileVram = 0
                if self.vram is not None:
                    fileVram = start + self.vram
                printVerbose("\t", toHex(start+self.commentOffset, 6)[2:], toHex(end-start, 4)[2:], toHex(fileVram, 8)[2:], "\t functions:", functionsInBoundary)

            start = self.fileBoundaries[-1]
            end = self.sizew*4 + self.inFileOffset

            functionsInBoundary = 0
            for func in self.symbolList:
                if func.vram is not None and self.vram is not None:
                    funcOffset = func.vram - self.vram
                    if start <= funcOffset < end:
                        functionsInBoundary += 1
            fileVram = 0
            if self.vram is not None:
                fileVram = start + self.vram
            printVerbose("\t", toHex(start+self.commentOffset, 6)[2:], toHex(end-start, 4)[2:], toHex(fileVram, 8)[2:], "\t functions:", functionsInBoundary)

            printVerbose()
        return


    def compareToFile(self, other: FileBase):
        result = super().compareToFile(other)

        if isinstance(other, Text):
            result["text"] = {
                "diff_opcode": self.countDiffOpcodes(other),
                "same_opcode_same_args": self.countSameOpcodeButDifferentArguments(other),
            }

        return result

    def countDiffOpcodes(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nFuncs, other.nFuncs)):
            func = self.symbolList[i]
            other_func = other.symbolList[i]
            assert isinstance(func, Function)
            assert isinstance(other_func, Function)
            result += func.countDiffOpcodes(other_func)
        return result

    def countSameOpcodeButDifferentArguments(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nFuncs, other.nFuncs)):
            func = self.symbolList[i]
            other_func = other.symbolList[i]
            assert isinstance(func, Function)
            assert isinstance(other_func, Function)
            result += func.countSameOpcodeButDifferentArguments(other_func)
        return result

    def blankOutDifferences(self, other_file: FileBase) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        if not isinstance(other_file, Text):
            return False

        was_updated = False
        for i in range(min(self.nFuncs, other_file.nFuncs)):
            func = self.symbolList[i]
            other_func = other_file.symbolList[i]
            assert isinstance(func, Function)
            assert isinstance(other_func, Function)
            was_updated = func.blankOutDifferences(other_func) or was_updated

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        for func in self.symbolList:
            assert isinstance(func, Function)
            was_updated = func.removePointers() or was_updated

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False

        if self.nFuncs > 0:
            func = self.symbolList[-1]
            assert isinstance(func, Function)
            func.removeTrailingNops()
            was_updated = True

        return was_updated
