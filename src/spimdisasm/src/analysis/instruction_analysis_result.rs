/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{
    access_type::AccessType, opcodes::Opcode, registers::Gpr, registers_meta::Register,
    vram::VramOffset, Instruction, Vram,
};

use crate::{
    addresses::{GpValue, Rom, RomVramRange},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    config::{GlobalConfig, GpConfig},
};

use super::RegisterTracker;

/// Info for tracking when a $gp register is set to an explicit address
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GpAddressSet {
    hi_rom: Rom,
    lo_rom: Rom,
    value: GpValue,
}
impl GpAddressSet {
    fn new(hi_rom: Rom, lo_rom: Rom, value: GpValue) -> Self {
        Self {
            hi_rom,
            lo_rom,
            value,
        }
    }

    pub(crate) fn hi_rom(&self) -> Rom {
        self.hi_rom
    }
    pub(crate) fn lo_rom(&self) -> Rom {
        self.lo_rom
    }
    pub(crate) fn value(&self) -> GpValue {
        self.value
    }
}

// Tracking when the $gp register is set to a value
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GpSetInfo {
    Address(GpAddressSet),
    Unknown,
}
impl GpSetInfo {
    fn new_address(hi_rom: Rom, lo_rom: Rom, value: GpValue) -> Self {
        GpSetInfo::Address(GpAddressSet::new(hi_rom, lo_rom, value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionAnalysisResult {
    ranges: RomVramRange,
    original_gp_config: Option<GpConfig>,
    current_gp_value: Option<GpValue>,

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
    non_linking_func_calls: UnorderedMap<Rom, Vram>,

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

    /// Instructions setting the $gp register, key: offset of the low instruction
    gp_sets: UnorderedMap<Rom, GpSetInfo>,

    got_access_addresses: UnorderedMap<Rom, Vram>,
    calculated_got_addresses: UnorderedMap<Rom, Vram>,

    hi_to_lo: UnorderedMap<Rom, Rom>,
    lo_to_hi: UnorderedMap<Rom, Rom>,

    // Jump and link (functions)
    indirect_function_call_instr: UnorderedMap<Rom, Vram>,
    indirect_function_call: UnorderedMap<Rom, Vram>,

    lo_rom_added_with_gp: UnorderedSet<Rom>,
}

impl InstructionAnalysisResult {
    #[must_use]
    pub(crate) fn new(ranges: RomVramRange, global_config: &GlobalConfig) -> Self {
        // TODO: require how many instructions this function has, so we can use `with_capacity`

        let gp_config = global_config.gp_config();

        Self {
            ranges,
            original_gp_config: gp_config.copied(),
            current_gp_value: gp_config.map(|x| x.gp_value()),
            referenced_vrams: UnorderedSet::new(),
            referenced_vrams_by_rom: UnorderedMap::new(),
            branch_targets: UnorderedMap::new(),
            branch_targets_outside: UnorderedMap::new(),
            func_calls: UnorderedMap::new(),
            non_linking_func_calls: UnorderedMap::new(),
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
            gp_sets: UnorderedMap::new(),
            got_access_addresses: UnorderedMap::new(),
            calculated_got_addresses: UnorderedMap::new(),
            hi_to_lo: UnorderedMap::new(),
            lo_to_hi: UnorderedMap::new(),
            indirect_function_call_instr: UnorderedMap::new(),
            indirect_function_call: UnorderedMap::new(),
            lo_rom_added_with_gp: UnorderedSet::new(),
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
    pub fn non_linking_func_calls(&self) -> &UnorderedMap<Rom, Vram> {
        &self.non_linking_func_calls
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
    pub fn address_per_instr(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_instr
    }
    #[must_use]
    pub fn address_per_instr_mut(&mut self) -> &mut UnorderedMap<Rom, Vram> {
        &mut self.address_per_instr
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
    pub fn address_per_lo_instr_mut(&mut self) -> &mut UnorderedMap<Rom, Vram> {
        &mut self.address_per_lo_instr
    }

    #[must_use]
    pub fn referenced_jumptables(&self) -> &UnorderedMap<Rom, Vram> {
        &self.referenced_jumptables
    }
    #[must_use]
    pub fn referenced_jumptables_mut(&mut self) -> &mut UnorderedMap<Rom, Vram> {
        &mut self.referenced_jumptables
    }

    #[must_use]
    pub fn type_info_per_address(
        &self,
    ) -> &UnorderedMap<Vram, UnorderedMap<(AccessType, bool), u32>> {
        &self.type_info_per_address
    }
    #[must_use]
    pub fn type_info_per_instr(&self) -> &UnorderedMap<Rom, (AccessType, bool)> {
        &self.type_info_per_instr
    }

    #[must_use]
    pub fn handwritten_instrs(&self) -> &UnorderedSet<Rom> {
        &self.handwritten_instrs
    }

    #[must_use]
    pub(crate) fn gp_sets(&self) -> &UnorderedMap<Rom, GpSetInfo> {
        &self.gp_sets
    }

    #[must_use]
    pub(crate) fn got_access_addresses(&self) -> &UnorderedMap<Rom, Vram> {
        &self.got_access_addresses
    }

    #[must_use]
    pub(crate) fn calculated_got_addresses(&self) -> &UnorderedMap<Rom, Vram> {
        &self.calculated_got_addresses
    }
    #[must_use]
    pub(crate) fn calculated_got_addresses_mut(&mut self) -> &mut UnorderedMap<Rom, Vram> {
        &mut self.calculated_got_addresses
    }

    #[must_use]
    pub(crate) fn hi_to_lo(&self) -> &UnorderedMap<Rom, Rom> {
        &self.hi_to_lo
    }
    #[must_use]
    pub(crate) fn lo_to_hi(&self) -> &UnorderedMap<Rom, Rom> {
        &self.lo_to_hi
    }

    #[must_use]
    pub(crate) fn indirect_function_call(&self) -> &UnorderedMap<Rom, Vram> {
        &self.indirect_function_call
    }
    #[must_use]
    pub(crate) fn indirect_function_call_mut(&mut self) -> &mut UnorderedMap<Rom, Vram> {
        &mut self.indirect_function_call
    }

    #[must_use]
    pub(crate) fn lo_rom_added_with_gp(&self) -> &UnorderedSet<Rom> {
        &self.lo_rom_added_with_gp
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
            self.process_branch(regs_tracker, instr, instr_rom, target_vram);
        } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
            // instr.opcode().is_jump_with_address()
            self.process_func_call(instr, instr_rom, target_vram);
        } else if instr.is_jumptable_jump() {
            self.process_jumptable_jump(regs_tracker, instr, instr_rom);
        } else if instr.opcode().is_jump() && instr.opcode().does_link() {
            // `jalr`. Implicit `!is_jump_with_address`
            self.process_jump_and_link_register(regs_tracker, instr, instr_rom);
        } else if instr.opcode().can_be_hi() {
            self.process_hi(regs_tracker, instr, instr_rom);
        } else if instr.opcode().can_be_lo() {
            self.process_signed_lo(regs_tracker, instr, instr_rom, prev_instr);
            self.process_symbol_dereference_type(regs_tracker, instr, instr_rom);
        } else if instr.opcode().can_be_unsigned_lo() {
            self.process_unsigned_lo(regs_tracker, instr, instr_rom);
        } else if instr.opcode() == Opcode::core_addu {
            let rd = instr.field_rd();
            let rs = instr.field_rs();
            let rt = instr.field_rt();

            if let (Some(rd), Some(rs), Some(rt)) = (rd, rs, rt) {
                let rs_is_gp = rs.is_global_pointer(instr.abi());
                let rt_is_gp = rt.is_global_pointer(instr.abi());

                if rd.is_global_pointer(instr.abi()) {
                    // special check for .cpload
                    /*
                    if len(self.unpairedCploads) > 0:
                        if instr.rs in {rabbitizer.RegGprO32.gp, rabbitizer.RegGprN32.gp}:
                            cpload = self.unpairedCploads.pop()
                            cpload.adduOffset = instrOffset
                            cpload.reg = instr.rt
                            self.cploadOffsets.add(cpload.hiOffset)
                            self.cploadOffsets.add(cpload.loOffset)
                            self.cploadOffsets.add(instrOffset)
                            self.cploads[instrOffset] = cpload
                    */
                } else if rs_is_gp ^ rt_is_gp {
                    let reg = if !rs_is_gp { rs } else { rt };

                    if reg == rd {
                        // We have something like
                        // addu        $t7, $t7, $gp

                        regs_tracker.set_added_with_gp(reg, instr_rom);
                    }
                }
            }
        }

        regs_tracker.overwrite_registers(instr, instr_rom);
        if let Some(reg) = instr.get_destination_gpr() {
            if reg.is_global_pointer(instr.abi()) {
                let gp_state = regs_tracker.get_gp_state();
                let info = if let (Some(hi_info), Some(lo_info)) =
                    (gp_state.hi_info(), gp_state.lo_info())
                {
                    GpSetInfo::new_address(
                        hi_info.instr_rom,
                        lo_info,
                        GpValue::new(gp_state.value()),
                    )
                } else {
                    if self.original_gp_config.is_some_and(|x| !x.pic()) {
                        self.current_gp_value = None;
                    }
                    GpSetInfo::Unknown
                };
                self.gp_sets.entry(instr_rom).or_insert(info);
            }
        }
    }
}

impl InstructionAnalysisResult {
    fn process_branch(
        &mut self,
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

        self.add_referenced_vram(instr_rom, target_vram);

        if self.ranges.in_vram_range(target_vram) {
            self.branch_targets.insert(instr_rom, target_vram);
        } else {
            self.branch_targets_outside.insert(instr_rom, target_vram);
        }
    }

    fn process_func_call(&mut self, instr: &Instruction, instr_rom: Rom, target_vram: Vram) {
        /*
        if instrOffset in self.funcCallInstrOffsets:
            # Already processed
            return
        */

        /*
        if not self.context.isAddressInGlobalRange(target):
            self.funcCallOutsideRangesOffsets[instrOffset] = target
        */

        self.add_referenced_vram(instr_rom, target_vram);
        self.func_calls.insert(instr_rom, target_vram);
        if !instr.opcode().does_link() {
            self.non_linking_func_calls.insert(instr_rom, target_vram);
        }
    }

    fn process_jumptable_jump(
        &mut self,
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
            self.add_referenced_vram(instr_rom, address);

            if jr_reg_data.added_with_gp().is_some() {
                self.lo_rom_added_with_gp.insert(lo_rom);
            }
        }
    }

    fn process_jump_and_link_register(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
    ) {
        if let Some(jr_reg_data) = regs_tracker.get_jr_reg_data(instr) {
            let lo_rom = jr_reg_data.lo_rom();
            let vram = Vram::new(jr_reg_data.address());

            self.indirect_function_call_instr.insert(instr_rom, vram);
            self.indirect_function_call.insert(lo_rom, vram);

            self.add_referenced_vram(instr_rom, vram);
            self.add_referenced_vram(lo_rom, vram);
        }
    }

    fn process_hi(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
    ) {
        regs_tracker.process_hi(instr, instr_rom);
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
        if self.address_per_hi_instr.get(&hi_rom).is_none() {
            self.constant_per_instr.entry(hi_rom).or_insert(constant);
        }
        self.constant_per_instr.insert(instr_rom, constant);

        self.hi_to_lo.insert(hi_rom, instr_rom);
        self.lo_to_hi.insert(instr_rom, hi_rom);

        regs_tracker.process_constant(instr, constant, instr_rom);
    }

    fn process_signed_lo(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        instr_rom: Rom,
        _prev_instr: Option<&Instruction>,
    ) {
        // TODO
        if instr.opcode().does_load()
            && instr
                .field_rs()
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

        if pairing_info.is_gp_got && !self.original_gp_config.is_some_and(|x| x.pic()) {
            return;
        }

        let upper_info = if pairing_info.is_gp_rel {
            None
        } else {
            Some((pairing_info.value, pairing_info.instr_rom))
        };

        if let Some((_upper_half, hi_rom)) = upper_info {
            if let Some((hi_reg, hi_imm)) = self.hi_instrs.get(&hi_rom) {
                if hi_reg.is_global_pointer(instr.abi()) {
                    if let Some(lo_rs) = instr.field_rs() {
                        if instr.opcode().reads_rs() && lo_rs.is_global_pointer(instr.abi()) {
                            if let Some(lo_rt) = instr.field_rt() {
                                if instr.opcode().modifies_rt()
                                    && lo_rt.is_global_pointer(instr.abi())
                                {
                                    if self.original_gp_config.is_some_and(|x| x.pic()) {
                                        /*
                                        # cpload
                                        self.unpairedCploads.append(CploadInfo(luiOffset, instrOffset))
                                        */
                                    } else if let Some(lo_gp_value) =
                                        instr.get_processed_immediate()
                                    {
                                        let hi_gp_value = (*hi_imm as u32) << 16;
                                        let gp_value = GpValue::new(
                                            hi_gp_value.wrapping_add_signed(lo_gp_value),
                                        );
                                        let gp_set =
                                            GpSetInfo::new_address(hi_rom, instr_rom, gp_value);
                                        self.gp_sets.insert(instr_rom, gp_set);
                                        self.gp_sets.insert(hi_rom, gp_set);
                                        if self.original_gp_config.is_some_and(|x| !x.pic()) {
                                            self.current_gp_value = Some(gp_value);
                                        }
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

        let address = self.pair_hi_lo(upper_info.as_ref(), instr, instr_rom);
        if address.is_none() {
            return;
        }
        let address = address.unwrap();
        if upper_info.is_none() && self.original_gp_config.is_some_and(|x| x.pic()) {
            self.process_got_symbol(address, instr_rom);
            regs_tracker.process_lo(instr, address.inner(), instr_rom);
            return;
        }

        if self.process_address(address, upper_info.as_ref(), instr, instr_rom) {
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
        } else if let Some(gp_value) = self.current_gp_value {
            // TODO: implement comparison for Vram and VramOffset
            if lower_half.is_negative() && lower_half.inner().unsigned_abs() > gp_value.inner() {
                None
            } else {
                // TODO: proper abstraction
                Some(Vram::new(
                    gp_value.inner().wrapping_add_signed(lower_half.inner()),
                ))
            }
        } else {
            None
        }
    }

    fn process_got_symbol(&mut self, address: Vram, instr_rom: Rom) {
        self.got_access_addresses.insert(instr_rom, address);
    }

    fn process_address(
        &mut self,
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

        self.add_referenced_vram(instr_rom, address);

        if self
            .address_per_lo_instr
            .insert(instr_rom, address)
            .is_none()
        {
            self.address_per_instr.insert(instr_rom, address);
        }
        if let Some((_upper_half, hi_rom)) = upper_info {
            let entry = self.address_per_hi_instr.entry(*hi_rom);
            if entry.is_vacant() {
                entry.or_insert(address);
                self.address_per_instr.insert(*hi_rom, address);
                self.add_referenced_vram(*hi_rom, address);
            }
            self.hi_to_lo.insert(*hi_rom, instr_rom);
            self.lo_to_hi.insert(instr_rom, *hi_rom);
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

    fn add_referenced_vram(&mut self, instr_rom: Rom, referenced_vram: Vram) {
        if !self.original_gp_config.is_some_and(|x| x.pic()) {
            self.referenced_vrams.insert(referenced_vram);
            self.referenced_vrams_by_rom
                .insert(instr_rom, referenced_vram);
        }
    }
}
