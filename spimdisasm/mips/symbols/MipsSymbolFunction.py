#!/usr/bin/env python3

# SPDX-FileCopyrightText: © 2022 Decompollaborate
# SPDX-License-Identifier: MIT

from __future__ import annotations

import rabbitizer

from ... import common

from . import SymbolText, analysis


class SymbolFunction(SymbolText):
    def __init__(self, context: common.Context, vromStart: int, vromEnd: int, inFileOffset: int, vram: int, instrsList: list[rabbitizer.Instruction], segmentVromStart: int, overlayCategory: str|None):
        super().__init__(context, vromStart, vromEnd, inFileOffset, vram, list(), segmentVromStart, overlayCategory)
        self.instructions = list(instrsList)

        self.instrAnalyzer = analysis.InstrAnalyzer(self.vram)

        self.branchesTaken: set[int] = set()

        self.pointersOffsets: set[int] = set()
        self.pointersRemoved: bool = False

        self.hasUnimplementedIntrs: bool = False
        self.isRsp: bool = False
        self.isLikelyHandwritten: bool = False

    @property
    def nInstr(self) -> int:
        return len(self.instructions)

    @property
    def sizew(self) -> int:
        return self.nInstr


    def _lookAheadSymbolFinder(self, instr: rabbitizer.Instruction, prevInstr: rabbitizer.Instruction, instructionOffset: int, trackedRegistersOriginal: rabbitizer.RegistersTracker):
        if not prevInstr.isBranch() and not prevInstr.isUnconditionalBranch():
            return

        currentVram = self.getVramOffset(instructionOffset)

        prevInstrOffset = instructionOffset - 4
        prevVram = self.getVramOffset(prevInstrOffset)
        branchOffset = prevInstr.getGenericBranchOffset(prevVram)
        branch = prevInstrOffset + branchOffset

        if branch < 0:
            # Avoid jumping outside of the function
            return

        regsTracker = rabbitizer.RegistersTracker(trackedRegistersOriginal)

        self.instrAnalyzer.processInstr(regsTracker, instr, instructionOffset, currentVram)

        if instructionOffset in self.branchesTaken:
            return
        self.branchesTaken.add(instructionOffset)

        sizew = len(self.instructions)*4
        while branch < sizew:
            prevTargetInstr = self.instructions[branch//4 - 1]
            targetInstr = self.instructions[branch//4]

            self.instrAnalyzer.processInstr(regsTracker, targetInstr, branch, self.getVramOffset(branch), prevTargetInstr)

            if prevTargetInstr.isUnconditionalBranch():
                return
            if prevTargetInstr.isJump() and not prevTargetInstr.doesLink():
                return

            self.instrAnalyzer.processPrevFuncCall(regsTracker, targetInstr, prevTargetInstr)
            branch += 4

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

                    if instructionOffset in self.instrAnalyzer.symbolInstrOffset:
                        if instructionOffset in self.instrAnalyzer.referencedJumpTableOffsets:
                            # Jump tables
                            addressOffset = self.instrAnalyzer.symbolInstrOffset[instructionOffset]
                            if relocSymbol.name != ".rodata":
                                common.Utils.eprint(f"Warning. Jumptable referenced in reloc does not have '.rodata' as its name")
                            contextOffsetSym = self.context.addOffsetJumpTable(addressOffset, sectType)
                            contextOffsetSym.referenceCounter += 1
                            relocSymbol.name = contextOffsetSym.name
                            self.instrAnalyzer.symbolInstrOffset[instructionOffset] = 0
                            if instructionOffset in self.instrAnalyzer.lowToHiDict:
                                luiOffset = self.instrAnalyzer.lowToHiDict[instructionOffset]
                                otherReloc = self.context.getRelocSymbol(self.inFileOffset+luiOffset, common.FileSectionType.Text)
                                if otherReloc is not None:
                                    otherReloc.name = relocSymbol.name
                                    self.instrAnalyzer.symbolInstrOffset[luiOffset] = 0
                        else:
                            addressOffset = self.instrAnalyzer.symbolInstrOffset[instructionOffset]
                            relocName = f"{relocSymbol.name}_{addressOffset:06X}"
                            # print(relocName, addressOffset, instr)
                            contextOffsetSym = common.ContextOffsetSymbol(addressOffset, relocName, sectType)
                            self.context.offsetSymbols[sectType][addressOffset] = contextOffsetSym
                            relocSymbol.name = relocName
                            self.instrAnalyzer.symbolInstrOffset[instructionOffset] = 0
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

        regsTracker = rabbitizer.RegistersTracker()

        instructionOffset = 0
        for instr in self.instructions:
            currentVram = self.getVramOffset(instructionOffset)
            prevInstr = self.instructions[instructionOffset//4 - 1]

            self.instrAnalyzer.printAnalisisDebugInfo_IterInfo(regsTracker, instr, currentVram)

            if not self.isLikelyHandwritten:
                self.isLikelyHandwritten = instr.isLikelyHandwritten()

            if not common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS and not instr.isImplemented():
                # Abort analysis
                self.hasUnimplementedIntrs = True
                return

            if not prevInstr.isBranchLikely() and not prevInstr.isUnconditionalBranch():
                self.instrAnalyzer.processInstr(regsTracker, instr, instructionOffset, currentVram, prevInstr)

            # look-ahead symbol finder
            self._lookAheadSymbolFinder(instr, prevInstr, instructionOffset, regsTracker)

            self.instrAnalyzer.processPrevFuncCall(regsTracker, instr, prevInstr, currentVram)

            instructionOffset += 4

        self.instrAnalyzer.printSymbolFinderDebugInfo_UnpairedLuis()

        self._processElfRelocSymbols()

        # Branches
        for instrOffset, targetBranchVram in self.instrAnalyzer.branchInstrOffsets.items():
            branch = self.instrAnalyzer.branchTargetInstrOffsets[instrOffset]
            labelSym = self.addBranchLabel(targetBranchVram, isAutogenerated=True, symbolVrom=self.getVromOffset(branch))
            labelSym.referenceCounter += 1

        # Function calls
        for targetVram in self.instrAnalyzer.funcCallInstrOffsets.values():
            funcSym = self.addFunction(targetVram, isAutogenerated=True)
            funcSym.referenceCounter += 1

        if not self.isRsp and len(self.instrAnalyzer.funcCallOutsideRangesOffsets) > 0:
            self.isLikelyHandwritten = True

        # Symbols
        for loOffset, symVram in self.instrAnalyzer.symbolLoInstrOffset.items():
            if symVram in self.context.bannedSymbols:
                continue

            # Check for user-defined symbol patches
            patchedAddress = self.getLoPatch(self.getVramOffset(loOffset))
            if patchedAddress is not None:
                symVram = patchedAddress

            symType = self.instrAnalyzer.possibleSymbolTypes.get(symVram, None)
            contextSym = self.getSymbol(symVram)
            if contextSym is None:
                if not common.GlobalConfig.ADD_NEW_SYMBOLS:
                    continue
                contextSym = self.addSymbol(symVram, isAutogenerated=True)
            else:
                # TODO: do this in a less ugly way
                if contextSym.address != symVram:
                    if contextSym.address % 4 != 0 or symVram % 4 != 0:
                        if contextSym.getType() in {"u16", "s16", "u8", "u8"} or symType in {"u16", "s16", "u8", "u8"}:
                            if not (contextSym.getSize() > 4):
                                if common.GlobalConfig.ADD_NEW_SYMBOLS:
                                    contextSym = self.addSymbol(symVram, isAutogenerated=True)

            contextSym.referenceCounter += 1
            if symType is not None:
                contextSym.setTypeIfUnset(symType)

        # Jump tables
        for targetVram in self.instrAnalyzer.jumpRegisterIntrOffset.values():
            self.addJumpTable(targetVram, isAutogenerated=True)

        for instr in self.instructions:
            instr.inHandwrittenFunction = self.isLikelyHandwritten


    def countExtraPadding(self) -> int:
        count = 0
        for i in range(len(self.instructions)-1, 0, -1):
            instr = self.instructions[i]
            nextInstr = self.instructions[i-1]

            if nextInstr.uniqueId == rabbitizer.InstrId.cpu_jr:
                return count

            if not instr.isNop():
                return count

            count += 1
        return count


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

        for instructionOffset in self.instrAnalyzer.symbolInstrOffset:
            self.instructions[instructionOffset//4].blankOut()
        was_updated = len(self.instrAnalyzer.symbolInstrOffset) > 0 or was_updated

        for fileOffset in self.pointersOffsets:
            index = (fileOffset - self.inFileOffset)//4
            if index < 0:
                continue
            if index >= self.nInstr:
                continue
            self.instructions[index].blankOut()

        if common.GlobalConfig.IGNORE_BRANCHES:
            for instructionOffset in self.instrAnalyzer.branchInstrOffsets:
                self.instructions[instructionOffset//4].blankOut()
            was_updated = len(self.instrAnalyzer.branchInstrOffsets) > 0 or was_updated

        self.pointersRemoved = True

        return was_updated

    def removeTrailingNops(self) -> bool:
        was_updated = False
        first_nop = self.nInstr

        for i in range(self.nInstr-1, 0-1, -1):
            instr = self.instructions[i]
            if not instr.isNop():
                if instr.isJrRa():
                    first_nop += 1
                break
            first_nop = i

        if first_nop < self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated


    def generateHiLoStr(self, instr: rabbitizer.Instruction, symName: str) -> str:
        if instr.canBeHi():
            return f"%hi({symName})"

        if instr.rs in {rabbitizer.RegGprO32.gp, rabbitizer.RegGprN32.gp}:
            if instr.rt in {rabbitizer.RegGprO32.gp, rabbitizer.RegGprN32.gp} or not instr.modifiesRt():
                return f"%gp_rel({symName})"

        return f"%lo({symName})"

    def getImmOverrideForInstruction(self, instr: rabbitizer.Instruction, instructionOffset: int) -> str|None:
        if len(self.context.relocSymbols[self.sectionType]) > 0:
            # Check possible symbols using reloc information (probably from a .o elf file)
            possibleImmOverride = self.context.getRelocSymbol(self.inFileOffset + instructionOffset, self.sectionType)
            if possibleImmOverride is not None:
                auxOverride = possibleImmOverride.getName()
                if instr.isIType():
                    if instructionOffset in self.instrAnalyzer.symbolInstrOffset:
                        addressOffset = self.instrAnalyzer.symbolInstrOffset[instructionOffset]
                        auxOverride = possibleImmOverride.getNamePlusOffset(addressOffset)

                    auxOverride = self.generateHiLoStr(instr, auxOverride)
                return auxOverride

        if instr.isBranch() or instr.isUnconditionalBranch():
            if not common.GlobalConfig.IGNORE_BRANCHES:
                branchOffset = instr.getGenericBranchOffset(self.getVramOffset(instructionOffset))
                targetBranchVram = self.getVramOffset(instructionOffset + branchOffset)
                labelSymbol = self.getSymbol(targetBranchVram, tryPlusOffset=False)
                if labelSymbol is not None:
                    return labelSymbol.getName()

        elif instr.isIType():
            if not self.pointersRemoved and instructionOffset in self.instrAnalyzer.symbolInstrOffset:
                address = self.instrAnalyzer.symbolInstrOffset[instructionOffset]

                if address in self.context.bannedSymbols:
                    return None

                instrVram = self.getVramOffset(instructionOffset)
                if instr.canBeHi():
                    # we need to get the address of the lo instruction to get the patch
                    if instructionOffset in self.instrAnalyzer.hiToLowDict:
                        instrVram = self.getVramOffset(self.instrAnalyzer.hiToLowDict[instructionOffset])

                # Check for user-defined symbol patches
                patchedAddress = self.getLoPatch(instrVram)
                if patchedAddress is not None:
                    symbol = self.getSymbol(patchedAddress, tryPlusOffset=True, checkUpperLimit=False)
                else:
                    symbol = self.getSymbol(address, tryPlusOffset=True)

                if symbol is not None:
                    return self.generateHiLoStr(instr, symbol.getSymbolPlusOffset(address))

            elif instructionOffset in self.instrAnalyzer.constantInstrOffset:
                constant = self.instrAnalyzer.constantInstrOffset[instructionOffset]

                symbol = self.getConstant(constant)
                if symbol is not None:
                    return self.generateHiLoStr(instr, symbol.getName())

                # Pretend this pair is a constant
                if instr.canBeHi():
                    loInstr = self.instructions[self.instrAnalyzer.hiToLowDict[instructionOffset] // 4]
                    if loInstr.canBeLo() and loInstr.isUnsigned():
                        return f"(0x{constant:X} >> 16)"
                elif instr.canBeLo() and instr.isUnsigned():
                    return f"(0x{constant:X} & 0xFFFF)"

                if common.GlobalConfig.SYMBOL_FINDER_FILTERED_ADDRESSES_AS_HILO:
                    return self.generateHiLoStr(instr, f"0x{constant:X}")

            elif instr.canBeHi():
                # Unpaired LUI
                return f"(0x{instr.getImmediate()<<16:X} >> 16)"

        elif instr.isJType():
            possibleOverride = self.getSymbol(instr.getInstrIndexAsVram(), tryPlusOffset=False)
            if possibleOverride is not None:
                return possibleOverride.getName()

        return None

    def getLabelForOffset(self, instructionOffset: int) -> str:
        if common.GlobalConfig.IGNORE_BRANCHES or instructionOffset == 0:
            # Skip over this function to avoid duplication
            return ""

        currentVram = self.getVramOffset(instructionOffset)
        labelSym = self.getSymbol(currentVram, tryPlusOffset=False)
        if labelSym is None and len(self.context.offsetJumpTablesLabels) > 0:
            labelSym = self.context.getOffsetGenericLabel(self.inFileOffset+instructionOffset, common.FileSectionType.Text)
        if labelSym is None and len(self.context.offsetSymbols[self.sectionType]) > 0:
            labelSym = self.context.getOffsetSymbol(self.inFileOffset+instructionOffset, common.FileSectionType.Text)

        if labelSym is None or labelSym.overlayCategory != self.overlayCategory:
            return ""

        labelSym.isDefined = True
        labelSym.sectionType = self.sectionType
        if labelSym.type == common.SymbolSpecialType.function or labelSym.type == common.SymbolSpecialType.jumptablelabel:
            label = labelSym.getSymbolLabel() + common.GlobalConfig.LINE_ENDS
            if common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL:
                label += f"{labelSym.getName()}:{common.GlobalConfig.LINE_ENDS}"
            return label
        return labelSym.getName() + ":" + common.GlobalConfig.LINE_ENDS


    def disassemble(self) -> str:
        output = ""

        if not common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS:
            if self.hasUnimplementedIntrs:
                return self.disassembleAsData()

        if self.isLikelyHandwritten:
            output += "# Handwritten function" + common.GlobalConfig.LINE_ENDS

        output += self.getLabel()

        if common.GlobalConfig.ASM_TEXT_ENT_LABEL:
            output += f"{common.GlobalConfig.ASM_TEXT_ENT_LABEL} {self.getName()}" + common.GlobalConfig.LINE_ENDS

        if common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL:
            output += f"{self.getName()}:" + common.GlobalConfig.LINE_ENDS

        wasLastInstABranch = False
        instructionOffset = 0
        for instr in self.instructions:
            immOverride = self.getImmOverrideForInstruction(instr, instructionOffset)
            comment = self.generateAsmLineComment(instructionOffset, instr.getRaw())
            extraLJust = 0

            if wasLastInstABranch:
                extraLJust = -1
                comment += " "

            line = instr.disassemble(immOverride, extraLJust=extraLJust)

            label = self.getLabelForOffset(instructionOffset)
            output += f"{label}{comment}  {line}" + common.GlobalConfig.LINE_ENDS

            wasLastInstABranch = instr.hasDelaySlot()
            instructionOffset += 4

        if common.GlobalConfig.ASM_TEXT_END_LABEL:
            output += f"{common.GlobalConfig.ASM_TEXT_END_LABEL} {self.getName()}" + common.GlobalConfig.LINE_ENDS

        return output

    def disassembleAsData(self) -> str:
        self.words = [instr.getRaw() for instr in self.instructions]
        return super().disassembleAsData()
