/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{access_type::AccessType, registers::Gpr, Instruction, Vram};

use crate::{
    addresses::{GlobalOffsetTable, Rom, RomVramRange},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
};

use super::{
    InstrOpJumptable, InstrOpLink, InstrOpPairedAddress, InstrOpRegisterOperation, InstrOpTailCall,
    InstructionOperation, RegisterTracker,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GpSetInfo {
    hi_rom: Rom,
    lo_rom: Rom,
}
impl GpSetInfo {
    fn new_address(hi_rom: Rom, lo_rom: Rom) -> Self {
        Self { hi_rom, lo_rom }
    }

    pub(crate) fn hi_rom(&self) -> Rom {
        self.hi_rom
    }
    pub(crate) fn lo_rom(&self) -> Rom {
        self.lo_rom
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstructionAnalysisResult {
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

    address_per_got_hi: UnorderedMap<Rom, Vram>,
    address_per_got_lo: UnorderedMap<Rom, Vram>,

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

    /// Rom address for every instruction that is part of a `.cpload`.
    cpload_roms: UnorderedSet<Rom>,
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
            branch_calls: UnorderedMap::new(),
            maybe_tail_calls: UnorderedMap::new(),
            referenced_jumptables: UnorderedMap::new(),
            hi_instrs: UnorderedMap::new(),
            non_lo_instrs: UnorderedSet::new(),
            constant_per_instr: UnorderedMap::new(),
            address_per_instr: UnorderedMap::new(),
            address_per_hi_instr: UnorderedMap::new(),
            address_per_lo_instr: UnorderedMap::new(),
            address_per_got_hi: UnorderedMap::new(),
            address_per_got_lo: UnorderedMap::new(),
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
            cpload_roms: UnorderedSet::new(),
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
    pub(crate) fn address_per_got_hi(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_got_hi
    }
    #[must_use]
    pub(crate) fn address_per_got_lo(&self) -> &UnorderedMap<Rom, Vram> {
        &self.address_per_got_lo
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum InstrAnalysisInfo {
    No,
    JumptableJump { jumptable_vram: Vram },
}

impl InstructionAnalysisResult {
    pub(crate) fn process_instr(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> InstrAnalysisInfo {
        let instr_rom = self.rom_from_instr(instr);

        if instr.is_likely_handwritten() {
            self.handwritten_instrs.insert(instr_rom);
        }

        let instr_processed_result =
            regs_tracker.process_instruction(instr, instr_rom, global_offset_table);

        match instr_processed_result {
            InstructionOperation::Link { info } => match info {
                InstrOpLink::DirectLinkingCall { target_vram } => {
                    self.process_func_call(instr_rom, target_vram);
                    InstrAnalysisInfo::No
                }
                InstrOpLink::LinkingBranch { target_vram } => {
                    self.process_branch_call(instr_rom, target_vram);
                    InstrAnalysisInfo::No
                }
                InstrOpLink::RawRegisterLink { vram, rom }
                | InstrOpLink::Call16RegisterLink { vram, rom }
                | InstrOpLink::CallHiLoRegisterLink { vram, rom } => {
                    self.process_jump_and_link_register(instr_rom, vram, rom, false);
                    InstrAnalysisInfo::No
                }
                InstrOpLink::DereferencedRegisterLink {
                    dereferenced_vram,
                    dereferenced_rom,
                } => {
                    self.process_jump_and_link_register(
                        instr_rom,
                        dereferenced_vram,
                        dereferenced_rom,
                        true,
                    );
                    InstrAnalysisInfo::No
                }
                InstrOpLink::UnknownJumpAndLinkRegister { .. } => InstrAnalysisInfo::No,
            },

            InstructionOperation::TailCall { info } => match info {
                InstrOpTailCall::MaybeDirectTailCall { target_vram } => {
                    self.process_maybe_tail_call(instr_rom, target_vram);
                    InstrAnalysisInfo::No
                }
                InstrOpTailCall::RawRegisterTailCall { .. } => {
                    // TODO: do something with this info
                    InstrAnalysisInfo::No
                }
                InstrOpTailCall::DereferencedRegisterTailCall { .. }
                | InstrOpTailCall::UnknownRegisterJump { .. } => InstrAnalysisInfo::No,
            },

            InstructionOperation::JumptableJump {
                jumptable_vram,
                dereferenced_rom,
                info,
            } => {
                self.process_jumptable_jump(
                    instr_rom,
                    jumptable_vram,
                    dereferenced_rom,
                    info == InstrOpJumptable::Pic,
                );
                InstrAnalysisInfo::JumptableJump { jumptable_vram }
            }

            InstructionOperation::ReturnJump => InstrAnalysisInfo::No,

            InstructionOperation::Branch { target_vram } => {
                self.process_branch(instr_rom, target_vram);
                InstrAnalysisInfo::No
            }

            InstructionOperation::Hi { dst_reg, value } => {
                self.hi_instrs
                    .insert(instr_rom, (dst_reg, (value >> 16) as u16));
                InstrAnalysisInfo::No
            }

            InstructionOperation::PairedAddress { vram, info } => match info {
                InstrOpPairedAddress::PairedLo {
                    hi_rom,
                    access_info,
                    ..
                } => {
                    self.process_address(vram, Some(hi_rom), instr_rom);
                    if let Some(access_info) = access_info {
                        self.apply_symbol_type(vram, instr_rom, access_info);
                    }
                    InstrAnalysisInfo::No
                }
                InstrOpPairedAddress::GpRel { access_info, .. } => {
                    self.process_address(vram, None, instr_rom);
                    if let Some(access_info) = access_info {
                        self.apply_symbol_type(vram, instr_rom, access_info);
                    }
                    InstrAnalysisInfo::No
                }
                InstrOpPairedAddress::GpGotGlobal { .. }
                | InstrOpPairedAddress::GpGotLazyResolver { .. } => {
                    self.process_global_got_symbol(vram, instr_rom);
                    InstrAnalysisInfo::No
                }
                InstrOpPairedAddress::GpGotLocal { .. } => {
                    self.process_local_got_symbol(vram, instr_rom);
                    InstrAnalysisInfo::No
                }
                InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom,
                    access_info,
                    ..
                } => {
                    // TODO: move all this code to a function
                    self.unpaired_local_got_addresses.remove(&upper_rom);
                    self.paired_local_got_addresses.insert(upper_rom, vram);
                    self.paired_local_got_addresses.insert(instr_rom, vram);

                    self.add_referenced_vram(instr_rom, vram);
                    self.hi_to_lo.insert(upper_rom, instr_rom);
                    self.lo_to_hi.insert(instr_rom, upper_rom);
                    if let Some(access_info) = access_info {
                        self.apply_symbol_type(vram, instr_rom, access_info);
                    }
                    InstrAnalysisInfo::No
                }
                InstrOpPairedAddress::PairedGotLo { hi_rom } => {
                    self.process_paired_got_lo(vram, hi_rom, instr_rom);
                    InstrAnalysisInfo::No
                }
            },

            InstructionOperation::GpSet { hi_rom } => {
                self.gp_sets
                    .insert(instr_rom, GpSetInfo::new_address(hi_rom, instr_rom));
                InstrAnalysisInfo::No
            }
            InstructionOperation::DereferencedRawAddress {
                original_address,
                access_info,
                ..
            } => {
                self.apply_symbol_type(original_address, instr_rom, access_info);
                InstrAnalysisInfo::No
            }
            InstructionOperation::DanglingLo { .. } => InstrAnalysisInfo::No,
            InstructionOperation::Constant { constant, hi_rom } => {
                self.process_constant(constant, instr_rom, hi_rom);
                InstrAnalysisInfo::No
            }
            InstructionOperation::UnpairedConstant { .. } => InstrAnalysisInfo::No,
            InstructionOperation::RegisterOperation { info } => match info {
                InstrOpRegisterOperation::SuspectedCpload { hi_rom, lo_rom, .. } => {
                    self.cpload_roms.insert(hi_rom);
                    self.cpload_roms.insert(lo_rom);
                    self.cpload_roms.insert(instr_rom);
                    InstrAnalysisInfo::No
                }
                InstrOpRegisterOperation::RegisterAddition { .. }
                | InstrOpRegisterOperation::RegisterSubtraction { .. }
                | InstrOpRegisterOperation::Or { .. } => InstrAnalysisInfo::No,
            },
            InstructionOperation::UnhandledOpcode { .. }
            | InstructionOperation::InvalidInstr { .. } => InstrAnalysisInfo::No,
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

    fn process_jumptable_jump(
        &mut self,
        instr_rom: Rom,
        jumptable_vram: Vram,
        dereferenced_rom: Rom,
        added_with_gp: bool,
    ) {
        self.referenced_jumptables
            .insert(dereferenced_rom, jumptable_vram);

        // self.jumpRegisterIntrOffset[instrOffset] = jumptable_vram
        self.add_referenced_vram(instr_rom, jumptable_vram);

        if added_with_gp {
            self.lo_rom_added_with_gp.insert(dereferenced_rom);
        }
    }

    fn process_jump_and_link_register(
        &mut self,
        instr_rom: Rom,
        address: Vram,
        lo_rom: Rom,
        was_dereferenced: bool,
    ) {
        if was_dereferenced {
            self.indirect_function_call_instr.insert(instr_rom, address);
            self.indirect_function_call.insert(lo_rom, address);
        } else {
            self.raw_indirect_function_call.insert(lo_rom, address);
        }

        self.add_referenced_vram(instr_rom, address);
        self.add_referenced_vram(lo_rom, address);
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
}

impl InstructionAnalysisResult {
    fn process_global_got_symbol(&mut self, address: Vram, instr_rom: Rom) {
        self.global_got_addresses.insert(instr_rom, address);
    }

    fn process_local_got_symbol(&mut self, address: Vram, instr_rom: Rom) {
        self.unpaired_local_got_addresses.insert(instr_rom, address);
    }

    fn process_address(&mut self, address: Vram, hi_rom: Option<Rom>, instr_rom: Rom) {
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
    }

    fn process_paired_got_lo(&mut self, vram: Vram, hi_rom: Rom, lo_rom: Rom) {
        self.add_referenced_vram(lo_rom, vram);

        self.address_per_got_lo.insert(lo_rom, vram);

        let entry = self.address_per_got_hi.entry(hi_rom);
        if entry.is_vacant() {
            entry.or_insert(vram);
            self.add_referenced_vram(hi_rom, vram);
        }
    }

    fn apply_symbol_type(
        &mut self,
        address: Vram,
        instr_rom: Rom,
        access_info: (AccessType, bool),
    ) {
        let (access_type, unsigned_memory_address) = access_info;
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

impl InstructionAnalysisResult {
    fn rom_from_instr(&self, instr: &Instruction) -> Rom {
        self.ranges
            .rom_from_vram(instr.vram())
            .expect("This should not panic")
    }

    fn add_referenced_vram(&mut self, instr_rom: Rom, referenced_vram: Vram) {
        self.referenced_vrams.insert(referenced_vram);
        self.referenced_vrams_by_rom
            .insert(instr_rom, referenced_vram);
    }
}
