/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Instruction;

use crate::{
    addresses::{RomVramRange, Vram},
    collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet},
    context::{Context, OwnedSegmentNotFoundError},
    parent_segment_info::ParentSegmentInfo,
};

use super::{InstructionAnalysisResult, RegisterTracker};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionAnalyzer {
    ranges: RomVramRange,

    branches_taken: UnorderedSet<(Vram, bool)>,
}

impl InstructionAnalyzer {
    pub(crate) fn analyze(
        context: &Context,
        parent_info: &ParentSegmentInfo,
        ranges: RomVramRange,
        instrs: &[Instruction],
    ) -> Result<InstructionAnalysisResult, OwnedSegmentNotFoundError> {
        assert!(!instrs.is_empty(), "Empty instruction list?. {:?}", ranges,);

        let mut analyzer = Self {
            ranges,
            branches_taken: UnorderedSet::new(),
        };
        let mut regs_tracker = RegisterTracker::new();
        let mut result = InstructionAnalysisResult::new(ranges, context.global_config());

        // The below iteration skips the first instruction so we have to process it explicitly here.
        result.process_instr(&mut regs_tracker, &instrs[0], None);

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
                result.process_instr(&mut regs_tracker, &instr, Some(&prev_instr));
            }

            analyzer.look_ahead(
                context,
                parent_info,
                &mut result,
                &regs_tracker,
                instrs,
                &instr,
                &prev_instr,
                local_offset,
                prev_instr.opcode().is_branch_likely(),
            )?;

            analyzer.follow_jumptable(
                context,
                parent_info,
                &mut result,
                &regs_tracker,
                instrs,
                &prev_instr,
                false,
            )?;

            if (prev_instr_opcode.is_jump() && !prev_instr_opcode.does_link())
                || prev_instr.is_unconditional_branch()
            {
                regs_tracker.clear();
            }

            result.process_prev_func_call(&mut regs_tracker, &prev_instr);
        }

        Ok(result)
    }

    // TODO
    #[allow(clippy::too_many_arguments)]
    fn look_ahead(
        &mut self,
        context: &Context,
        parent_info: &ParentSegmentInfo,
        result: &mut InstructionAnalysisResult,
        original_regs_tracker: &RegisterTracker,
        instrs: &[Instruction],
        instr: &Instruction,
        prev_instr: &Instruction,
        local_offset: usize,
        prev_is_likely: bool,
    ) -> Result<(), OwnedSegmentNotFoundError> {
        let branch_offset = if let Some(offset) = prev_instr.get_branch_offset_generic() {
            offset
        } else {
            return Ok(());
        };

        if !self
            .branches_taken
            .insert((prev_instr.vram(), prev_is_likely))
        {
            // If we already processed this branch then don't do it again.
            return Ok(());
        }

        let prev_local_offset = local_offset - 4;
        let target_local_offset = {
            let temp = prev_local_offset as i32 + branch_offset.inner();
            if temp <= 0 {
                // Avoid jumping outside of the function.
                return Ok(());
            }
            temp as usize
        };

        // Make a copy
        let mut regs_tracker = *original_regs_tracker;

        if prev_is_likely
        /*|| prev_instr.is_unconditional_branch()*/
        {
            result.process_instr(&mut regs_tracker, instr, Some(prev_instr));
        }

        self.look_ahead_impl(
            context,
            parent_info,
            result,
            regs_tracker,
            instrs,
            target_local_offset,
            prev_is_likely,
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn follow_jumptable(
        &mut self,
        context: &Context,
        parent_info: &ParentSegmentInfo,
        result: &mut InstructionAnalysisResult,
        original_regs_tracker: &RegisterTracker,
        instrs: &[Instruction],
        prev_instr: &Instruction,
        prev_is_likely: bool,
    ) -> Result<(), OwnedSegmentNotFoundError> {
        let jumptable_address =
            if let Some(jr_reg_data) = original_regs_tracker.get_jr_reg_data(prev_instr) {
                if jr_reg_data.branch_info().is_none() {
                    Vram::new(jr_reg_data.address())
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            };

        if !self
            .branches_taken
            .insert((prev_instr.vram(), prev_is_likely))
        {
            // If we already processed this branch then don't do it again.
            return Ok(());
        }

        let jumptable_ref = if let Some(jumptable_ref) = context
            .find_owned_segment(parent_info)?
            .find_reference(jumptable_address, FindSettings::new(false))
        {
            jumptable_ref
        } else {
            return Ok(());
        };

        for jtbl_label_vram in jumptable_ref.table_labels() {
            if result.ranges().in_vram_range(*jtbl_label_vram) {
                let target_local_offset =
                    (*jtbl_label_vram - result.ranges().vram().start()).inner() as usize;

                self.look_ahead_impl(
                    context,
                    parent_info,
                    result,
                    *original_regs_tracker,
                    instrs,
                    target_local_offset,
                    prev_is_likely,
                )?;
            }
        }

        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
    fn look_ahead_impl(
        &mut self,
        context: &Context,
        parent_info: &ParentSegmentInfo,
        result: &mut InstructionAnalysisResult,
        mut regs_tracker: RegisterTracker,
        instrs: &[Instruction],
        mut target_local_offset: usize,
        prev_is_likely: bool,
    ) -> Result<(), OwnedSegmentNotFoundError> {
        while target_local_offset / 4 < instrs.len() {
            let prev_target_instr = &instrs[target_local_offset / 4 - 1];
            let target_instr = &instrs[target_local_offset / 4];

            if !prev_target_instr.opcode().is_branch_likely()
            /*&& !prev_target_instr.is_unconditional_branch()*/
            {
                result.process_instr(&mut regs_tracker, target_instr, Some(prev_target_instr));
            }
            self.look_ahead(
                context,
                parent_info,
                result,
                &regs_tracker,
                instrs,
                target_instr,
                prev_target_instr,
                target_local_offset,
                prev_is_likely || prev_target_instr.opcode().is_branch_likely(),
            )?;

            self.follow_jumptable(
                context,
                parent_info,
                result,
                &regs_tracker,
                instrs,
                prev_target_instr,
                prev_is_likely,
            )?;

            if prev_target_instr.is_unconditional_branch()
                || (prev_target_instr.opcode().is_jump() && !prev_target_instr.opcode().does_link())
            {
                // Since we took the branch on the previous `look_ahead` call then we don't have
                // anything else to process here.
                return Ok(());
            }

            result.process_prev_func_call(&mut regs_tracker, prev_target_instr);

            target_local_offset += 4;
        }

        Ok(())
    }
}
