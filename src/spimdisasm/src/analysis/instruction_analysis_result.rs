/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{
    access_type::AccessType, opcodes::Opcode, registers::Gpr, registers_meta::Register,
    vram::VramOffset, Instruction, Vram,
};

use crate::{
    addresses::{Rom, RomVramRange},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    context::Context,
};

use super::RegisterTracker;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionAnalysisResult {
    ranges: RomVramRange,

    /// Every referenced vram found.
    referenced_vrams: UnorderedSet<Vram>,
    /// Key is the rom of the instruction referencing that address, value is the referenced address.
    referenced_vrams_by_rom: UnorderedMap<Rom, Vram>,

    /// Key is the rom of the branch instruction, value is the vram target for that instruction.
    branch_targets: UnorderedMap<Rom, Vram>,
    /// Same as `branch_targets`, but for branching outside the current function.
    branch_targets_outside: UnorderedMap<Rom, Vram>,

    /// Key is the rom of the instruction, value is the address of the called function.
    func_calls: UnorderedMap<Rom, Vram>,

    referenced_jumptables: UnorderedMap<Rom, Vram>,

    hi_instrs: UnorderedMap<Rom, (Gpr, u16)>,
    non_lo_instrs: UnorderedSet<Rom>,

    constant_per_instr: UnorderedMap<Rom, u32>,

    // TODO: merge these 3 thingies
    address_per_instr: UnorderedMap<Rom, Vram>,
    address_per_hi_instr: UnorderedMap<Rom, Vram>,
    address_per_lo_instr: UnorderedMap<Rom, Vram>,

    type_info_per_address: UnorderedMap<Vram, UnorderedMap<(AccessType, bool), u32>>,
    type_info_per_instr: UnorderedMap<Rom, (AccessType, bool)>,

    handwritten_instrs: UnorderedSet<Rom>,
}

impl InstructionAnalysisResult {
    #[must_use]
    pub(crate) fn new(ranges: RomVramRange) -> Self {
        // TODO: require how many instructions this function has, so we can use `with_capacity`
        Self {
            ranges,
            referenced_vrams: UnorderedSet::new(),
            referenced_vrams_by_rom: UnorderedMap::new(),
            branch_targets: UnorderedMap::new(),
            branch_targets_outside: UnorderedMap::new(),
            func_calls: UnorderedMap::new(),
            referenced_jumptables: UnorderedMap::new(),
            hi_instrs: UnorderedMap::new(),
            non_lo_instrs: UnorderedSet::new(),
            constant_per_instr: UnorderedMap::new(),
            address_per_instr: UnorderedMap::new(),
            address_per_hi_instr: UnorderedMap::new(),
            address_per_lo_instr: UnorderedMap::new(),
            type_info_per_address: UnorderedMap::new(),
            type_info_per_instr: UnorderedMap::new(),
            handwritten_instrs: UnorderedSet::new(),
        }
    }

    #[must_use]
    pub fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }

    #[must_use]
    pub fn referenced_vrams(&self) -> &UnorderedSet<Vram> {
        &self.referenced_vrams
    }

    #[must_use]
    pub fn branch_targets(&self) -> &UnorderedMap<Rom, Vram> {
        &self.branch_targets
    }
    #[must_use]
    pub fn branch_targets_outside(&self) -> &UnorderedMap<Rom, Vram> {
        &self.branch_targets_outside
    }

    #[must_use]
    pub fn func_calls(&self) -> &UnorderedMap<Rom, Vram> {
        &self.func_calls
    }

    #[must_use]
    pub fn hi_instrs(&self) -> &UnorderedMap<Rom, (Gpr, u16)> {
        &self.hi_instrs
    }

    #[must_use]
    pub fn constant_per_instr(&self) -> &UnorderedMap<Rom, u32> {
        &self.constant_per_instr
    }

    #[must_use]
    pub fn address_per_hi_instr(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_hi_instr
    }
    #[must_use]
    pub fn address_per_lo_instr(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_lo_instr
    }

    #[must_use]
    pub fn referenced_jumptables(&self) -> &UnorderedMap<Rom, Vram> {
        &self.referenced_jumptables
    }

    #[must_use]
    pub fn type_info_per_address(
        &self,
    ) -> &UnorderedMap<Vram, UnorderedMap<(AccessType, bool), u32>> {
        &self.type_info_per_address
    }

    #[must_use]
    pub fn handwritten_instrs(&self) -> &UnorderedSet<Rom> {
        &self.handwritten_instrs
    }
}

