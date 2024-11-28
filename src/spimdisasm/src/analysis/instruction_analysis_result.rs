/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use rabbitizer::{Instruction, Vram};

use crate::{address_range::AddressRange, context::Context};

use super::RegisterTracker;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstructionAnalysisResult {
    vram_range: AddressRange<Vram>,

    /// Every referenced vram found.
    referenced_vrams: BTreeSet<Vram>,

    /// Key is the vram of the branch instruction. Value is the vram target for that instruction
    branch_targets: BTreeMap<Vram, Vram>,
}

impl InstructionAnalysisResult {
    #[must_use]
    pub(crate) fn new(vram_range: AddressRange<Vram>) -> Self {
        Self {
            vram_range,
            referenced_vrams: BTreeSet::new(),
            branch_targets: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn referenced_vrams(&self) -> &BTreeSet<Vram> {
        &self.referenced_vrams
    }

    #[must_use]
    pub fn branch_targets(&self) -> &BTreeMap<Vram, Vram> {
        &self.branch_targets
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
            self.process_branch(context, regs_tracker, instr, target_vram);
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
        if !self.vram_range.in_range(target_vram) {
            return;
        }

        regs_tracker.process_branch(instr);

        /*
        if instrOffset in self.branchInstrOffsets:
            # Already processed
            return
        */

        if let Some(gp_config) = context.global_config().gp_config() {
            if !gp_config.pic() {
                self.referenced_vrams.insert(target_vram);
            }
        }

        self.branch_targets.insert(instr.vram(), target_vram);
    }
}
