/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use rabbitizer::{Instruction, Vram};

use crate::{context::Context, rom_address::RomAddress, rom_vram_range::RomVramRange};

use super::RegisterTracker;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstructionAnalysisResult {
    ranges: RomVramRange,

    /// Every referenced vram found.
    referenced_vrams: BTreeSet<Vram>,
    /// Key is the rom of the instruction referencing that address, value is the referenced address.
    referenced_vrams_by_rom: BTreeMap<RomAddress, Vram>,

    /// Key is the rom of the branch instruction, value is the vram target for that instruction.
    branch_targets: BTreeMap<RomAddress, Vram>,

    /// Key is the rom of the instruction, value is the address of the called function.
    func_calls: BTreeMap<RomAddress, Vram>,
}

impl InstructionAnalysisResult {
    #[must_use]
    pub(crate) fn new(ranges: RomVramRange) -> Self {
        Self {
            ranges,
            referenced_vrams: BTreeSet::new(),
            referenced_vrams_by_rom: BTreeMap::new(),
            branch_targets: BTreeMap::new(),
            func_calls: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn referenced_vrams(&self) -> &BTreeSet<Vram> {
        &self.referenced_vrams
    }

    #[must_use]
    pub fn branch_targets(&self) -> &BTreeMap<RomAddress, Vram> {
        &self.branch_targets
    }

    #[must_use]
    pub fn func_calls(&self) -> &BTreeMap<RomAddress, Vram> {
        &self.func_calls
    }
}

impl InstructionAnalysisResult {
    pub(crate) fn process_prev_func_call(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        prev_instr: &Instruction,
    ) {
        regs_tracker.unset_registers_after_func_call(instr, prev_instr);
    }

    pub(crate) fn process_instr(
        &mut self,
        context: &Context,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        _prev_instr: Option<&Instruction>,
    ) {
        if let Some(target_vram) = instr.get_branch_vram_generic() {
            // instr.opcode().is_branch() or instr.is_unconditional_branch()
            self.process_branch(context, regs_tracker, instr, target_vram);
        } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
            // instr.opcode().is_jump_with_address()
            self.process_func_call(context, instr, target_vram);
        }
    }
}

impl InstructionAnalysisResult {
    fn process_branch(
        &mut self,
        context: &Context,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        target_vram: Vram,
    ) {
        if !self.ranges.in_vram_range(target_vram) {
            return;
        }

        regs_tracker.process_branch(instr);

        /*
        if instrOffset in self.branchInstrOffsets:
            # Already processed
            return
        */

        let instr_rom = self
            .ranges
            .rom_from_vram(instr.vram())
            .expect("This should not panic");
        self.add_referenced_vram(context, instr_rom, target_vram);
        self.branch_targets.insert(instr_rom, target_vram);
    }

    fn process_func_call(&mut self, context: &Context, instr: &Instruction, target_vram: Vram) {
        /*
        if instrOffset in self.funcCallInstrOffsets:
            # Already processed
            return
        */

        /*
        if not self.context.isAddressInGlobalRange(target):
            self.funcCallOutsideRangesOffsets[instrOffset] = target
        */

        let instr_rom = self
            .ranges
            .rom_from_vram(instr.vram())
            .expect("This should not panic");
        self.add_referenced_vram(context, instr_rom, target_vram);
        self.func_calls.insert(instr_rom, target_vram);
    }
}

impl InstructionAnalysisResult {
    fn add_referenced_vram(
        &mut self,
        context: &Context,
        instr_rom: RomAddress,
        referenced_vram: Vram,
    ) {
        if let Some(gp_config) = context.global_config().gp_config() {
            if !gp_config.pic() {
                self.referenced_vrams.insert(referenced_vram);
                self.referenced_vrams_by_rom
                    .insert(instr_rom, referenced_vram);
            }
        }
    }
}