impl InstructionAnalysisResult {
    pub(crate) fn process_prev_func_call(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        prev_instr: &Instruction,
    ) {
        regs_tracker.unset_registers_after_func_call(prev_instr);
    }

    pub(crate) fn process_instr(
        &mut self,
        context: &Context,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        prev_instr: Option<&Instruction>,
    ) {
        let instr_rom = self.rom_from_instr(instr);

        if instr.is_likely_handwritten() {
            self.handwritten_instrs.insert(instr_rom);
        }

        if let Some(target_vram) = instr.get_branch_vram_generic() {
            // instr.opcode().is_branch() or instr.is_unconditional_branch()
            self.process_branch(context, regs_tracker, instr, instr_rom, target_vram);
        } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
            // instr.opcode().is_jump_with_address()
            self.process_func_call(context, instr, instr_rom, target_vram);
        } else if instr.is_jumptable_jump() {
            self.process_jumptable_jump(context, regs_tracker, instr, instr_rom);
        } else if instr.opcode().is_jump() && instr.opcode().does_link() {
            // `jalr`. Implicit `!is_jump_with_address`
            self.process_jump_and_link_register(regs_tracker, instr, instr_rom);
        } else if instr.opcode().can_be_hi() {
            self.process_hi(regs_tracker, instr, instr_rom, prev_instr);
        } else if instr.opcode().is_unsigned() {
            self.process_unsigned_lo(regs_tracker, instr, instr_rom);
        } else if instr.opcode().can_be_lo() {
            self.process_signed_lo(context, regs_tracker, instr, instr_rom, prev_instr);
            self.process_symbol_dereference_type(regs_tracker, instr, instr_rom);
        } else if instr.opcode() == Opcode::core_addu {
            /*
            # special check for .cpload
            if len(self.unpairedCploads) > 0:
                if instr.rd in {rabbitizer.RegGprO32.gp, rabbitizer.RegGprN32.gp} and instr.rs in {rabbitizer.RegGprO32.gp, rabbitizer.RegGprN32.gp}:
                    cpload = self.unpairedCploads.pop()
                    cpload.adduOffset = instrOffset
                    cpload.reg = instr.rt
                    self.cploadOffsets.add(cpload.hiOffset)
                    self.cploadOffsets.add(cpload.loOffset)
                    self.cploadOffsets.add(instrOffset)
                    self.cploads[instrOffset] = cpload
            */
        }

        regs_tracker.overwrite_registers(instr, instr_rom);
    }
}

impl InstructionAnalysisResult {
    fn process_branch(
        &mut self,
        context: &Context,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
        target_vram: Vram,
    ) {
        regs_tracker.process_branch(instr, instr_rom);

        /*
        if instrOffset in self.branchInstrOffsets:
            # Already processed
            return
        */

        self.add_referenced_vram(context, instr_rom, target_vram);

        if self.ranges.in_vram_range(target_vram) {
            self.branch_targets.insert(instr_rom, target_vram);
        } else {
            self.branch_targets_outside.insert(instr_rom, target_vram);
        }
    }

    fn process_func_call(
        &mut self,
        context: &Context,
        _instr: &Instruction,
        instr_rom: Rom,
        target_vram: Vram,
    ) {
        /*
        if instrOffset in self.funcCallInstrOffsets:
            # Already processed
            return
        */

        /*
        if not self.context.isAddressInGlobalRange(target):
            self.funcCallOutsideRangesOffsets[instrOffset] = target
        */

        self.add_referenced_vram(context, instr_rom, target_vram);
        self.func_calls.insert(instr_rom, target_vram);
    }

