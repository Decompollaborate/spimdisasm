#!/usr/bin/env python3

from __future__ import annotations

from ..common.Utils import *
from ..common.GlobalConfig import GlobalConfig
from ..common.Context import Context, ContextSymbol, ContextOffsetSymbol
from ..common.FileSectionType import FileSectionType

from .Instructions import InstructionBase, InstructionId, InstructionsNotEmitedByIDO, InstructionNormal, InstructionCoprocessor0, InstructionCoprocessor2


class Function:
    def __init__(self, name: str, instructions: List[InstructionBase], context: Context, inFileOffset: int, vram: int = -1):
        self.name: str = name
        self.instructions: List[InstructionBase] = list(instructions)
        self.context: Context = context
        self.inFileOffset: int = inFileOffset
        self.commentOffset: int = 0
        self.vram: int = vram
        self.index: int = -1
        self.pointersRemoved: bool = False

        self.localLabels: Dict[int, str] = dict()
        # TODO: this needs a better name
        self.pointersPerInstruction: Dict[int, int] = dict()
        self.constantsPerInstruction: Dict[int, int] = dict()
        self.branchInstructions: List[int] = list()

        # key: %hi (lui) instruction offset, value: %lo instruction offset
        self.hiToLowDict: dict[int, int] = dict()
        # key: %lo instruction offset, value: %hi (lui) instruction offset
        self.lowToHiDict: dict[int, int] = dict()

        self.luiInstructions: dict[int, InstructionBase] = dict()
        self.nonPointerLuiSet: set[int] = set()

        self.pointersOffsets: set[int] = set()
        self.referencedJumpTableOffsets: set[int] = set()

        self.referencedVRams: Set[int] = set()
        self.referencedConstants: Set[int] = set()

        self.hasUnimplementedIntrs: bool = False

        self.parent: Any = None

        self.isRsp: bool = False

        self.isLikelyHandwritten: bool = False

    @property
    def nInstr(self) -> int:
        return len(self.instructions)


    def _printAnalisisDebugInfo_IterInfo(self, instr: InstructionBase, register1: int|None, register2: int|None, register3: int|None, currentVram: int, trackedRegisters: dict, registersValues: dict):
        if not GlobalConfig.PRINT_FUNCTION_ANALYSIS_DEBUG_INFO:
            return

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
        print()

    def _printSymbolFinderDebugInfo_DelTrackedRegister(self, instr: InstructionBase, register: int, currentVram: int, trackedRegisters: dict):
        if not GlobalConfig.PRINT_SYMBOL_FINDER_DEBUG_INFO:
            return

        print()
        print(f"vram: {currentVram:X}")
        print(instr)
        print(trackedRegisters)
        print(f"deleting {register} / {instr.getRegisterName(register)}")
        print()

    def _printSymbolFinderDebugInfo_UnpairedLuis(self):
        if not GlobalConfig.PRINT_UNPAIRED_LUIS_DEBUG_INFO:
            return

        for instructionOffset, luiInstr in self.luiInstructions.items():
            # inFileOffset = self.inFileOffset + instructionOffset
            currentVram = self.vram + instructionOffset
            if instructionOffset in self.nonPointerLuiSet:
                continue
            if instructionOffset in self.constantsPerInstruction:
                print(f"{currentVram:06X} ", end="")
                print(f"C  {self.constantsPerInstruction[instructionOffset]:8X}", luiInstr)
            else:
                if GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and luiInstr.immediate < 0x4000: # filter out stuff that may not be a real symbol
                    continue
                if GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and luiInstr.immediate >= 0xC000: # filter out stuff that may not be a real symbol
                    continue
                print(f"{currentVram:06X} ", end="")
                if instructionOffset in self.pointersPerInstruction:
                    print(f"P  {self.pointersPerInstruction[instructionOffset]:8X}", luiInstr)
                else:
                    print("NO         ", luiInstr)


    def _processSymbol(self, luiInstr: InstructionBase, luiOffset: int, lowerInstr: InstructionBase, lowerOffset: int) -> int|None:
        upperHalf = luiInstr.immediate << 16
        lowerHalf = from2Complement(lowerInstr.immediate, 16)
        address = upperHalf + lowerHalf
        if address in self.context.bannedSymbols:
            return None

        if GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and luiInstr.immediate < 0x4000: # filter out stuff that may not be a real symbol
            return None
        if GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and luiInstr.immediate >= 0xC000: # filter out stuff that may not be a real symbol
            return None

        self.referencedVRams.add(address)
        contextSym = self.context.getGenericSymbol(address)
        if contextSym is None:
            if GlobalConfig.ADD_NEW_SYMBOLS:
                contextSym = self.context.addSymbol(address, None)
                instrType = lowerInstr.mapInstrToType()
                if instrType is not None:
                    contextSym.setTypeIfUnset(instrType)
                contextSym.isAutogenerated = True
                contextSym.referenceCounter = 1
        else:
            contextSym.referenceCounter += 1

        if lowerOffset not in self.pointersPerInstruction:
            self.pointersPerInstruction[lowerOffset] = address
        if luiOffset not in self.pointersPerInstruction:
            self.pointersPerInstruction[luiOffset] = address

        self.hiToLowDict[luiOffset] = lowerOffset
        self.lowToHiDict[lowerOffset] = luiOffset

        return address

    def _processConstant(self, luiInstr: InstructionBase, luiOffset: int, lowerInstr: InstructionBase, lowerOffset: int) -> int|None:
        luiInstr = self.instructions[luiOffset//4]
        upperHalf = luiInstr.immediate << 16
        lowerHalf = lowerInstr.immediate
        constant = upperHalf | lowerHalf

        self.referencedConstants.add(constant)

        self.constantsPerInstruction[lowerOffset] = constant
        self.constantsPerInstruction[luiOffset] = constant

        self.hiToLowDict[luiOffset] = lowerOffset
        self.lowToHiDict[lowerOffset] = luiOffset

        return constant

    def _removeRegisterFromTrackers(self, instr: InstructionBase, currentVram: int, trackedRegisters: dict, trackedRegistersAll: dict, registersValues: dict, wasRegisterValuesUpdated: bool):
        shouldRemove = False
        register = 0

        if not instr.isFloatInstruction():
            if instr.isRType() or (instr.isBranch() and isinstance(instr, InstructionNormal)):
                # $at is a one-use register
                at = -1
                if instr.getRegisterName(instr.rs) == "$at":
                    at = instr.rs
                elif instr.getRegisterName(instr.rt) == "$at":
                    at = instr.rt

                if at in trackedRegistersAll:
                    otherInstrIndex = trackedRegistersAll[at]
                    otherInstr = self.instructions[otherInstrIndex]
                    if otherInstr.uniqueId == InstructionId.LUI:
                        self.nonPointerLuiSet.add(otherInstrIndex*4)
                    shouldRemove = True
                    register = at

            if instr.uniqueId != InstructionId.LUI and instr.modifiesRt():
                shouldRemove = True
                register = instr.rt

            if instr.modifiesRd():
                shouldRemove = True
                register = instr.rd

                # Usually array offsets use an ADDU to add the index of the array
                if instr.uniqueId == InstructionId.ADDU:
                    if instr.rd != instr.rs and instr.rd != instr.rt:
                        shouldRemove = True
                    else:
                        shouldRemove = False

        else:
            if instr.uniqueId in (InstructionId.MTC1, InstructionId.DMTC1, InstructionId.CTC1):
                # IDO usually use a register as a temp when loading a constant value
                # into the float coprocessor, after that IDO never re-uses the value
                # in that register for anything else
                shouldRemove = True
                register = instr.rt

        if shouldRemove:
            if register in trackedRegisters:
                self._printSymbolFinderDebugInfo_DelTrackedRegister(instr, register, currentVram, trackedRegisters)
                del trackedRegisters[register]
            if register in trackedRegistersAll:
                del trackedRegistersAll[register]
            if not wasRegisterValuesUpdated:
                if register in registersValues:
                    del registersValues[register]

    def analyze(self):
        if not GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS and self.hasUnimplementedIntrs:
            if self.vram > -1:
                offset = 0
                for instr in self.instructions:
                    currentVram = self.vram + offset
                    contextSym = self.context.getSymbol(currentVram, False)
                    if contextSym is not None:
                        contextSym.isDefined = True

                    offset += 4
            return

        # Search for LUI instructions first
        instructionOffset = 0
        for instr in self.instructions:
            if instr.uniqueId == InstructionId.LUI:
                self.luiInstructions[instructionOffset] = instr
            if instructionOffset > 0:
                prevInstr = self.instructions[instructionOffset//4 - 1]
                if prevInstr.isJType():
                    self.nonPointerLuiSet.add(instructionOffset)
            instructionOffset += 4

        trackedRegisters: Dict[int, int] = dict()
        trackedRegistersAll: Dict[int, int] = dict()
        # key: register, value: (vram, offset of instruction which set this value)
        registersValues: Dict[int, tuple[int, int]] = dict()

        isABranchInBetweenLastLui: bool|None = None

        instructionOffset = 0
        for instr in self.instructions:
            currentVram = self.vram + instructionOffset
            wasRegisterValuesUpdated = False
            self.isLikelyHandwritten |= instr.uniqueId in InstructionsNotEmitedByIDO

            self._printAnalisisDebugInfo_IterInfo(instr, instr.rs, instr.rt, instr.rd, currentVram, trackedRegisters, registersValues)

            if not self.isLikelyHandwritten:
                if isinstance(instr, InstructionCoprocessor2):
                    self.isLikelyHandwritten = True
                elif isinstance(instr, InstructionCoprocessor0):
                    self.isLikelyHandwritten = True

            if not GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS and not instr.isImplemented():
                # Abort analysis
                self.hasUnimplementedIntrs = True
                return

            if instr.isBranch():
                isABranchInBetweenLastLui = True
                diff = from2Complement(instr.immediate, 16)
                branch = instructionOffset + diff*4 + 1*4
                if self.vram >= 0:
                    self.referencedVRams.add(self.vram + branch)
                    auxLabel = self.context.getGenericLabel(self.vram + branch)
                    if auxLabel is not None:
                        auxLabel.referenceCounter += 1
                        label = auxLabel.name
                    else:
                        label = ".L" + toHex(self.vram + branch, 6)[2:]
                else:
                    label = ".L" + toHex(self.inFileOffset + branch, 6)[2:]

                self.localLabels[self.inFileOffset + branch] = label
                if self.vram >= 0:
                    self.context.addBranchLabel(self.vram + branch, label)
                self.branchInstructions.append(instructionOffset)

            elif instr.isJType():
                target = instr.instr_index << 2
                if not self.isRsp:
                    target |= 0x80000000
                    if target >= 0x84000000:
                        # RSP address space?
                        self.isLikelyHandwritten = True
                if instr.uniqueId == InstructionId.J and not self.isRsp:
                    # self.context.addFakeFunction(target, "fakefunc_" + toHex(target, 8)[2:])
                    self.context.addFakeFunction(target, ".L" + toHex(target, 8)[2:])
                else:
                    self.context.addFunction(None, target, "func_" + toHex(target, 8)[2:])
                self.pointersPerInstruction[instructionOffset] = target

            # symbol finder
            elif instr.isIType():
                # TODO: Consider following branches
                lastInstr = self.instructions[instructionOffset//4 - 1]
                if instr.uniqueId == InstructionId.LUI:
                    isABranchInBetweenLastLui = False
                    if lastInstr.isBranch():
                        # If the previous instructions is a branch, do a
                        # look-ahead and check the branch target for possible pointers
                        diff = from2Complement(lastInstr.immediate, 16)
                        branch = instructionOffset + diff*4
                        if branch > 0:
                            targetInstr = self.instructions[branch//4]
                            if targetInstr.uniqueId == InstructionId.JR and targetInstr.getRegisterName(targetInstr.rs) == "$ra":
                                # If the target instruction is a JR $ra, then look up its delay slot instead
                                branch += 4
                                targetInstr = self.instructions[branch//4]
                            if targetInstr.isIType() and targetInstr.rs == instr.rt:
                                if targetInstr.uniqueId not in (InstructionId.LUI, InstructionId.ANDI, InstructionId.ORI, InstructionId.XORI, InstructionId.CACHE):
                                    self._processSymbol(instr, instructionOffset, targetInstr, branch)

                            if not (lastInstr.isBranchLikely() or lastInstr.uniqueId == InstructionId.B):
                                # If the previous instructions is a branch likely, then nulify
                                # the effects of this instruction for future analysis
                                trackedRegisters[instr.rt] = instructionOffset//4
                    else:
                        trackedRegisters[instr.rt] = instructionOffset//4
                    trackedRegistersAll[instr.rt] = instructionOffset//4
                else:
                    if instr.uniqueId == InstructionId.ORI:
                        # Constants
                        rs = instr.rs
                        if rs in trackedRegistersAll:
                            luiOffset = trackedRegistersAll[rs] * 4
                            luiInstr = self.instructions[luiOffset//4]
                            constant = self._processConstant(luiInstr, luiOffset, instr, instructionOffset)
                            if constant is not None:
                                registersValues[instr.rt] = (constant, instructionOffset)
                    elif instr.uniqueId not in (InstructionId.ANDI, InstructionId.XORI, InstructionId.CACHE):
                        rs = instr.rs
                        if rs in trackedRegisters:
                            luiInstr = self.instructions[trackedRegisters[rs]]
                            address = self._processSymbol(luiInstr, trackedRegisters[rs]*4, instr, instructionOffset)
                            if address is not None:
                                registersValues[instr.rt] = (address, instructionOffset)
                                wasRegisterValuesUpdated = True

                        instrType = instr.mapInstrToType()
                        if instrType is not None:
                            if rs in registersValues:
                                address, _ = registersValues[rs]
                                contextSym = self.context.getSymbol(address, tryPlusOffset=False)
                                if contextSym is not None:
                                    contextSym.setTypeIfUnset(instrType)

            elif instr.uniqueId == InstructionId.JR:
                rs = instr.rs
                if instr.getRegisterName(rs) != "$ra":
                    if rs in registersValues:
                        # print(instructionOffset, rs, trackedRegisters, trackedRegistersAll, registersValues, self.pointersPerInstruction)
                        address, jmptblSeterOffset = registersValues[rs]
                        self.referencedJumpTableOffsets.add(jmptblSeterOffset)
                        self.referencedVRams.add(address)
                        jumpTableSymbol = self.context.addJumpTable(address)
                        jumpTableSymbol.referenceCounter += 1

            self._removeRegisterFromTrackers(instr, currentVram, trackedRegisters, trackedRegistersAll, registersValues, wasRegisterValuesUpdated)

            # look-ahead symbol finder
            lastInstr = self.instructions[instructionOffset//4 - 1]
            if lastInstr.isBranch():
                diff = from2Complement(lastInstr.immediate, 16)
                branch = instructionOffset + diff*4
                if branch > 0 and branch//4 < len(self.instructions):
                    # Check the 5 next instructions in the target branch
                    for i in range(5):
                        if branch//4 >= len(self.instructions):
                            break
                        targetInstr = self.instructions[branch//4]
                        if targetInstr.isBranch():
                            break
                        if targetInstr.isJType():
                            break
                        if targetInstr.modifiesRd():
                            rd = targetInstr.rd
                            # Check if the register is overwritten before finding the low instruction
                            if rd in trackedRegisters:
                                luiInstr = self.instructions[trackedRegisters[rd]]
                                if rd == luiInstr.rt:
                                    break
                        if targetInstr.isIType():
                            if targetInstr.uniqueId not in (InstructionId.LUI, InstructionId.ANDI, InstructionId.ORI, InstructionId.XORI, InstructionId.CACHE):
                                rs = targetInstr.rs
                                if rs in trackedRegisters:
                                    luiInstr = self.instructions[trackedRegisters[rs]]
                                    self._processSymbol(luiInstr, trackedRegisters[rs]*4, targetInstr, branch)
                            break
                        branch += 4

            instructionOffset += 4

        self._printSymbolFinderDebugInfo_UnpairedLuis()

        if len(self.context.relocSymbols[FileSectionType.Text]) > 0:
            # Process reloc symbols (probably from a .elf file)
            instructionOffset = 0
            inFileOffset = self.inFileOffset
            for instr in self.instructions:
                relocSymbol = self.context.getRelocSymbol(inFileOffset, FileSectionType.Text)
                if relocSymbol is not None:
                    if relocSymbol.name.startswith("."):
                        sectType = FileSectionType.fromStr(relocSymbol.name)

                        if instructionOffset in self.pointersPerInstruction:
                            if instructionOffset in self.referencedJumpTableOffsets:
                                # Jump tables
                                addressOffset = self.pointersPerInstruction[instructionOffset]
                                if relocSymbol.name != ".rodata":
                                    eprint(f"Warning. Jumptable referenced in reloc does not have '.rodata' as its name")
                                contextOffsetSym = self.context.addOffsetJumpTable(addressOffset, sectType)
                                contextOffsetSym.referenceCounter += 1
                                relocSymbol.name = contextOffsetSym.name
                                self.pointersPerInstruction[instructionOffset] = 0
                                if instructionOffset in self.lowToHiDict:
                                    luiOffset = self.lowToHiDict[instructionOffset]
                                    otherReloc = self.context.getRelocSymbol(self.inFileOffset+luiOffset, FileSectionType.Text)
                                    if otherReloc is not None:
                                        otherReloc.name = relocSymbol.name
                                        self.pointersPerInstruction[luiOffset] = 0
                            else:
                                addressOffset = self.pointersPerInstruction[instructionOffset]
                                relocName = f"{relocSymbol.name}_{addressOffset:06X}"
                                isStatic = False
                                if relocName.startswith("."):
                                    isStatic = True
                                    relocName = relocName[1:]
                                # print(relocName, addressOffset, instr)
                                contextOffsetSym = ContextOffsetSymbol(addressOffset, relocName, sectType)
                                contextOffsetSym.isStatic = isStatic
                                self.context.offsetSymbols[sectType][addressOffset] = contextOffsetSym
                                relocSymbol.name = relocName
                                self.pointersPerInstruction[instructionOffset] = 0
                inFileOffset += 4
                instructionOffset += 4

    def countDiffOpcodes(self, other: Function) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            if not self.instructions[i].sameOpcode(other.instructions[i]):
                result += 1
        return result

    def countSameOpcodeButDifferentArguments(self, other: Function) -> int:
        result = 0
        for i in range(min(self.nInstr, other.nInstr)):
            instr1 = self.instructions[i]
            instr2 = other.instructions[i]
            if instr1.sameOpcodeButDifferentArguments(instr2):
                result += 1
        return result

    def blankOutDifferences(self, other_func: Function) -> bool:
        if not GlobalConfig.REMOVE_POINTERS:
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
        if not GlobalConfig.REMOVE_POINTERS:
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

        if GlobalConfig.IGNORE_BRANCHES:
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
            if instr.uniqueId != InstructionId.NOP:
                if instr.uniqueId == InstructionId.JR and instr.getRegisterName(instr.rs) == "$ra":
                    first_nop += 1
                break
            first_nop = i

        if first_nop < self.nInstr:
            was_updated = True
            del self.instructions[first_nop:]
        return was_updated

    def disassemble(self) -> str:
        output = ""

        if not GlobalConfig.DISASSEMBLE_UNKNOWN_INSTRUCTIONS:
            if self.hasUnimplementedIntrs:
                return self.disassembleAsData()

        if self.isLikelyHandwritten:
            output += "/* Handwritten function */\n"

        if self.name != "":
            output += f"glabel {self.name}"
            if GlobalConfig.FUNCTION_ASM_COUNT:
                if self.index >= 0:
                    output += f" # {self.index}"
            output += "\n"

        wasLastInstABranch = False

        instructionOffset = 0
        auxOffset = self.inFileOffset
        for instr in self.instructions:
            offsetHex = toHex(auxOffset + self.commentOffset, 6)[2:]
            vramHex = ""
            if self.vram >= 0:
                vramHex = toHex(self.vram + instructionOffset, 8)[2:]
            instrHex = toHex(instr.instr, 8)[2:]

            immOverride = None

            if instr.isBranch():
                if not GlobalConfig.IGNORE_BRANCHES:
                    diff = from2Complement(instr.immediate, 16)
                    branch = instructionOffset + diff*4 + 1*4
                    label = self.context.getGenericLabel(self.vram + branch)
                    if self.vram >= 0 and label is not None:
                        immOverride = label.name
                        label.referenceCounter += 1
                    elif self.inFileOffset + branch in self.localLabels:
                        immOverride = self.localLabels[self.inFileOffset + branch]

            elif instr.isIType():
                if not self.pointersRemoved and instructionOffset in self.pointersPerInstruction:
                    address = self.pointersPerInstruction[instructionOffset]

                    symbol = self.context.getGenericSymbol(address, True)
                    if symbol is not None:
                        symbolName = symbol.getSymbolPlusOffset(address)
                        if instr.uniqueId == InstructionId.LUI:
                            immOverride = f"%hi({symbolName})"
                        else:
                            immOverride= f"%lo({symbolName})"
                elif instructionOffset in self.constantsPerInstruction:
                    constant = self.constantsPerInstruction[instructionOffset]

                    symbol = self.context.getConstant(constant)
                    if symbol is not None:
                        constantName = symbol.name
                        if instr.uniqueId == InstructionId.LUI:
                            immOverride = f"%hi({constantName})"
                        else:
                            immOverride= f"%lo({constantName})"
                    else:
                        if instr.uniqueId == InstructionId.LUI:
                            immOverride = f"(0x{constant:X} >> 16)"
                        else:
                            immOverride = f"(0x{constant:X} & 0xFFFF)"

            elif instr.isJType():
                possibleOverride = self.context.getAnySymbol(instr.getInstrIndexAsVram())
                if immOverride is None and possibleOverride is not None:
                    immOverride = possibleOverride.name

            # Check possible symbols using reloc information (probably from a .o elf file)
            possibleImmOverride = self.context.getRelocSymbol(auxOffset, FileSectionType.Text)
            if possibleImmOverride is not None:
                auxOverride = possibleImmOverride.name
                if instr.isIType():
                    if instructionOffset in self.pointersPerInstruction:
                        addressOffset = self.pointersPerInstruction[instructionOffset]
                        auxOverride = possibleImmOverride.getNamePlusOffset(addressOffset)
                    if instr.uniqueId == InstructionId.LUI:
                        auxOverride = f"%hi({auxOverride})"
                    else:
                        auxOverride= f"%lo({auxOverride})"
                immOverride = auxOverride

            if wasLastInstABranch:
                instr.ljustWidthOpcode -= 1

            line = instr.disassemble(immOverride)

            if wasLastInstABranch:
                instr.ljustWidthOpcode += 1

            #comment = " "
            comment = ""
            if GlobalConfig.ASM_COMMENT:
                comment = f"/* {offsetHex} {vramHex} {instrHex} */  "
            if wasLastInstABranch:
                comment += " "
            line = comment + line

            label = ""
            if not GlobalConfig.IGNORE_BRANCHES:
                currentVram = self.vram + instructionOffset
                labelAux = self.context.getGenericLabel(currentVram)
                label_offsetBranch = self.context.getOffsetGenericLabel(auxOffset, FileSectionType.Text)
                label_offsetSymbol = self.context.getOffsetSymbol(auxOffset, FileSectionType.Text)
                if self.vram >= 0 and labelAux is not None:
                    if instructionOffset == 0:
                        # Skip over this function to avoid duplication
                        pass
                    elif currentVram in self.context.jumpTablesLabels:
                        label = "glabel " + labelAux.name + "\n"
                    else:
                        label = labelAux.name + ":\n"
                elif label_offsetBranch is not None:
                    if instructionOffset == 0:
                        # Skip over this function to avoid duplication
                        pass
                    elif auxOffset in self.context.offsetJumpTablesLabels:
                        label = "glabel " + label_offsetBranch.name + "\n"
                    else:
                        label = label_offsetBranch.name + ":\n"
                elif auxOffset in self.localLabels:
                    label = self.localLabels[auxOffset] + ":\n"
                elif currentVram in self.context.fakeFunctions:
                    label = self.context.fakeFunctions[currentVram].name + ":\n"
                elif label_offsetSymbol is not None:
                    if instructionOffset == 0:
                        # Skip over this function to avoid duplication
                        pass
                    else:
                        label = f"{label_offsetSymbol.name}:\n"

            output += label + line + "\n"

            wasLastInstABranch = instr.isBranch() or instr.isJType() or instr.uniqueId in (InstructionId.JR, InstructionId.JALR)

            instructionOffset += 4
            auxOffset += 4

        return output

    def disassembleAsData(self) -> str:
        output = ""

        instructionOffset = 0
        auxOffset = self.inFileOffset
        for instr in self.instructions:
            offsetHex = toHex(auxOffset + self.commentOffset, 6)[2:]
            vramHex = ""
            label = ""
            if self.vram >= 0:
                vramHex = toHex(self.vram + instructionOffset, 8)[2:]
                auxLabel = self.context.getGenericLabel(self.vram + instructionOffset) or self.context.getGenericSymbol(self.vram + instructionOffset, tryPlusOffset=False)
                if auxLabel is not None:
                    label = f"\nglabel {auxLabel.name}\n"
                    # TODO: required?
                    auxLabel.referenceCounter += 1

                contextVar = self.context.getSymbol(self.vram + instructionOffset, False)
                if contextVar is not None:
                    contextVar.isDefined = True

            instrHex = toHex(instr.instr, 8)[2:]

            line = f".word  0x{instrHex}"

            #comment = " "
            comment = ""
            if GlobalConfig.ASM_COMMENT:
                comment = f"/* {offsetHex} {vramHex} {instrHex} */  "
            line = comment + line

            output += label + line + "\n"

            instructionOffset += 4
            auxOffset += 4

        return output
