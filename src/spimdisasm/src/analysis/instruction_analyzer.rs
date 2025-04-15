/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Instruction;

use crate::{
    addresses::{GlobalOffsetTable, RomVramRange, Size, Vram},
    collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet},
    context::{Context, OwnedSegmentNotFoundError},
    parent_segment_info::ParentSegmentInfo,
};

use super::{InstrAnalysisInfo, InstructionAnalysisResult, RegisterTracker};

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
        let mut regs_tracker = RegisterTracker::new(
            instrs[0].abi(),
            Some(ranges.vram().start()),
            context.global_config().gp_config().copied(),
        );
        let mut result = InstructionAnalysisResult::new(ranges);
        let global_offset_table = context
            .find_owned_segment(parent_info)?
            .global_offset_table();

        // The below iteration skips the first instruction so we have to process it explicitly here.
        let mut prev_instr_analysis_info =
            Some(result.process_instr(&mut regs_tracker, &instrs[0], global_offset_table));
        regs_tracker.clear_afterwards(None, None);

        // TODO: maybe implement a way to know which instructions have been processed?

        for (i, w) in instrs.windows(2).enumerate() {
            let prev_instr = w[0];
            let instr = w[1];
            let local_offset = (i + 1) * 4;
            let current_vram = instr.vram();

            if !instr.is_valid() {
                regs_tracker.clear_afterwards(Some(&prev_instr), Some(current_vram + Size::new(4)));
                prev_instr_analysis_info = None;
                continue;
            }

            /*
            if instr.isLikelyHandwritten() and not self.isRsp:
                self.isLikelyHandwritten = True
                self.endOfLineComment[instructionOffset//4] = " /* handwritten instruction */
"
            */

            let info = if !prev_instr.opcode().is_branch_likely() {
                Some(result.process_instr(&mut regs_tracker, &instr, global_offset_table))
            } else {
                None
            };

            if let Some(InstrAnalysisInfo::JumptableJump { jumptable_vram }) =
                prev_instr_analysis_info
            {
                analyzer.follow_jumptable(
                    context,
                    parent_info,
                    &mut result,
                    &regs_tracker,
                    instrs,
                    &prev_instr,
                    false,
                    global_offset_table,
                    jumptable_vram,
                )?;
            } else {
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
                    global_offset_table,
                )?;
            }

            regs_tracker.clear_afterwards(Some(&prev_instr), Some(current_vram + Size::new(4)));
            prev_instr_analysis_info = info;
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
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> Result<(), OwnedSegmentNotFoundError> {
        if prev_instr.opcode().does_link() {
            return Ok(());
        }
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

        if prev_is_likely {
            result.process_instr(&mut regs_tracker, instr, global_offset_table);
        }

        self.look_ahead_impl(
            context,
            parent_info,
            result,
            regs_tracker,
            instrs,
            target_local_offset,
            prev_is_likely,
            global_offset_table,
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
        global_offset_table: Option<&GlobalOffsetTable>,
        jumptable_vram: Vram,
    ) -> Result<(), OwnedSegmentNotFoundError> {
        if !self
            .branches_taken
            .insert((prev_instr.vram(), prev_is_likely))
        {
            // If we already processed this branch then don't do it again.
            return Ok(());
        }

        let jumptable_ref = if let Some(jumptable_ref) = context
            .find_owned_segment(parent_info)?
            .find_reference(jumptable_vram, FindSettings::new(false))
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
                    global_offset_table,
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
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> Result<(), OwnedSegmentNotFoundError> {
        let mut prev_instr_analysis_info = None;

        while target_local_offset / 4 < instrs.len() {
            let prev_target_instr = &instrs[target_local_offset / 4 - 1];
            let target_instr = &instrs[target_local_offset / 4];

            let info = if !prev_target_instr.opcode().is_branch_likely() {
                Some(result.process_instr(&mut regs_tracker, target_instr, global_offset_table))
            } else {
                None
            };
            if let Some(InstrAnalysisInfo::JumptableJump { jumptable_vram }) =
                prev_instr_analysis_info
            {
                self.follow_jumptable(
                    context,
                    parent_info,
                    result,
                    &regs_tracker,
                    instrs,
                    prev_target_instr,
                    prev_is_likely,
                    global_offset_table,
                    jumptable_vram,
                )?;
            } else {
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
                    global_offset_table,
                )?;
            }

            let current_vram = target_instr.vram();
            if regs_tracker
                .clear_afterwards(Some(prev_target_instr), Some(current_vram + Size::new(4)))
            {
                // Since we took the branch on the previous `look_ahead` call then we don't have
                // anything else to process here.
                return Ok(());
            }

            prev_instr_analysis_info = info;
            target_local_offset += 4;
        }

        Ok(())
    }
}
