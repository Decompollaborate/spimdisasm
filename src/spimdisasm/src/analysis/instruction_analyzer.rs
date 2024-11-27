/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_set::BTreeSet;
use rabbitizer::{Instruction, Vram};

use crate::context::Context;

use super::RegisterTracker;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstructionAnalyzer {
    vram_start: Vram,
    vram_end: Vram,

    branches_taken: BTreeSet<u32>,
}

impl InstructionAnalyzer {
    #[must_use]
    pub(crate) fn analyze(
        _context: &Context,
        vram_start: Vram,
        vram_end: Vram,
        instrs: &[Instruction],
    ) -> Self {
        assert!(
            !instrs.is_empty(),
            "Empty instruction list?. {:?}",
            vram_start,
        );

        let mut analyzer = Self {
            vram_start,
            vram_end,
            branches_taken: BTreeSet::new(),
        };
        let mut regs_tracker = RegisterTracker::new();

        analyzer.process_instr(&mut regs_tracker, &instrs[0], None);

        for (i, w) in instrs.windows(2).enumerate() {
            let prev_instr = w[0];
            let instr = w[1];
            let local_offset = (i + 1) * 4;

            if !instr.is_valid() {
                continue;
            }

            /*
            if instr.isLikelyHandwritten() and not self.isRsp:
                self.isLikelyHandwritten = True
                self.endOfLineComment[instructionOffset//4] = " /* handwritten instruction */
"
            */

            let prev_instr_opcode = prev_instr.opcode();
            if !prev_instr.opcode().is_branch_likely()
            /*&& !prev_instr.is_unconditional_branch()*/
            {
                analyzer.process_instr(&mut regs_tracker, &instr, Some(&prev_instr));
            }

            analyzer.look_ahead(&regs_tracker, instrs, &instr, &prev_instr, local_offset);

            if prev_instr_opcode.is_jump_with_address() && !prev_instr_opcode.does_link() {
                if let Some(target_vram) = prev_instr.get_branch_vram_generic() {
                    if target_vram < vram_start || target_vram >= vram_end {
                        // The instruction is jumping outside the current function, meaning the
                        // current state of the registers will be garbage for the rest of the
                        // function, so we just reset the tracker.
                        // Jumping outside the current function and skip linking usually is caused
                        // by tail call optimizations.
                        regs_tracker.clear();
                    }
                }
            }

            analyzer.process_prev_func_call(&mut regs_tracker, &instr, &prev_instr);
        }

        analyzer
    }

    fn look_ahead(
        &mut self,
        original_regs_tracker: &RegisterTracker,
        instrs: &[Instruction],
        instr: &Instruction,
        prev_instr: &Instruction,
        local_offset: usize,
    ) {
        let branch_offset = if let Some(offset) = prev_instr.get_branch_offset_generic() {
            offset
        } else {
            return;
        };

        if !self.branches_taken.insert(local_offset as u32) {
            // If we already processed this branch then don't do it again.
            return;
        }

        let prev_local_offset = local_offset - 4;
        let mut target_local_offset = {
            let temp = prev_local_offset as i32 + branch_offset.inner();
            if temp <= 0 {
                // Avoid jumping outside of the function.
                return;
            }
            temp as usize
        };

        // Make a copy
        let mut regs_tracker = *original_regs_tracker;

        if prev_instr.opcode().is_branch_likely()
        /*|| prev_instr.is_unconditional_branch()*/
        {
            self.process_instr(&mut regs_tracker, instr, Some(prev_instr));
        }

        while target_local_offset < instrs.len() {
            let prev_target_instr = instrs[target_local_offset / 4 - 1];
            let target_instr = instrs[target_local_offset / 4];

            if prev_instr.opcode().is_branch_likely()
            /*|| prev_instr.is_unconditional_branch()*/
            {
                self.process_instr(&mut regs_tracker, &target_instr, Some(&prev_target_instr));
            }
            self.look_ahead(
                &regs_tracker,
                instrs,
                &target_instr,
                &prev_target_instr,
                target_local_offset,
            );

            if prev_target_instr.is_unconditional_branch() {
                // Since we took the branch on the previous `look_ahead` call then we don't have
                // anything else to process here.
                return;
            }
            if prev_target_instr.opcode().is_jump() && !prev_target_instr.opcode().does_link() {
                // Technically this is another form of unconditional branching.
                return;
            }

            self.process_prev_func_call(&mut regs_tracker, &target_instr, &prev_target_instr);

            target_local_offset += 4;
        }
    }
}

impl InstructionAnalyzer {
    fn process_prev_func_call(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        prev_instr: &Instruction,
    ) {
        regs_tracker.unset_registers_after_func_call(instr, prev_instr);
    }

    fn process_instr(
        &mut self,
        _regs_tracker: &mut RegisterTracker,
        _instr: &Instruction,
        _prev_instr: Option<&Instruction>,
    ) {
    }
}
