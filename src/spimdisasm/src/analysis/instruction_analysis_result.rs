/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use rabbitizer::{
    access_type::AccessType, opcodes::Opcode, registers::Gpr, registers_meta::Register,
    Instruction, Vram,
};

use crate::{
    addresses::{GlobalOffsetTable, GpValue, Rom, RomVramRange},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    config::{GlobalConfig, GpConfig},
};

use super::{InstrProcessedResult, JrRegData, RegisterTracker};

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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UnfinishedCpload {
    hi_rom: Rom,
    lo_rom: Rom,
}

impl UnfinishedCpload {
    fn new(hi_rom: Rom, lo_rom: Rom) -> Self {
        Self { hi_rom, lo_rom }
    }

    fn finish(self, addu_rom: Rom, reg: Gpr) -> CploadInfo {
        CploadInfo::new(self.hi_rom, self.lo_rom, addu_rom, reg)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CploadInfo {
    hi_rom: Rom,
    lo_rom: Rom,
    addu_rom: Rom,
    reg: Gpr,
}

impl CploadInfo {
    fn new(hi_rom: Rom, lo_rom: Rom, addu_rom: Rom, reg: Gpr) -> Self {
        Self {
            hi_rom,
            lo_rom,
            addu_rom,
            reg,
        }
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
pub(crate) struct InstructionAnalysisResult {
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
    branch_calls: UnorderedMap<Rom, Vram>,
    maybe_tail_calls: UnorderedMap<Rom, Vram>,

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

    global_got_addresses: UnorderedMap<Rom, Vram>,
    unpaired_local_got_addresses: UnorderedMap<Rom, Vram>,
    paired_local_got_addresses: UnorderedMap<Rom, Vram>,

    hi_to_lo: UnorderedMap<Rom, Rom>,
    lo_to_hi: UnorderedMap<Rom, Rom>,

    // Jump and link (functions)
    indirect_function_call_instr: UnorderedMap<Rom, Vram>,
    indirect_function_call: UnorderedMap<Rom, Vram>,
    raw_indirect_function_call: UnorderedMap<Rom, Vram>,

    lo_rom_added_with_gp: UnorderedSet<Rom>,

    /// `.cploads` which are not yet fully paired.
    unfinished_cploads: Vec<UnfinishedCpload>,
    /// Rom address for every instruction that is part of a `.cpload`.
    cpload_roms: UnorderedSet<Rom>,
    /// Completed cpload, key: rom of last instruction of the `.cpload`.
    cploads: UnorderedMap<Rom, CploadInfo>,
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
            branch_calls: UnorderedMap::new(),
            maybe_tail_calls: UnorderedMap::new(),
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
            global_got_addresses: UnorderedMap::new(),
            unpaired_local_got_addresses: UnorderedMap::new(),
            paired_local_got_addresses: UnorderedMap::new(),
            hi_to_lo: UnorderedMap::new(),
            lo_to_hi: UnorderedMap::new(),
            indirect_function_call_instr: UnorderedMap::new(),
            indirect_function_call: UnorderedMap::new(),
            raw_indirect_function_call: UnorderedMap::new(),
            lo_rom_added_with_gp: UnorderedSet::new(),
            unfinished_cploads: Vec::new(),
            cpload_roms: UnorderedSet::new(),
            cploads: UnorderedMap::new(),
        }
    }

    #[must_use]
    pub(crate) fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }

    #[must_use]
    pub(crate) fn referenced_vrams(&self) -> &UnorderedSet<Vram> {
        &self.referenced_vrams
    }

    #[must_use]
    pub(crate) fn branch_targets(&self) -> &UnorderedMap<Rom, Vram> {
        &self.branch_targets
    }
    #[must_use]
    pub(crate) fn branch_targets_outside(&self) -> &UnorderedMap<Rom, Vram> {
        &self.branch_targets_outside
    }

    #[must_use]
    pub(crate) fn func_calls(&self) -> &UnorderedMap<Rom, Vram> {
        &self.func_calls
    }
    #[must_use]
    pub(crate) fn branch_calls(&self) -> &UnorderedMap<Rom, Vram> {
        &self.branch_calls
    }
    #[must_use]
    pub(crate) fn maybe_tail_calls(&self) -> &UnorderedMap<Rom, Vram> {
        &self.maybe_tail_calls
    }

    #[must_use]
    pub(crate) fn hi_instrs(&self) -> &UnorderedMap<Rom, (Gpr, u16)> {
        &self.hi_instrs
    }

    #[must_use]
    pub(crate) fn constant_per_instr(&self) -> &UnorderedMap<Rom, u32> {
        &self.constant_per_instr
    }

    #[must_use]
    pub(crate) fn address_per_hi_instr(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_hi_instr
    }
    #[must_use]
    pub(crate) fn address_per_lo_instr(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_lo_instr
    }

    #[must_use]
    pub(crate) fn referenced_jumptables(&self) -> &UnorderedMap<Rom, Vram> {
        &self.referenced_jumptables
    }

    #[must_use]
    pub(crate) fn type_info_per_address(
        &self,
    ) -> &UnorderedMap<Vram, UnorderedMap<(AccessType, bool), u32>> {
        &self.type_info_per_address
    }

    #[must_use]
    pub(crate) fn handwritten_instrs(&self) -> &UnorderedSet<Rom> {
        &self.handwritten_instrs
    }

    #[must_use]
    pub(crate) fn gp_sets(&self) -> &UnorderedMap<Rom, GpSetInfo> {
        &self.gp_sets
    }

    #[must_use]
    pub(crate) fn global_got_addresses(&self) -> &UnorderedMap<Rom, Vram> {
        &self.global_got_addresses
    }
    #[must_use]
    pub(crate) fn unpaired_local_got_addresses(&self) -> &UnorderedMap<Rom, Vram> {
        &self.unpaired_local_got_addresses
    }
    #[must_use]
    pub(crate) fn paired_local_got_addresses(&self) -> &UnorderedMap<Rom, Vram> {
        &self.paired_local_got_addresses
    }

    #[must_use]
    pub(crate) fn raw_indirect_function_call(&self) -> &UnorderedMap<Rom, Vram> {
        &self.raw_indirect_function_call
    }

    #[must_use]
    pub(crate) fn lo_rom_added_with_gp(&self) -> &UnorderedSet<Rom> {
        &self.lo_rom_added_with_gp
    }

    #[must_use]
    pub(crate) fn cpload_roms(&self) -> &UnorderedSet<Rom> {
        &self.cpload_roms
    }
}

impl InstructionAnalysisResult {
    pub(crate) fn process_instr(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) {
        let instr_rom = self.rom_from_instr(instr);

        if instr.is_likely_handwritten() {
            self.handwritten_instrs.insert(instr_rom);
        }

        let instr_processed_result = regs_tracker.process_instruction(
            instr,
            instr_rom,
            global_offset_table,
            self.original_gp_config.as_ref(),
            self.current_gp_value.as_ref(),
        );

        let mut special_case = false;

        match instr_processed_result {
            InstrProcessedResult::DirectLinkingCall { target_vram } => {
                self.process_func_call(instr_rom, target_vram);
            }
            InstrProcessedResult::LinkingBranch { target_vram } => {
                self.process_branch_call(instr_rom, target_vram);
            }
            InstrProcessedResult::MaybeDirectTailCall { target_vram } => {
                self.process_maybe_tail_call(instr_rom, target_vram);
            }
            InstrProcessedResult::Branch { target_vram } => {
                self.process_branch(instr_rom, target_vram);
            }
            InstrProcessedResult::JumptableJump { jr_reg_data } => {
                self.process_jumptable_jump(jr_reg_data, instr_rom);
            }
            InstrProcessedResult::UnknownRegInfoJump { .. } => {}
            InstrProcessedResult::DereferencedRegisterLink { jr_reg_data } => {
                self.process_jump_and_link_register(jr_reg_data, instr_rom, true)
            }
            InstrProcessedResult::RawRegisterLink { jr_reg_data } => {
                self.process_jump_and_link_register(jr_reg_data, instr_rom, false)
            }
            InstrProcessedResult::UnknownJumpAndLinkRegister { .. } => {}
            InstrProcessedResult::Hi { dst_reg, value } => {
                self.hi_instrs
                    .insert(instr_rom, (dst_reg, (value >> 16) as u16));
            }
            InstrProcessedResult::PairedLo { hi_rom, vram, .. } => {
                if let Some((hi_reg, hi_imm)) = self.hi_instrs.get(&hi_rom) {
                    if hi_reg.is_global_pointer(instr.abi()) {
                        if let Some(lo_rs) = instr.field_rs() {
                            if instr.opcode().reads_rs() && lo_rs.is_global_pointer(instr.abi()) {
                                if let Some(lo_rt) = instr.field_rt() {
                                    if instr.opcode().modifies_rt()
                                        && lo_rt.is_global_pointer(instr.abi())
                                    {
                                        if self.original_gp_config.is_some_and(|x| x.pic()) {
                                            self.unfinished_cploads
                                                .push(UnfinishedCpload::new(hi_rom, instr_rom));
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
                                        // Avoid counting this pairing as a normal symbol
                                        special_case = true;
                                    }
                                }
                            }
                        }
                    }
                }

                if !special_case {
                    self.process_address(vram, Some(hi_rom), instr, instr_rom);
                }
            }
            InstrProcessedResult::GpRel { vram, .. } => {
                self.process_address(vram, None, instr, instr_rom);
            }
            InstrProcessedResult::GpGotGlobal { vram, .. }
            | InstrProcessedResult::GpGotLazyResolver { vram, .. } => {
                self.process_global_got_symbol(vram, instr_rom);
            }
            InstrProcessedResult::GpGotLocal { vram, .. } => {
                self.process_local_got_symbol(vram, instr_rom);
            }
            InstrProcessedResult::PairedGpGotLo {
                upper_rom, vram, ..
            } => {
                self.unpaired_local_got_addresses.remove(&upper_rom);
                self.paired_local_got_addresses.insert(upper_rom, vram);
                self.paired_local_got_addresses.insert(instr_rom, vram);

                self.add_referenced_vram(instr_rom, vram);
                self.hi_to_lo.insert(upper_rom, instr_rom);
                self.lo_to_hi.insert(instr_rom, upper_rom);
            }
            InstrProcessedResult::DanglingLo { .. } => {}
            InstrProcessedResult::Constant { constant, hi_rom } => {
                self.process_constant(constant, instr_rom, hi_rom);
            }
            InstrProcessedResult::UnpairedConstant { .. } => {}
            InstrProcessedResult::UnhandledOpcode { opcode } => {
                if opcode == Opcode::core_addu {
                    let rd = instr.field_rd();
                    let rs = instr.field_rs();
                    let rt = instr.field_rt();

                    if let (Some(rd), Some(rs), Some(rt)) = (rd, rs, rt) {
                        let rs_is_gp = rs.is_global_pointer(instr.abi());
                        let rt_is_gp = rt.is_global_pointer(instr.abi());

                        if rd.is_global_pointer(instr.abi()) {
                            // special check for .cpload
                            if rs.is_global_pointer(instr.abi()) {
                                if let Some(unfinished_cpload) = self.unfinished_cploads.pop() {
                                    let cpload = unfinished_cpload.finish(instr_rom, rt);

                                    self.cpload_roms.insert(cpload.hi_rom);
                                    self.cpload_roms.insert(cpload.lo_rom);
                                    self.cpload_roms.insert(cpload.addu_rom);
                                    self.cploads.insert(instr_rom, cpload);
                                }
                            }
                        } else if rs_is_gp ^ rt_is_gp {
                            let reg = if !rs_is_gp { rs } else { rt };

                            if reg == rd {
                                // We have something like
                                // addu        $t7, $t7, $gp

                                regs_tracker.set_added_with_gp(reg, instr_rom);
                                special_case = true;
                            }
                        }
                    }
                }
            }
            InstrProcessedResult::InvalidInstr { .. } => {}
        }

        // Consider including this info in `InstrProcessedResult::PairedLo` and family
        self.process_symbol_dereference_type(regs_tracker, instr, instr_rom);

        if !special_case {
            regs_tracker.overwrite_registers(instr, instr_rom);
        }

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
    fn process_branch(&mut self, instr_rom: Rom, target_vram: Vram) {
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

    fn process_func_call(&mut self, instr_rom: Rom, target_vram: Vram) {
        self.add_referenced_vram(instr_rom, target_vram);
        self.func_calls.insert(instr_rom, target_vram);
    }

    fn process_branch_call(&mut self, instr_rom: Rom, target_vram: Vram) {
        self.add_referenced_vram(instr_rom, target_vram);
        self.branch_calls.insert(instr_rom, target_vram);
    }

    fn process_maybe_tail_call(&mut self, instr_rom: Rom, target_vram: Vram) {
        self.add_referenced_vram(instr_rom, target_vram);
        self.maybe_tail_calls.insert(instr_rom, target_vram);
    }

    fn process_jumptable_jump(&mut self, jr_reg_data: JrRegData, instr_rom: Rom) {
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

    fn process_jump_and_link_register(
        &mut self,
        jr_reg_data: JrRegData,
        instr_rom: Rom,
        was_dereferenced: bool,
    ) {
        let lo_rom = jr_reg_data.lo_rom();
        let vram = Vram::new(jr_reg_data.address());

        if was_dereferenced {
            self.indirect_function_call_instr.insert(instr_rom, vram);
            self.indirect_function_call.insert(lo_rom, vram);
        } else {
            self.raw_indirect_function_call.insert(lo_rom, vram);
        }

        self.add_referenced_vram(instr_rom, vram);
        self.add_referenced_vram(lo_rom, vram);
    }

    fn process_constant(&mut self, constant: u32, instr_rom: Rom, hi_rom: Rom) {
        // self.referencedConstants.add(constant)

        // self.constantHiInstrOffset[luiOffset] = constant
        // self.constantLoInstrOffset[lowerOffset] = constant
        if self.address_per_hi_instr.get(&hi_rom).is_none() {
            self.constant_per_instr.entry(hi_rom).or_insert(constant);
        }
        self.constant_per_instr.insert(instr_rom, constant);

        self.hi_to_lo.insert(hi_rom, instr_rom);
        self.lo_to_hi.insert(instr_rom, hi_rom);
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
    fn process_global_got_symbol(&mut self, address: Vram, instr_rom: Rom) {
        self.global_got_addresses.insert(instr_rom, address);
    }

    fn process_local_got_symbol(&mut self, address: Vram, instr_rom: Rom) {
        self.unpaired_local_got_addresses.insert(instr_rom, address);
    }

    fn process_address(
        &mut self,
        address: Vram,
        hi_rom: Option<Rom>,
        instr: &Instruction,
        instr_rom: Rom,
    ) {
        self.add_referenced_vram(instr_rom, address);

        self.address_per_lo_instr
            .entry(instr_rom)
            .or_insert(address);
        if let Some(hi_rom) = hi_rom {
            let entry = self.address_per_hi_instr.entry(hi_rom);
            if entry.is_vacant() {
                entry.or_insert(address);
                self.address_per_instr.insert(hi_rom, address);
                self.add_referenced_vram(hi_rom, address);
            }
            self.hi_to_lo.insert(hi_rom, instr_rom);
            self.lo_to_hi.insert(instr_rom, hi_rom);
        } else {
            /*
            self.symbolGpInstrOffset[lowerOffset] = address
            self.gpReferencedSymbols.add(address)
            self.symbolInstrOffset[lowerOffset] = address
            */
        }

        self.process_symbol_type(address, instr, instr_rom);
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