    fn process_jumptable_jump(
        &mut self,
        context: &Context,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
    ) {
        if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(instr) {
            let lo_rom = jr_reg_data.lo_rom();
            let address = Vram::new(jr_reg_data.address());

            if jr_reg_data.branch_info().is_some() {
                // Jumptables never check the register they are branching into,
                // since the references should always be valid.
                // This kind of check usually is performed on tail call
                // optimizations when a function pointer is involved.
                // For example:
                // ```mips
                // lw          $t0, ...
                // beqz        $t0, .LXXXXXXXX
                //  nop
                // jr          $t0
                //  nop
                // ```
                // TODO
                // self.rejectedjumpRegisterIntrOffset[instrOffset] = (offset, address, jrRegData.lastBranchOffset())
            } else {
                self.referenced_jumptables.insert(lo_rom, address);
            }

            // self.jumpRegisterIntrOffset[instrOffset] = address
            self.add_referenced_vram(context, instr_rom, address);
        }
    }

    fn process_jump_and_link_register(
        &mut self,
        _regs_tracker: &mut RegisterTracker,
        _instr: &Instruction,
        _instr_rom: Rom,
    ) {
        // TODO
        /*
        jrRegData = regsTracker.getJrRegData(instr)
        if jrRegData.hasInfo():
            offset = jrRegData.offset()
            address = jrRegData.address()

            self.indirectFunctionCallOffsets[offset] = address
            self.indirectFunctionCallIntrOffset[instrOffset] = address
            if not common.GlobalConfig.PIC:
                self.referencedVrams.add(address)
        */
    }

    fn process_hi(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
        prev_instr: Option<&Instruction>,
    ) {
        regs_tracker.process_hi(instr, instr_rom, prev_instr);
        self.hi_instrs.insert(
            instr_rom,
            (
                instr.get_destination_gpr().unwrap(),
                instr.get_processed_immediate().unwrap() as u16,
            ),
        );
    }

    fn process_unsigned_lo(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
    ) {
        // Pairing with an `ori`, so we treat this as a constant.
        if let Some(hi_info) = regs_tracker.get_hi_info_for_constant(instr) {
            if let Some((_hi_reg, hi_imm)) = self.hi_instrs.get(&hi_info.instr_rom) {
                self.process_constant(regs_tracker, instr, instr_rom, *hi_imm, hi_info.instr_rom)
            }
        }
    }

    fn process_constant(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
        hi_imm: u16,
        hi_rom: Rom,
    ) {
        let upper = hi_imm as u32;
        let lower = instr.get_processed_immediate().unwrap() as u32; // TODO: avoid unwrap
        let constant = (upper << 16) | lower;

        // self.referencedConstants.add(constant)

        // self.constantHiInstrOffset[luiOffset] = constant
        // self.constantLoInstrOffset[lowerOffset] = constant
        self.constant_per_instr.insert(hi_rom, constant);
        self.constant_per_instr.insert(instr_rom, constant);

        // self.hiToLowDict[luiOffset] = lowerOffset
        // self.lowToHiDict[lowerOffset] = luiOffset

        regs_tracker.process_constant(instr, constant, instr_rom);
    }

