#!/usr/bin/python3

from __future__ import annotations

from .Utils import *
from .GlobalConfig import GlobalConfig
from .MipsFile import File
from .Instructions import InstructionBase, wordToInstruction
from .MipsFunction import Function


class Text(File):
    def __init__(self, array_of_bytes: bytearray, filename: str, version: str):
        super().__init__(array_of_bytes, filename, version)

        self.instructions: List[InstructionBase] = list()
        for word in self.words:
            self.instructions.append(wordToInstruction(word))

        self.functions: List[Function] = list()

        # TODO: do something with this information
        self.fileBoundaries: List[int] = list()

    @property
    def nInstr(self) -> int:
        return len(self.instructions)

    @property
    def nFuncs(self) -> int:
        return len(self.functions)

    def findFunctions(self):
        functionEnded = False
        farthestBranch = 0
        funcsStartsList = [0]

        index = 0
        nInstr = self.nInstr
        while index < nInstr:
            instr = self.instructions[index]
            if functionEnded:
                functionEnded = False
                index += 1
                isboundary = False
                while index < nInstr:
                    instr = self.instructions[index]
                    if instr.getOpcodeName() != "NOP":
                        if isboundary:
                            self.fileBoundaries.append(self.offset + index*4)
                        break
                    index += 1
                    isboundary = True
                funcsStartsList.append(index)
                if index >= len(self.instructions):
                    break
                instr = self.instructions[index]

            if instr.isBranch():
                branch = from2Complement(instr.immediate, 16) + 1
                if branch > farthestBranch:
                    # keep track of the farthest branch target
                    farthestBranch = branch
                if branch < 0:
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
        for startIndex in range(len(funcsStartsList)-1):
            start = funcsStartsList[startIndex]
            end = funcsStartsList[startIndex+1]

            funcName = f"func_{i}"
            vram = -1
            if self.vRamStart != -1:
                vram = self.getVramOffset(start*4)
                funcName = "func_" + toHex(self.getVramOffset(start*4), 6)[2:] + f" # {i}"

            func = Function(funcName, self.instructions[start:end], self.offset + start*4)
            func.vram = vram
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
        first_nop = self.nInstr
        # TODO consider moving this to Function
        for i in range(self.nInstr-1, 0-1, -1):
            instr = self.instructions[i]
            opcodeName = instr.getOpcodeName()
            if opcodeName != "NOP":
                if opcodeName == "JR" and instr.getRegisterName(instr.rs) == "$ra":
                    first_nop += 1
                break
            first_nop = i
        if first_nop < self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated

    def updateBytes(self):
        self.instructions = []
        for func in self.functions:
            self.instructions += func.instructions
        self.words = []
        for instr in self.instructions:
            self.words.append(instr.instr)
        super().updateBytes()

    def saveToFile(self, filepath: str):
        super().saveToFile(filepath + ".text")

        with open(filepath + ".text.asm", "w") as f:
            f.write(".section .text\n\n")
            i = 0
            offset = self.offset
            for func in self.functions:
                f.write(func.disassemble())
                f.write("\n")
                i += 1

def readMipsText(file: str, version: str) -> Text:
    filename = f"baserom_{version}/{file}"
    return Text(readFileAsBytearray(filename), filename, version)
