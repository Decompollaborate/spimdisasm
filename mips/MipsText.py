#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFileBase import FileBase
from .MipsSection import Section
from .Instructions import InstructionBase, wordToInstruction, InstructionCoprocessor0
from .MipsFunction import Function
from .MipsContext import Context


class Text(Section):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, context: Context):
        super().__init__(array_of_bytes, filename, version, context)

        self.functions: List[Function] = list()

        # TODO: do something with this information
        self.fileBoundaries: List[int] = list()

    @property
    def nFuncs(self) -> int:
        return len(self.functions)

    def analyze(self):
        functionEnded = False
        farthestBranch = 0
        funcsStartsList = [0]
        unimplementedInstructionsFuncList = []

        instructions: List[InstructionBase] = list()
        for word in self.words:
            instructions.append(wordToInstruction(word))

        trackedRegisters: Dict[int, int] = dict()
        registersValues: Dict[int, int] = dict()
        instructionOffset = 0

        isLikelyHandwritten = False

        isInstrImplemented = True
        index = 0
        nInstr = len(instructions)
        while index < nInstr:
            instr = instructions[index]
            if not instr.isImplemented():
                isInstrImplemented = False

            if functionEnded:
                functionEnded = False
                isLikelyHandwritten = False
                index += 1
                isboundary = False
                while index < nInstr:
                    instr = instructions[index]
                    if instr.getOpcodeName() != "NOP":
                        if isboundary:
                            self.fileBoundaries.append(self.offset + index*4)
                        break
                    index += 1
                    isboundary = True
                funcsStartsList.append(index)
                unimplementedInstructionsFuncList.append(not isInstrImplemented)
                if index >= len(instructions):
                    break
                instr = instructions[index]
                isInstrImplemented = instr.isImplemented()

            if not isLikelyHandwritten:
                opcodeName = instr.getOpcodeName()
                if opcodeName in ("COP2",):
                    isLikelyHandwritten = True
                    isInstrImplemented = False
                elif isinstance(instr, InstructionCoprocessor0):
                #if isinstance(instr, InstructionCoprocessor0):
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
                    j = len(funcsStartsList) - 1
                    while j >= 0:
                        if index + branch < funcsStartsList[j]:
                            del funcsStartsList[j]
                            del unimplementedInstructionsFuncList[j-1]
                        else:
                            break
                        j -= 1

            elif instr.isIType():
                opcodeName = instr.getOpcodeName()
                isLui = opcodeName == "LUI"
                if isLui:
                    if instr.immediate >= 0x4000: # filter out stuff that may not be a real symbol
                        trackedRegisters[instr.rt] = instructionOffset//4
                elif instr.isIType() and opcodeName not in ("ANDI", "ORI", "XORI", "CACHE"):
                    rs = instr.rs
                    if rs in trackedRegisters:
                        luiInstr = instructions[trackedRegisters[rs]]
                        upperHalf = luiInstr.immediate << 16
                        lowerHalf = from2Complement(instr.immediate, 16)
                        registersValues[instr.rt] = upperHalf + lowerHalf

            if not (farthestBranch > 0):
                opcodeName = instr.getOpcodeName()
                if opcodeName == "JR":
                    if instr.getRegisterName(instr.rs) == "$ra":
                        functionEnded = True
                        trackedRegisters.clear()
                        registersValues.clear()
                    else:
                        if instr.rs in registersValues:
                            functionEnded = True
                            trackedRegisters.clear()
                            registersValues.clear()
                elif opcodeName == "J" and isLikelyHandwritten:
                    functionEnded = True
                    trackedRegisters.clear()
                    registersValues.clear()

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
            vram = -1
            if self.vRamStart >= 0:
                vram = self.getVramOffset(start*4)
                funcSymbol = self.context.getFunction(vram)
                if funcSymbol is not None:
                    funcName = funcSymbol.name
                else:
                    funcName = "func_" + toHex(self.getVramOffset(start*4), 6)[2:]

                if not hasUnimplementedIntrs:
                    self.context.addFunction(self.filename, vram, funcName)
                    funcSymbol = self.context.getFunction(vram)
                    if funcSymbol is not None:
                        funcSymbol.isDefined = True

            func = Function(funcName, instructions[start:end], self.context, self.offset + start*4, vram=vram)
            func.index = i
            func.pointersOffsets += self.pointersOffsets
            func.hasUnimplementedIntrs = hasUnimplementedIntrs
            func.analyze()
            self.functions.append(func)
            i += 1

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
            func = self.functions[i]
            other_func = other.functions[i]
            result += func.countDiffOpcodes(other_func)
        return result

    def countSameOpcodeButDifferentArguments(self, other: Text) -> int:
        result = 0
        for i in range(min(self.nFuncs, other.nFuncs)):
            func = self.functions[i]
            other_func = other.functions[i]
            result += func.countSameOpcodeButDifferentArguments(other_func)
        return result

    def blankOutDifferences(self, other_file: FileBase) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        if not isinstance(other_file, Text):
            return False

        was_updated = False
        for i in range(min(self.nFuncs, other_file.nFuncs)):
            func = self.functions[i]
            other_func = other_file.functions[i]
            was_updated = func.blankOutDifferences(other_func) or was_updated

        return was_updated

    def removePointers(self) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
            return False

        was_updated = False
        for func in self.functions:
            was_updated = func.removePointers() or was_updated

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False

        if self.nFuncs > 0:
            self.functions[-1].removeTrailingNops()
            was_updated = True

        return was_updated

    def updateBytes(self):
        self.words = []
        for func in self.functions:
            for instr in func.instructions:
                self.words.append(instr.instr)
        super().updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".text")

        with open(filepath + ".text.s", "w") as f:
            f.write(".include \"macro.inc\"\n")
            f.write("\n")
            f.write("# assembler directives\n")
            f.write(".set noat      # allow manual use of $at\n")
            f.write(".set noreorder # don't insert nops after branches\n")
            f.write(".set gp=64     # allow use of 64-bit general purpose registers\n")
            f.write("\n")
            f.write(".section .text\n")
            f.write("\n")
            f.write(".balign 16\n")
            for func in self.functions:
                f.write("\n")
                f.write(func.disassemble())

    def setCommentOffset(self, commentOffset: int):
        super().setCommentOffset(commentOffset)
        for func in self.functions:
            func.commentOffset = commentOffset

def readMipsText(file: str, version: str) -> Text:
    filename = f"baserom_{version}/{file}"
    return Text(readFileAsBytearray(filename), filename, version, Context())
