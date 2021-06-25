#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFile import File
from .Instructions import InstructionBase, wordToInstruction
from .MipsFunction import Function
from .MipsContext import Context


class Text(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str, context: Context):
        super().__init__(array_of_bytes, filename, version, context)

        self.functions: List[Function] = list()

        # TODO: do something with this information
        self.fileBoundaries: List[int] = list()

    @property
    def nFuncs(self) -> int:
        return len(self.functions)

    def findFunctions(self):
        functionEnded = False
        farthestBranch = 0
        funcsStartsList = [0]

        instructions: List[InstructionBase] = list()
        for word in self.words:
            instructions.append(wordToInstruction(word))

        index = 0
        nInstr = len(instructions)
        while index < nInstr:
            instr = instructions[index]
            if functionEnded:
                functionEnded = False
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
                if index >= len(instructions):
                    break
                instr = instructions[index]

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
                        else:
                            break
                        j -= 1

            if not (farthestBranch > 0):
                opcodeName = instr.getOpcodeName()
                if opcodeName == "JR" and instr.getRegisterName(instr.rs) == "$ra":
                    functionEnded = True
                #elif opcodeName == "J":
                #    functionEnded = True

            index += 1
            farthestBranch -= 1

        i = 0
        startsCount = len(funcsStartsList)
        for startIndex in range(startsCount):
            start = funcsStartsList[startIndex]
            end = nInstr
            if startIndex + 1 < startsCount:
                end = funcsStartsList[startIndex+1]

            if start >= end:
                break

            funcName = f"func_{i}"
            vram = -1
            if self.vRamStart >= 0:
                vram = self.getVramOffset(start*4)
                if vram in self.context.funcAddresses:
                    funcName = self.context.funcAddresses[vram]
                else:
                    funcName = "func_" + toHex(self.getVramOffset(start*4), 6)[2:]

                if vram not in self.context.funcAddresses:
                    self.context.funcAddresses[vram] = funcName

            func = Function(funcName, instructions[start:end], self.context, self.offset + start*4, vram=vram)
            func.index = i
            self.functions.append(func)
            i += 1

    def compareToFile(self, other: File):
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

    def blankOutDifferences(self, other_file: File) -> bool:
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

        with open(filepath + ".text.asm", "w") as f:
            f.write(".section .text\n\n")
            for func in self.functions:
                f.write(func.disassemble())
                f.write("\n")

def readMipsText(file: str, version: str) -> Text:
    filename = f"baserom_{version}/{file}"
    return Text(readFileAsBytearray(filename), filename, version, Context())
