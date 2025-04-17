/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use rabbitizer::{access_type::AccessType, Instruction};

use crate::{
    addresses::{GlobalOffsetTable, Rom, RomVramRange, Vram},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
};

use super::{
    InstrAnalysisInfo, InstrOpLink, InstrOpPairedAddress, InstrOpRegisterOperation,
    InstrOpTailCall, InstructionOperation, RegisterTracker,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstructionAnalysisResult {
    ranges: RomVramRange,

    instruction_infos: Vec<InstrAnalysisInfo>,

    /// Every referenced vram found.
    referenced_vrams: UnorderedSet<Vram>,

    type_info_per_address: UnorderedMap<Vram, UnorderedMap<(AccessType, bool), u32>>,

    handwritten_instrs: UnorderedSet<Rom>,
}

impl InstructionAnalysisResult {
    #[must_use]
    pub(crate) fn new(ranges: RomVramRange) -> Self {
        let instr_count = ranges.rom().size().inner() / 4;
        let instruction_infos = vec![InstrAnalysisInfo::No; instr_count as usize];

        Self {
            ranges,
            instruction_infos,
            referenced_vrams: UnorderedSet::new(),
            type_info_per_address: UnorderedMap::new(),
            handwritten_instrs: UnorderedSet::new(),
        }
    }

    #[must_use]
    pub(crate) fn ranges(&self) -> &RomVramRange {
        &self.ranges
    }

    #[must_use]
    pub(crate) fn instruction_infos(&self) -> &[InstrAnalysisInfo] {
        &self.instruction_infos
    }

    #[must_use]
    pub(crate) fn referenced_vrams(&self) -> &UnorderedSet<Vram> {
        &self.referenced_vrams
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
}

impl InstructionAnalysisResult {
    pub(crate) fn process_instr(
        &mut self,
        regs_tracker: &mut RegisterTracker,
        instr: &Instruction,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> InstrAnalysisInfo {
        let instr_rom = self.rom_from_instr(instr);
        let instr_index = self.index_from_rom(instr_rom);

        if instr.is_likely_handwritten() {
            self.handwritten_instrs.insert(instr_rom);
        }

        let instr_processed_result =
            regs_tracker.process_instruction(instr, instr_rom, global_offset_table);

        let info = match instr_processed_result {
            InstructionOperation::Link { info } => match info {
                InstrOpLink::DirectLinkingCall { target_vram } => {
                    self.add_referenced_vram(target_vram);
                    InstrAnalysisInfo::DirectLink { target_vram }
                }
                InstrOpLink::LinkingBranch { target_vram } => {
                    self.add_referenced_vram(target_vram);
                    InstrAnalysisInfo::BranchLink { target_vram }
                }
                InstrOpLink::RawRegisterLink { vram, .. } => {
                    InstrAnalysisInfo::JumpAndLinkRegisterRaw { raw_vram: vram }
                }
                InstrOpLink::Call16RegisterLink { vram, rom } => {
                    self.set_info(
                        self.index_from_rom(rom),
                        InstrAnalysisInfo::GotCall16 { vram },
                    );
                    InstrAnalysisInfo::JumpAndLinkRegisterRaw { raw_vram: vram }
                }
                InstrOpLink::CallHiLoRegisterLink {
                    vram,
                    hi_rom,
                    lo_rom,
                } => {
                    self.set_info(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::GotCallHi { vram },
                    );
                    self.set_info(
                        self.index_from_rom(lo_rom),
                        InstrAnalysisInfo::GotCallLo { vram },
                    );
                    InstrAnalysisInfo::JumpAndLinkRegisterRaw { raw_vram: vram }
                }
                InstrOpLink::DereferencedRegisterLink {
                    dereferenced_vram, ..
                } => InstrAnalysisInfo::JumpAndLinkRegisterDereferenced { dereferenced_vram },
                InstrOpLink::UnknownJumpAndLinkRegister { .. } => InstrAnalysisInfo::No,
            },

            InstructionOperation::TailCall { info } => match info {
                InstrOpTailCall::MaybeDirectTailCall { target_vram } => {
                    self.add_referenced_vram(target_vram);
                    InstrAnalysisInfo::MaybeDirectTailCall { target_vram }
                }
                InstrOpTailCall::RawRegisterTailCall { vram, .. } => {
                    InstrAnalysisInfo::RawRegisterTailCall { raw_vram: vram }
                }
                InstrOpTailCall::DereferencedRegisterTailCall {
                    dereferenced_vram, ..
                } => InstrAnalysisInfo::DereferencedRegisterTailCall { dereferenced_vram },
                InstrOpTailCall::UnknownRegisterJump { .. } => InstrAnalysisInfo::No,
            },

            InstructionOperation::JumptableJump {
                jumptable_vram,
                info,
                ..
            } => InstrAnalysisInfo::Jumptable {
                jumptable_vram,
                kind: info,
            },

            InstructionOperation::ReturnJump => InstrAnalysisInfo::No,

            InstructionOperation::Branch { target_vram } => {
                self.add_referenced_vram(target_vram);
                if self.ranges.in_vram_range(target_vram) {
                    InstrAnalysisInfo::Branch { target_vram }
                } else {
                    InstrAnalysisInfo::BranchOutside { target_vram }
                }
            }

            InstructionOperation::Hi { value, .. } => InstrAnalysisInfo::UnpairedHi { value },

            InstructionOperation::PairedAddress { vram, info } => match info {
                InstrOpPairedAddress::PairedLo {
                    hi_rom,
                    access_info,
                    ..
                } => {
                    self.add_referenced_vram(vram);

                    if let Some(access_info) = access_info {
                        self.apply_symbol_type(vram, access_info);
                    }
                    self.set_info_if_empty(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::PairedHi { vram },
                    );
                    InstrAnalysisInfo::PairedLo { vram }
                }
                InstrOpPairedAddress::GpRel { access_info, .. } => {
                    self.add_referenced_vram(vram);
                    if let Some(access_info) = access_info {
                        self.apply_symbol_type(vram, access_info);
                    }
                    InstrAnalysisInfo::GpRel { vram }
                }
                InstrOpPairedAddress::GpGotLazyResolver {} => {
                    InstrAnalysisInfo::GotLazyResolver { vram }
                }
                InstrOpPairedAddress::GpGotGlobal {} => {
                    self.add_referenced_vram(vram);
                    InstrAnalysisInfo::GotGlobal { vram }
                }
                InstrOpPairedAddress::GpGotLocal { .. } => InstrAnalysisInfo::GotLocal { vram },
                InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom,
                    access_info,
                    ..
                } => {
                    self.add_referenced_vram(vram);
                    if let Some(access_info) = access_info {
                        self.apply_symbol_type(vram, access_info);
                    }
                    self.set_info(
                        self.index_from_rom(upper_rom),
                        InstrAnalysisInfo::GotLocalPaired { vram },
                    );
                    InstrAnalysisInfo::PairedLo { vram }
                }
                InstrOpPairedAddress::PairedGotLo { hi_rom } => {
                    self.add_referenced_vram(vram);
                    self.set_info_if_empty(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::PairedGotHi { vram },
                    );
                    InstrAnalysisInfo::PairedGotLo { vram }
                }
                InstrOpPairedAddress::PairedLoUnaligned {
                    hi_rom,
                    access_info,
                    unaddended_address,
                } => {
                    self.add_referenced_vram(unaddended_address);

                    self.apply_symbol_type(unaddended_address, access_info);
                    self.set_info_if_empty(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::PairedHiUnaligned {
                            unaddended_vram: unaddended_address,
                            addended_vram: vram,
                        },
                    );
                    InstrAnalysisInfo::PairedLoUnaligned {
                        unaddended_vram: unaddended_address,
                        addended_vram: vram,
                    }
                }
                InstrOpPairedAddress::GpRelUnaligned {
                    access_info,
                    unaddended_address,
                } => {
                    self.add_referenced_vram(unaddended_address);

                    self.apply_symbol_type(unaddended_address, access_info);
                    InstrAnalysisInfo::GpRelUnaligned {
                        unaddended_vram: unaddended_address,
                        addended_vram: vram,
                    }
                }
            },

            InstructionOperation::GpSet { hi_rom } => {
                self.set_info_if_empty(self.index_from_rom(hi_rom), InstrAnalysisInfo::GpSetHi);
                InstrAnalysisInfo::GpSetLo
            }
            InstructionOperation::DereferencedRawAddress {
                original_address,
                access_info,
                ..
            } => {
                self.apply_symbol_type(original_address, access_info);
                InstrAnalysisInfo::No
            }
            InstructionOperation::DanglingLo { .. } => InstrAnalysisInfo::No,
            InstructionOperation::Constant { constant, hi_rom } => {
                self.set_info_if_empty(
                    self.index_from_rom(hi_rom),
                    InstrAnalysisInfo::ConstantHi { constant },
                );
                InstrAnalysisInfo::ConstantLo { constant }
            }
            InstructionOperation::UnpairedConstant { .. } => InstrAnalysisInfo::No,
            InstructionOperation::RegisterOperation { info } => match info {
                InstrOpRegisterOperation::SuspectedCpload { hi_rom, lo_rom, .. } => {
                    self.set_info(self.index_from_rom(hi_rom), InstrAnalysisInfo::CploadHi);
                    self.set_info(self.index_from_rom(lo_rom), InstrAnalysisInfo::CploadLo);
                    InstrAnalysisInfo::CploadAddu
                }
                InstrOpRegisterOperation::RegisterAddition { .. }
                | InstrOpRegisterOperation::RegisterSubtraction { .. }
                | InstrOpRegisterOperation::Or { .. } => InstrAnalysisInfo::No,
            },

            InstructionOperation::UnhandledOpcode { .. }
            | InstructionOperation::InvalidInstr { .. } => InstrAnalysisInfo::No,
        };

        self.set_info(instr_index, info);

        info
    }
}

impl InstructionAnalysisResult {
    fn apply_symbol_type(&mut self, address: Vram, access_info: (AccessType, bool)) {
        let (access_type, unsigned_memory_address) = access_info;
        self.type_info_per_address
            .entry(address)
            .or_default()
            .entry((access_type, unsigned_memory_address))
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }
}

impl InstructionAnalysisResult {
    fn rom_from_instr(&self, instr: &Instruction) -> Rom {
        self.ranges
            .rom_from_vram(instr.vram())
            .expect("This should not panic")
    }
    fn index_from_rom(&self, rom: Rom) -> usize {
        (rom - self.ranges.rom().start()).inner() as usize / 4
    }

    fn set_info_if_empty(&mut self, index: usize, info: InstrAnalysisInfo) {
        if !matches!(info, InstrAnalysisInfo::No) {
            let val = &mut self.instruction_infos[index];
            if matches!(
                val,
                InstrAnalysisInfo::No | InstrAnalysisInfo::UnpairedHi { .. }
            ) {
                *val = info;
            }
        }
    }
    fn set_info(&mut self, index: usize, info: InstrAnalysisInfo) {
        // Only set new info for the instruction if it isn't `No`.
        // This allows us to avoid overriding information we already have from
        // previous runs, usually due to funny control flow.
        if !matches!(info, InstrAnalysisInfo::No) {
            let val = &mut self.instruction_infos[index];

            if matches!(
                info,
                InstrAnalysisInfo::UnpairedHi { .. } | InstrAnalysisInfo::GotGlobal { .. }
            ) {
                // Only set `UnpairedHi` or `GotGlobal` if we have no info about
                // this instruction.
                // This way we can avoid unpairing stuff that we already paired.
                if matches!(val, InstrAnalysisInfo::No) {
                    *val = info;
                }
            } else {
                *val = info;
            }
        }
    }

    fn add_referenced_vram(&mut self, referenced_vram: Vram) {
        self.referenced_vrams.insert(referenced_vram);
    }
}
