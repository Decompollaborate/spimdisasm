/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_set::BTreeSet;
use rabbitizer::Instruction;

use crate::{context::Context, rom_vram_range::RomVramRange};

use super::{InstructionAnalysisResult, RegisterTracker};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstructionAnalyzer {
    ranges: RomVramRange,

    branches_taken: BTreeSet<u32>,
}

impl InstructionAnalyzer {
    #[must_use]
    pub(crate) fn analyze(
        context: &Context,
        ranges: RomVramRange,
        instrs: &[Instruction],
    ) -> InstructionAnalysisResult {
        assert!(!instrs.is_empty(), "Empty instruction list?. {:?}", ranges,);

        let mut analyzer = Self {
            ranges,
            branches_taken: BTreeSet::new(),
        };
        let mut regs_tracker = RegisterTracker::new();
        let mut result = InstructionAnalysisResult::new(ranges);

        result.process_instr(context, &mut regs_tracker, &instrs[0], None);

        // TODO: maybe implement a way to know which instructions have been processed?

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
                result.process_instr(context, &mut regs_tracker, &instr, Some(&prev_instr));
            }

            analyzer.look_ahead(
                context,
                &mut result,
                &regs_tracker,
                instrs,
                &instr,
                &prev_instr,
                local_offset,
            );

            if prev_instr_opcode.is_jump_with_address() && !prev_instr_opcode.does_link() {
                if let Some(target_vram) = prev_instr.get_branch_vram_generic() {
                    if !ranges.in_vram_range(target_vram) {
                        // The instruction is jumping outside the current function, meaning the
                        // current state of the registers will be garbage for the rest of the
                        // function, so we just reset the tracker.
                        // Jumping outside the current function and skip linking usually is caused
                        // by tail call optimizations.
                        regs_tracker.clear();
                    }
                }
            }

            result.process_prev_func_call(&mut regs_tracker, &instr, &prev_instr);
        }

        result
    }

    // TODO
    #[allow(clippy::too_many_arguments)]
    fn look_ahead(
        &mut self,
        context: &Context,
        result: &mut InstructionAnalysisResult,
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
            result.process_instr(context, &mut regs_tracker, instr, Some(prev_instr));
        }

        while target_local_offset / 4 < instrs.len() {
            let prev_target_instr = instrs[target_local_offset / 4 - 1];
            let target_instr = instrs[target_local_offset / 4];

            if !prev_target_instr.opcode().is_branch_likely()
            /*&& !prev_target_instr.is_unconditional_branch()*/
            {
                result.process_instr(
                    context,
                    &mut regs_tracker,
                    &target_instr,
                    Some(&prev_target_instr),
                );
            }
            self.look_ahead(
                context,
                result,
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

            result.process_prev_func_call(&mut regs_tracker, &target_instr, &prev_target_instr);

            target_local_offset += 4;
        }
    }
}
