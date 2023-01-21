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
        branchOffset = prevInstr.getBranchOffsetGeneric()
        branch = prevInstrOffset + branchOffset

        if branch < 0:
            # Avoid jumping outside of the function
            return

        regsTracker = rabbitizer.RegistersTracker(trackedRegistersOriginal)

        self.instrAnalyzer.processInstr(regsTracker, instr, instructionOffset, currentVram, None)

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

    def _runInstructionAnalyzer(self):
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

    def _processElfRelocSymbols(self):
        instructionOffset = 0
        vrom = self.vromStart
        for instr in self.instructions:
            relocInfo = self.context.globalRelocationOverrides.get(vrom)
            if relocInfo is not None and relocInfo.staticReference is not None:
                hiOffset = self.instrAnalyzer.lowToHiDict.get(instructionOffset)
                if hiOffset is not None:
                    self.instrAnalyzer.symbolInstrOffset[hiOffset] = self.instrAnalyzer.symbolInstrOffset[instructionOffset]

                if relocInfo.staticReference.sectionType == common.FileSectionType.Rodata:
                    symbolVram = relocInfo.staticReference.sectionVram
                    if instr.hasOperandAlias(rabbitizer.OperandType.cpu_immediate):
                        if instructionOffset in self.instrAnalyzer.symbolInstrOffset:
                            addressOffset = self.instrAnalyzer.symbolInstrOffset[instructionOffset]
                            symbolVram += addressOffset
                        else:
                            symbolVram += instr.getProcessedImmediate()
                    contextSym = self.addSymbol(symbolVram, isAutogenerated=True)
                    contextSym._isStatic = True

            vrom += 4
            instructionOffset += 4


    def _postProcessGotAccesses(self):
        if not common.GlobalConfig.PIC:
            return

        globalGotOffsets: set[int] = set()

        for gotAccessOffset, gotAccess in self.instrAnalyzer.gotAccessAddresses.items():
            gotEntry, inGlobalTable = self.context.got.getGotSymEntry(gotAccess)
            if gotEntry is None or inGlobalTable is None:
                continue

            gotAddress = gotEntry.address
            if inGlobalTable:
                globalGotOffsets.add(gotAccessOffset)
                if gotEntry.contextSym is not None:
                    contextSym = gotEntry.contextSym
                else:
                    contextSym = self.addSymbol(gotEntry.address, isAutogenerated=True)
                contextSym.isGot = True
                contextSym.isGotGlobal = True
                loOffset = self.instrAnalyzer.hiToLowDict.get(gotAccessOffset)
                if loOffset is not None:
                    if loOffset in self.instrAnalyzer.symbolLoInstrOffset:
                        del self.instrAnalyzer.symbolLoInstrOffset[loOffset]
                    if loOffset in self.instrAnalyzer.symbolInstrOffset:
                        del self.instrAnalyzer.symbolInstrOffset[loOffset]
            else:
                # GOT-locals need to be paired
                loOffset = self.instrAnalyzer.hiToLowDict.get(gotAccessOffset)
                if loOffset is not None:
                    loInstr = self.instructions[loOffset//4]
                    gotAddress += loInstr.getProcessedImmediate()
                    self.instrAnalyzer.symbolInstrOffset[loOffset] = gotAddress

                    symAccess = self.instrAnalyzer.symbolTypesOffsets.get(loOffset)
                    if symAccess is not None:
                        if gotAddress not in self.instrAnalyzer.possibleSymbolTypes:
                            self.instrAnalyzer.possibleSymbolTypes[gotAddress] = dict()
                        if symAccess not in self.instrAnalyzer.possibleSymbolTypes[gotAddress]:
                            self.instrAnalyzer.possibleSymbolTypes[gotAddress][symAccess] = 0
                        self.instrAnalyzer.possibleSymbolTypes[gotAddress][symAccess] += 1

                contextSym = self.addSymbol(gotAddress, isAutogenerated=True)
                contextSym.isGot = True
                contextSym.isGotLocal = True
            self.instrAnalyzer.symbolInstrOffset[gotAccessOffset] = gotAddress
            self.instrAnalyzer.referencedVrams.add(gotAddress)

        for loOffset, symVram in self.instrAnalyzer.symbolLoInstrOffset.items():
            hiOffset = self.instrAnalyzer.lowToHiDict.get(loOffset)
            if hiOffset is not None and hiOffset in self.instrAnalyzer.gotAccessAddresses:
                if hiOffset in globalGotOffsets:
                    if loOffset in self.instrAnalyzer.symbolInstrOffset:
                        del self.instrAnalyzer.symbolInstrOffset[loOffset]
                    continue

                if hiOffset in self.instrAnalyzer.symbolInstrOffset:
                    symVram = self.instrAnalyzer.symbolInstrOffset[hiOffset]
                    self.instrAnalyzer.symbolLoInstrOffset[loOffset] = symVram

                if loOffset in self.instrAnalyzer.referencedJumpTableOffsets:
                    self.instrAnalyzer.referencedJumpTableOffsets[loOffset] = symVram

                if loOffset in self.instrAnalyzer.indirectFunctionCallOffsets:
                    self.instrAnalyzer.indirectFunctionCallOffsets[loOffset] = symVram
        return

    def _getRelocTypeForInstruction(self, instr: rabbitizer.Instruction, instrOffset: int, contextSym: common.ContextSymbol|None=None, gotHiLo: bool=False) -> common.RelocType:
        if instr.canBeHi():
            if common.GlobalConfig.PIC:
                if contextSym is not None and gotHiLo:
                    if contextSym.isGotGlobal and contextSym.type == common.SymbolSpecialType.function:
                        return common.RelocType.MIPS_CALL_HI16
                    else:
                        return common.RelocType.MIPS_GOT_HI16
            return common.RelocType.MIPS_HI16

        if instr.rs in {rabbitizer.RegGprO32.gp, rabbitizer.RegGprN32.gp}:
            if not common.GlobalConfig.PIC:
                return common.RelocType.MIPS_GPREL16

            if contextSym is not None:
                if contextSym.isGotGlobal and contextSym.type == common.SymbolSpecialType.function and instrOffset in self.instrAnalyzer.indirectFunctionCallOffsets:
                    return common.RelocType.MIPS_CALL16
                elif contextSym.isGot:
                    return common.RelocType.MIPS_GOT16

        elif common.GlobalConfig.PIC:
            if contextSym is not None and gotHiLo:
                if contextSym.isGotGlobal and contextSym.type == common.SymbolSpecialType.function:
                    return common.RelocType.MIPS_CALL_LO16
                else:
                    return common.RelocType.MIPS_GOT_LO16
        return common.RelocType.MIPS_LO16

    def _generateRelocsFromInstructionAnalyzer(self):
        for instrOffset, address in self.instrAnalyzer.symbolInstrOffset.items():
            if self.context.isAddressBanned(address):
                continue

            contextSym = self.getSymbol(address)

            gotAccess = self.instrAnalyzer.gotAccessAddresses.get(instrOffset)
            if gotAccess is not None:
                gotEntry, inGlobalTable = self.context.got.getGotSymEntry(gotAccess)
                if gotEntry is not None:
                    contextSym = gotEntry.contextSym
            if contextSym is None:
                contextSym = self.getSymbol(address, tryPlusOffset=True)

            gotHiLo = False
            if contextSym is None and address < 0 and common.GlobalConfig.PIC and common.GlobalConfig.GP_VALUE is not None:
                # Negative pointer may mean it is a weird GOT access
                gotAccess = common.GlobalConfig.GP_VALUE + address
                gotEntry, inGlobalTable = self.context.got.getGotSymEntry(gotAccess)
                if gotEntry is not None:
                    if gotEntry.contextSym is not None:
                        contextSym = gotEntry.contextSym
                    else:
                        contextSym = self.getSymbol(gotEntry.address)
                    gotHiLo = True
                    address = gotEntry.address

            if contextSym is None:
                continue

            if contextSym.isGotGlobal:
                if instrOffset not in self.instrAnalyzer.gotAccessAddresses and not gotHiLo:
                    continue

            instr = self.instructions[instrOffset//4]

            relocType = self._getRelocTypeForInstruction(instr, instrOffset, contextSym, gotHiLo)
            self.relocs[instrOffset] = common.RelocationInfo(relocType, contextSym, address - contextSym.vram)

        for instrOffset in self.instrAnalyzer.cploadOffsets:
            # .cpload directive is meant to use the `_gp_disp` pseudo-symbol
            instr = self.instructions[instrOffset//4]

            relocType = self._getRelocTypeForInstruction(instr, instrOffset)
            self.relocs[instrOffset] = common.RelocationInfo(relocType, "_gp_disp")

        for instrOffset, constant in self.instrAnalyzer.constantInstrOffset.items():
            instr = self.instructions[instrOffset//4]
            relocType = self._getRelocTypeForInstruction(instr, instrOffset)

            symbol = self.getConstant(constant)
            if symbol is not None:
                self.relocs[instrOffset] = common.RelocationInfo(relocType, symbol.getName())
            elif common.GlobalConfig.SYMBOL_FINDER_FILTERED_ADDRESSES_AS_HILO:
                self.relocs[instrOffset] = common.RelocationInfo(relocType, f"0x{constant:X}")
            else:
                # Pretend this pair is a constant
                loInstr = instr
                if instr.canBeHi():
                    loInstr = self.instructions[self.instrAnalyzer.hiToLowDict[instrOffset] // 4]

                generatedStr = self.generateHiLoConstantStr(constant, instr, loInstr)
                if generatedStr is not None:
                    # return generatedStr
                    self.relocs[instrOffset] = common.RelocationInfo(common.RelocType.MIPS_NONE, generatedStr)


        for instrOffset, targetVram in self.instrAnalyzer.funcCallInstrOffsets.items():
            funcSym = self.getSymbol(targetVram, tryPlusOffset=False)
            if funcSym is None:
                continue
            self.relocs[instrOffset] = common.RelocationInfo(common.RelocType.MIPS_26, funcSym)


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

        self._runInstructionAnalyzer()

        self._postProcessGotAccesses()
        self._processElfRelocSymbols()

        # Branches
        for instrOffset, targetBranchVram in self.instrAnalyzer.branchInstrOffsets.items():
            branch = self.instrAnalyzer.branchTargetInstrOffsets[instrOffset]
            labelSym = self.addBranchLabel(targetBranchVram, isAutogenerated=True, symbolVrom=self.getVromOffset(branch))
            labelSym.referenceCounter += 1
            labelSym.referenceFunctions.add(self.contextSym)

        # Function calls
        for instrOffset, targetVram in self.instrAnalyzer.funcCallInstrOffsets.items():
            funcSym = self.addFunction(targetVram, isAutogenerated=True)
            funcSym.referenceCounter += 1
            funcSym.referenceFunctions.add(self.contextSym)


        if not self.isRsp and len(self.instrAnalyzer.funcCallOutsideRangesOffsets) > 0:
            self.isLikelyHandwritten = True

        # Symbols
        for loOffset, symVram in self.instrAnalyzer.symbolLoInstrOffset.items():
            if self.context.isAddressBanned(symVram):
                continue

            symAccessDict = self.instrAnalyzer.possibleSymbolTypes.get(symVram, dict())
            symAccess = None
            if len(symAccessDict) == 1:
                # Infer type info if there's only one access type
                symAccess = list(symAccessDict)[0]

            contextSym = self.getSymbol(symVram)
            if contextSym is None:
                contextSym = self.addSymbol(symVram, isAutogenerated=True)
            else:
                # TODO: do this in a less ugly way
                if contextSym.address != symVram:
                    if contextSym.address % 4 != 0 or symVram % 4 != 0:
                        if contextSym.getType() in {"u16", "s16", "u8", "u8"} or (symAccess is not None and symAccess.accessType in {rabbitizer.AccessType.BYTE, rabbitizer.AccessType.SHORT}):
                            if not (contextSym.getSize() > 4):
                                if contextSym.size is None or symVram >= contextSym.address + contextSym.size:
                                    if symAccess is not None:
                                        contextSym.setAccessTypeIfUnset(symAccess.accessType, symAccess.unsignedMemoryAccess)
                                    contextSym.setFirstLoAccessIfUnset(loOffset)
                                    contextSym = self.addSymbol(symVram, isAutogenerated=True)

            contextSym.referenceCounter += 1
            contextSym.referenceFunctions.add(self.contextSym)
            if symAccess is not None:
                contextSym.setAccessTypeIfUnset(symAccess.accessType, symAccess.unsignedMemoryAccess)
            contextSym.setFirstLoAccessIfUnset(loOffset)

        self._generateRelocsFromInstructionAnalyzer()

        # Jump tables
        for targetVram in self.instrAnalyzer.referencedJumpTableOffsets.values():
            self.addJumpTable(targetVram, isAutogenerated=True)


        if self.isLikelyHandwritten:
            for instr in self.instructions:
                instr.inHandwrittenFunction = self.isLikelyHandwritten

        self.contextSym.autodetectedSize = self.sizew*4


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
                if instr.isReturn():
                    first_nop += 1
                break
            first_nop = i

        if first_nop < self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated

    def generateHiLoConstantStr(self, constantValue: int, currentInstr: rabbitizer.Instruction, loInstr: rabbitizer.Instruction|None) -> str|None:
        if loInstr is None:
            if currentInstr.canBeHi():
                return f"(0x{constantValue:X} >> 16)"
            return None

        if loInstr.canBeLo():
            if loInstr.isUnsigned():
                if currentInstr.canBeHi():
                    return f"(0x{constantValue:X} >> 16)"
                if currentInstr.canBeLo():
                    return f"(0x{constantValue:X} & 0xFFFF)"
                return None
            else:
                hiHalf = constantValue >> 16
                loHalf = constantValue & 0xFFFF
                if loHalf < 0x8000:
                    # positive lo half
                    if currentInstr.canBeHi():
                        return f"(0x{constantValue:X} >> 16)"
                    if currentInstr.canBeLo():
                        return f"(0x{constantValue:X} & 0xFFFF)"
                else:
                    # negative lo half
                    # loHalf = rabbitizer.Utils.from2Complement(loHalf, 16)
                    pass
        return None


    def getImmOverrideForInstruction(self, instr: rabbitizer.Instruction, instrOffset: int) -> str|None:
        if self.pointersRemoved:
            return None

        relocInfo = self.context.globalRelocationOverrides.get(self.getVromOffset(instrOffset))
        if relocInfo is not None:
            if relocInfo.staticReference is not None:
                relocVram = relocInfo.staticReference.sectionVram
                addend = 0

                if instr.hasOperandAlias(rabbitizer.OperandType.cpu_immediate):
                    if instrOffset in self.instrAnalyzer.symbolInstrOffset:
                        addend = self.instrAnalyzer.symbolInstrOffset[instrOffset]
                    else:
                        addend = instr.getProcessedImmediate()
                labelSym = self.getSymbol(relocVram + addend, checkUpperLimit=False)
                if labelSym is not None:
                    relocInfo.symbol = labelSym
                    if relocVram + addend != labelSym.vram:
                        relocInfo.addend = (relocVram + addend) - labelSym.vram
            return relocInfo.getNameWithReloc()

        if instr.isBranch() or instr.isUnconditionalBranch():
            if common.GlobalConfig.IGNORE_BRANCHES:
                return None
            branchOffset = instr.getBranchOffsetGeneric()
            targetBranchVram = self.getVramOffset(instrOffset + branchOffset)
            labelSymbol = self.getSymbol(targetBranchVram, tryPlusOffset=False)
            if labelSymbol is not None:
                return labelSymbol.getName()
            return None

        relocInfo = self.relocs.get(instrOffset)
        if relocInfo is not None:
            return relocInfo.getNameWithReloc()

        if instr.hasOperandAlias(rabbitizer.OperandType.cpu_immediate):
            if instrOffset in self.instrAnalyzer.symbolInstrOffset:
                address = self.instrAnalyzer.symbolInstrOffset[instrOffset]
                return self.generateHiLoConstantStr(address, instr, instr)

            if instr.canBeHi():
                # Unpaired LUI
                return self.generateHiLoConstantStr(instr.getProcessedImmediate()<<16, instr, None)

        return None

    def getLabelForOffset(self, instructionOffset: int, migrate: bool=False) -> str:
        if common.GlobalConfig.IGNORE_BRANCHES or instructionOffset == 0:
            # Skip over this function to avoid duplication
            return ""

        currentVram = self.getVramOffset(instructionOffset)
        labelSym = self.getSymbol(currentVram, tryPlusOffset=False)

        if labelSym is None or labelSym.overlayCategory != self.overlayCategory:
            return ""

        labelSym.isDefined = True
        labelSym.sectionType = self.sectionType
        if labelSym.type == common.SymbolSpecialType.function or (labelSym.type == common.SymbolSpecialType.jumptablelabel and not migrate):
            label = ""
            labelMacro = labelSym.getLabelMacro()
            if labelMacro is not None:
                label += f"{labelMacro} {labelSym.getName()}{common.GlobalConfig.LINE_ENDS}"
            if common.GlobalConfig.ASM_TEXT_FUNC_AS_LABEL:
                label += f"{labelSym.getName()}:{common.GlobalConfig.LINE_ENDS}"
            return label
        return labelSym.getName() + ":" + common.GlobalConfig.LINE_ENDS

    def _emitInstruction(self, instr: rabbitizer.Instruction, instructionOffset: int, wasLastInstABranch: bool) -> str:
        immOverride = self.getImmOverrideForInstruction(instr, instructionOffset)
        comment = self.generateAsmLineComment(instructionOffset, instr.getRaw())
        extraLJust = 0

        if wasLastInstABranch:
            extraLJust = -1
            comment += " "

        line = instr.disassemble(immOverride, extraLJust=extraLJust)

        return f"{comment}  {line}{common.GlobalConfig.LINE_ENDS}"


    def _emitCpload(self, instr: rabbitizer.Instruction, instructionOffset: int, wasLastInstABranch: bool) -> str:
        output = ""

        cpload = self.instrAnalyzer.cploads.get(instructionOffset)
        if cpload is not None:
            hiInstr = self.instructions[cpload.hiOffset//4]
            loInstr = self.instructions[cpload.loOffset//4]
            gpDisp = hiInstr.getProcessedImmediate() << 16
            gpDisp += loInstr.getProcessedImmediate()
            output += f"# _gp_disp: 0x{gpDisp:X}{common.GlobalConfig.LINE_ENDS}"
            if common.GlobalConfig.EMIT_CPLOAD:
                assert cpload.reg is not None
                output += f".cpload ${cpload.reg.name}" + common.GlobalConfig.LINE_ENDS
            else:
                output += self._emitInstruction(instr, instructionOffset, wasLastInstABranch)
        else:
            if not common.GlobalConfig.EMIT_CPLOAD:
                output += self._emitInstruction(instr, instructionOffset, wasLastInstABranch)
            # don't emit the other instructions which are part of .cpload if the directive was emitted
        return output

    def disassemble(self, migrate: bool=False, useGlobalLabel: bool=True) -> str:
        output = ""

        if migrate:
            output += self.getSpimdisasmVersionString()

        if not common.GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS:
            if self.hasUnimplementedIntrs:
                return self.disassembleAsData(useGlobalLabel=useGlobalLabel)

        output += self.getReferenceeSymbols()

        if self.isLikelyHandwritten:
            if not self.isRsp:
                # RSP functions are always handwritten, so this is redundant
                output += "# Handwritten function" + common.GlobalConfig.LINE_ENDS

        output += self.getSymbolAsmDeclaration(self.getName(), useGlobalLabel)

        wasLastInstABranch = False
        instructionOffset = 0
        for instr in self.instructions:
            label = self.getLabelForOffset(instructionOffset, migrate=migrate)
            output += label

            isCpload = instructionOffset in self.instrAnalyzer.cploadOffsets
            if isCpload:
                output += self._emitCpload(instr, instructionOffset, wasLastInstABranch)
            else:
                output += self._emitInstruction(instr, instructionOffset, wasLastInstABranch)

            wasLastInstABranch = instr.hasDelaySlot()
            instructionOffset += 4

        if common.GlobalConfig.ASM_TEXT_END_LABEL:
            output += f"{common.GlobalConfig.ASM_TEXT_END_LABEL} {self.getName()}" + common.GlobalConfig.LINE_ENDS

        nameEnd = self.getNameEnd()
        if nameEnd is not None:
            output += self.getSymbolAsmDeclaration(nameEnd, useGlobalLabel)

        return output

    def disassembleAsData(self, useGlobalLabel: bool=True) -> str:
        self.words = []
        for i, instr in enumerate(self.instructions):
            if common.GlobalConfig.ASM_COMMENT:
                if not instr.isImplemented() or not instr.isValid():
                    self.endOfLineComment[i] = " # invalid instruction"
            self.words.append(instr.getRaw())
        return super().disassembleAsData(useGlobalLabel=useGlobalLabel)