    fn process_signed_lo(
        &mut self,
        context: &Context,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
        _prev_instr: Option<&Instruction>,
    ) {
        // TODO
        if instr.opcode().does_load()
            && instr
                .get_destination_gpr()
                .is_some_and(|reg| reg.is_global_pointer(instr.abi()))
        {
            regs_tracker.process_gp_load(instr, instr_rom);
        }

        /*
        if instrOffset in self.nonLoInstrOffsets:
            return
        */

        let pairing_info = regs_tracker.preprocess_lo_and_get_info(instr, instr_rom);
        if pairing_info.is_none() {
            if regs_tracker.has_lo_but_not_hi(instr) {
                self.non_lo_instrs.insert(instr_rom);
            }
            return;
        }
        let pairing_info = pairing_info.unwrap();

        if pairing_info.is_gp_got && !context.global_config().gp_config().is_some_and(|x| x.pic()) {
            return;
        }

        let upper_info = if pairing_info.is_gp_rel {
            None
        } else {
            Some((pairing_info.value, pairing_info.instr_rom))
        };

        if let Some((_upper_half, hi_rom)) = upper_info {
            if let Some((hi_reg, _hi_imm)) = self.hi_instrs.get(&hi_rom) {
                if hi_reg.is_global_pointer(instr.abi()) {
                    if let Some(lo_rs) = instr.field_rs() {
                        if instr.opcode().reads_rs() && lo_rs.is_global_pointer(instr.abi()) {
                            if let Some(lo_rt) = instr.field_rt() {
                                if instr.opcode().modifies_rt()
                                    && lo_rt.is_global_pointer(instr.abi())
                                {
                                    if context.global_config().gp_config().is_some_and(|x| x.pic())
                                    {
                                        /*
                                        # cpload
                                        self.unpairedCploads.append(CploadInfo(luiOffset, instrOffset))
                                        */
                                    } else {
                                        /*
                                        hiGpValue = luiInstr.getProcessedImmediate() << 16
                                        loGpValue = instr.getProcessedImmediate()
                                        self.gpSets[instrOffset] = GpSetInfo(luiOffset, instrOffset, hiGpValue+loGpValue)
                                        self.gpSetsOffsets.add(luiOffset)
                                        self.gpSetsOffsets.add(instrOffset)
                                        */
                                    }
                                    // Early return to avoid counting this pairing as a normal symbol
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }

        let address = self.pair_hi_lo(context, upper_info.as_ref(), instr, instr_rom);
        if address.is_none() {
            return;
        }
        let address = address.unwrap();
        if upper_info.is_none() && context.global_config().gp_config().is_some_and(|x| x.pic()) {
            self.process_got_symbol(address, instr_rom);
            return;
        }

        if self.process_address(context, address, upper_info.as_ref(), instr, instr_rom) {
            // TODO: move out from this check
            regs_tracker.process_lo(instr, address.inner(), instr_rom);
        }
    }

    fn process_symbol_dereference_type(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
    ) {
        if let Some(address) = regs_tracker.get_address_if_instr_can_set_type(instr, instr_rom) {
            self.process_symbol_type(Vram::new(address), instr, instr_rom);
        }
    }
}

impl InstructionAnalysisResult {
    fn pair_hi_lo(
        &mut self,
        context: &Context,
        upper_info: Option<&(i64, Rom)>,
        instr: &Instruction,
        _instr_rom: Rom,
    ) -> Option<Vram> {
        // upper_info being None means this symbol is a $gp access

        let lower_half = if let Some(x) = instr.get_processed_immediate() {
            VramOffset::new(x)
        } else {
            return None;
        };
        /*

        if lowerOffset in self.symbolLoInstrOffset:
            # This %lo has been processed already

            # Check the other lui has the same immediate value as this one, and reject the pair if it doesn't
            if hiValue is not None:
                otherLuiOffset = self.lowToHiDict.get(lowerOffset, None)
                if otherLuiOffset is not None:
                    otherLuiInstr = self.luiInstrs.get(otherLuiOffset, None)
                    if otherLuiInstr is not None:
                        if hiValue != otherLuiInstr.getProcessedImmediate() << 16:
                            return None

            if not common.GlobalConfig.COMPILER.value.pairMultipleHiToSameLow:
                # IDO does not pair multiples %hi to the same %lo
                return self.symbolLoInstrOffset[lowerOffset]

            else:
                if luiOffset is None or hiValue is None:
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
                    upperHalf = hiValue
                    address = upperHalf + lowerHalf
                    if address == self.symbolLoInstrOffset[lowerOffset]:
                        # Make an exception if the resulting address is the same
                        pass
                    else:
                        return self.symbolLoInstrOffset[lowerOffset]
        */

        if let Some((upper_half, _hi_rom)) = upper_info {
            if *upper_half < 0
                || (lower_half.is_negative()
                    && lower_half.inner().unsigned_abs() > *upper_half as u32)
            {
                None
            } else {
                Some(Vram::new(*upper_half as u32) + lower_half)
            }
        } else if let Some(gp_value) = context.global_config().gp_config().map(|x| x.gp_value()) {
            // TODO: implement comparison for Vram and VramOffset
            if lower_half.is_negative() && lower_half.inner().unsigned_abs() > gp_value.inner() {
                None
            } else {
                Some(gp_value + lower_half)
            }
        } else {
            None
        }
    }

    fn process_got_symbol(&mut self, _address: Vram, _instr_rom: Rom) {
        // TODO
    }

    fn process_address(
        &mut self,
        context: &Context,
        address: Vram,
        upper_info: Option<&(i64, Rom)>,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> bool {
        /*
        # filter out stuff that may not be a real symbol
        filterOut = False
        if not self.context.totalVramRange.isInRange(address):
            if common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES or common.GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES:
                filterOut |= common.GlobalConfig.SYMBOL_FINDER_FILTER_LOW_ADDRESSES and address < common.GlobalConfig.SYMBOL_FINDER_FILTER_ADDRESSES_ADDR_LOW
                filterOut |= common.GlobalConfig.SYMBOL_FINDER_FILTER_HIGH_ADDRESSES and address >= common.GlobalConfig.SYMBOL_FINDER_FILTER_ADDRESSES_ADDR_HIGH
            else:
                filterOut |= True

        if filterOut:
            contextSym = self.context.globalSegment.getSymbol(address)
            if contextSym is not None:
                if contextSym.isUserDeclared:
                    # If the user declared a symbol outside the total vram range then use it anyways
                    filterOut = False

        if address > 0 and filterOut and lowerInstr.uniqueId != rabbitizer.InstrId.cpu_addiu:
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
        */

        self.add_referenced_vram(context, instr_rom, address);

        if self
            .address_per_lo_instr
            .insert(instr_rom, address)
            .is_none()
        {
            self.address_per_instr.insert(instr_rom, address);
        }
        if let Some((_upper_half, hi_rom)) = upper_info {
            if self.address_per_hi_instr.insert(*hi_rom, address).is_none() {
                self.address_per_instr.insert(*hi_rom, address);
                self.add_referenced_vram(context, *hi_rom, address);
            }
            /*
            self.hiToLowDict[luiOffset] = lowerOffset
            self.lowToHiDict[lowerOffset] = luiOffset
            */
        } else {
            /*
            self.symbolGpInstrOffset[lowerOffset] = address
            self.gpReferencedSymbols.add(address)
            self.symbolInstrOffset[lowerOffset] = address
            */
        }

        self.process_symbol_type(address, instr, instr_rom);

        true
    }

    fn process_symbol_type(&mut self, address: Vram, instr: &Instruction, instr_rom: Rom) {
        if let Some(access_type) = instr.opcode().access_type() {
            let unsigned_memory_address = instr.opcode().does_unsigned_memory_access();

            self.type_info_per_address
                .entry(address)
                .or_default()
                .entry((access_type, unsigned_memory_address))
                .and_modify(|v| *v += 1)
                .or_insert(1);
            self.type_info_per_instr
                .insert(instr_rom, (access_type, unsigned_memory_address));
        }
    }
}

impl InstructionAnalysisResult {
    fn rom_from_instr(&self, instr: &Instruction) -> Rom {
        self.ranges
            .rom_from_vram(instr.vram())
            .expect("This should not panic")
    }

    fn add_referenced_vram(&mut self, context: &Context, instr_rom: Rom, referenced_vram: Vram) {
        if !context.global_config().gp_config().is_some_and(|x| x.pic()) {
            self.referenced_vrams.insert(referenced_vram);
            self.referenced_vrams_by_rom
                .insert(instr_rom, referenced_vram);
        }
    }
}
