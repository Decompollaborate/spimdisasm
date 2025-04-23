/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{sync::Arc, vec::Vec};
use core::mem;
use rabbitizer::{access_type::AccessType, Instruction};

use crate::{
    addresses::{GlobalOffsetTable, Rom, RomVramRange, Size, Vram},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
};

use super::{
    InstrAnalysisInfo, InstrOpLink, InstrOpPairedAddress, InstrOpRegisterOperation,
    InstrOpTailCall, InstructionOperation, RegisterTracker,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub(crate) struct InstructionAnalysisResult {
    ranges: RomVramRange,

    instruction_infos: Arc<[InstrAnalysisInfo]>,

    /// Every referenced vram found.
    referenced_vrams: UnorderedSet<Vram>,

    type_info_per_address: UnorderedMap<Vram, GatheredTypeInfo>,

    handwritten_instrs: UnorderedSet<Rom>,
}

impl InstructionAnalysisResult {
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
    pub(crate) fn type_info_per_address(&self) -> &UnorderedMap<Vram, GatheredTypeInfo> {
        &self.type_info_per_address
    }

    #[must_use]
    pub(crate) fn handwritten_instrs(&self) -> &UnorderedSet<Rom> {
        &self.handwritten_instrs
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstructionAnalysisBuilder {
    ranges: RomVramRange,

    instruction_infos: Vec<InstrAnalysisInfo>,

    /// Every referenced vram found.
    referenced_vrams: UnorderedSet<Vram>,

    type_info_per_address: UnorderedMap<Vram, GatheredTypeInfo>,

    handwritten_instrs: UnorderedSet<Rom>,
}

impl InstructionAnalysisBuilder {
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

    pub(crate) fn finish(self) -> InstructionAnalysisResult {
        let InstructionAnalysisBuilder {
            ranges,
            mut instruction_infos,
            mut referenced_vrams,
            type_info_per_address: type_info_per_address_old,
            handwritten_instrs,
        } = self;

        let type_info_per_address = {
            let mut type_info_per_address = UnorderedMap::new();
            let mut mips1_doublefloats = UnorderedMap::new();

            // Fixup access types.
            for (vram, info) in type_info_per_address_old {
                if let GatheredTypeInfo::AccessInfoCounter(counter) = info {
                    if counter.contains_key(&(AccessType::DOUBLEFLOAT, true)) {
                        // MIPS1 doublefloat

                        let actual_address = vram.align_down(8);
                        // let addended_address = actual_address + Size::new(0x4);
                        mips1_doublefloats.insert(actual_address, counter);
                    } else {
                        type_info_per_address
                            .insert(vram, GatheredTypeInfo::AccessInfoCounter(counter));
                    }
                } else {
                    type_info_per_address.insert(vram, info);
                }
            }

            for (actual_address, mut counter) in mips1_doublefloats {
                let addended_address = actual_address + Size::new(0x4);
                referenced_vrams.remove(&addended_address);
                if let Some(GatheredTypeInfo::AccessInfoCounter(other_counter)) =
                    type_info_per_address.remove(&addended_address)
                {
                    // Tell the other half of the doublefloat access that it shuold use an unaddended address.
                    for instr_rom in other_counter.into_iter().flat_map(|(_, rom_list)| rom_list) {
                        let index = (instr_rom - ranges.rom().start()).inner() as usize / 4;

                        let value = instruction_infos[index].clone().align_down_unaddended(8);
                        if let Some(upper_rom) = value.upper_rom() {
                            let upper_index =
                                (upper_rom - ranges.rom().start()).inner() as usize / 4;

                            // Update the upper half too.
                            let upper_value = instruction_infos[upper_index]
                                .clone()
                                .align_down_unaddended(8);
                            instruction_infos[upper_index] = upper_value;
                        }
                        instruction_infos[index] = value;
                    }
                }

                // The type inferrer only works if we have a single access type, so we remove the
                // `FLOAT` one. Keep the rest in case this points to an struct or something else.
                counter.remove(&(AccessType::FLOAT, false));
                type_info_per_address
                    .insert(actual_address, GatheredTypeInfo::AccessInfoCounter(counter));
            }

            type_info_per_address
        };

        InstructionAnalysisResult {
            ranges,
            instruction_infos: instruction_infos.into(),
            referenced_vrams,
            type_info_per_address,
            handwritten_instrs,
        }
    }
}

impl InstructionAnalysisBuilder {
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
                    self.apply_symbol_type(target_vram, TypeInfo::Function, instr_rom);
                    InstrAnalysisInfo::DirectLink { target_vram }
                }
                InstrOpLink::LinkingBranch { target_vram } => {
                    self.add_referenced_vram(target_vram);
                    self.apply_symbol_type(target_vram, TypeInfo::Function, instr_rom);
                    InstrAnalysisInfo::BranchLink { target_vram }
                }
                InstrOpLink::RawRegisterLink { vram, .. } => {
                    self.apply_symbol_type(vram, TypeInfo::Function, instr_rom);
                    InstrAnalysisInfo::JumpAndLinkRegisterRaw { raw_vram: vram }
                }
                InstrOpLink::Call16RegisterLink { vram, rom } => {
                    self.apply_symbol_type(vram, TypeInfo::Function, instr_rom);
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
                    got_entry,
                } => {
                    self.apply_symbol_type(vram, TypeInfo::Function, instr_rom);
                    self.set_info(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::GotCallHi {
                            vram,
                            got_entry: got_entry.clone(),
                        },
                    );
                    self.set_info(
                        self.index_from_rom(lo_rom),
                        InstrAnalysisInfo::GotCallLo { vram, got_entry },
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
                    self.apply_symbol_type(target_vram, TypeInfo::Function, instr_rom);
                    InstrAnalysisInfo::MaybeDirectTailCall { target_vram }
                }
                InstrOpTailCall::RawRegisterTailCall { vram, .. } => {
                    self.apply_symbol_type(vram, TypeInfo::Function, instr_rom);
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
            } => {
                self.apply_symbol_type(jumptable_vram, TypeInfo::Jumptable, instr_rom);
                InstrAnalysisInfo::Jumptable {
                    jumptable_vram,
                    kind: info,
                }
            }

            InstructionOperation::ReturnJump => InstrAnalysisInfo::No,

            InstructionOperation::Branch { target_vram } => {
                self.add_referenced_vram(target_vram);
                self.apply_symbol_type(target_vram, TypeInfo::BranchTarget, instr_rom);
                if self.ranges.in_vram_range(target_vram) {
                    InstrAnalysisInfo::Branch { target_vram }
                } else {
                    InstrAnalysisInfo::BranchOutside { target_vram }
                }
            }

            InstructionOperation::Hi { value, .. } => InstrAnalysisInfo::UnpairedHi { value },

            InstructionOperation::PairedAddress {
                addended_vram,
                unaddended_vram,
                info,
            } => match info {
                InstrOpPairedAddress::PairedLo {
                    hi_rom,
                    access_info,
                } => {
                    self.add_referenced_vram(unaddended_vram);

                    self.apply_symbol_type(unaddended_vram, access_info.into(), instr_rom);
                    self.set_info_if_empty(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::PairedHi {
                            addended_vram,
                            unaddended_vram,
                        },
                    );
                    InstrAnalysisInfo::PairedLo {
                        addended_vram,
                        unaddended_vram,
                        upper_rom: hi_rom,
                    }
                }
                InstrOpPairedAddress::GpRel { access_info } => {
                    self.add_referenced_vram(unaddended_vram);
                    self.apply_symbol_type(unaddended_vram, access_info.into(), instr_rom);
                    InstrAnalysisInfo::GpRel {
                        addended_vram,
                        unaddended_vram,
                    }
                }
                InstrOpPairedAddress::GpGotLazyResolver {} => {
                    self.apply_symbol_type(addended_vram, TypeInfo::No, instr_rom);
                    InstrAnalysisInfo::GotLazyResolver {
                        addended_vram,
                        unaddended_vram: addended_vram,
                    }
                }
                InstrOpPairedAddress::GpGotGlobal { global_entry } => {
                    self.add_referenced_vram(addended_vram);
                    self.apply_symbol_type(addended_vram, TypeInfo::No, instr_rom);
                    InstrAnalysisInfo::GotGlobal {
                        addended_vram,
                        unaddended_vram: addended_vram,
                        global_entry,
                    }
                }
                InstrOpPairedAddress::GpGotLocal {} => InstrAnalysisInfo::GotLocal {
                    addended_vram,
                    unaddended_vram: addended_vram,
                },
                InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom,
                    access_info,
                } => {
                    self.add_referenced_vram(unaddended_vram);
                    self.apply_symbol_type(unaddended_vram, access_info.into(), instr_rom);
                    self.set_info(
                        self.index_from_rom(upper_rom),
                        InstrAnalysisInfo::GotLocalPaired {
                            addended_vram,
                            unaddended_vram,
                        },
                    );
                    InstrAnalysisInfo::PairedLo {
                        addended_vram,
                        unaddended_vram,
                        upper_rom,
                    }
                }
                InstrOpPairedAddress::PairedGotLo { hi_rom, got_entry } => {
                    self.add_referenced_vram(addended_vram);
                    self.apply_symbol_type(addended_vram, TypeInfo::No, instr_rom);
                    self.set_info_if_empty(
                        self.index_from_rom(hi_rom),
                        InstrAnalysisInfo::PairedGotHi {
                            vram: addended_vram,
                            got_entry: got_entry.clone(),
                        },
                    );
                    InstrAnalysisInfo::PairedGotLo {
                        vram: addended_vram,
                        got_entry,
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
                self.apply_symbol_type(original_address, access_info.into(), instr_rom);
                InstrAnalysisInfo::No
            }
            InstructionOperation::DanglingLo { .. } => InstrAnalysisInfo::No,
            InstructionOperation::Constant { constant, hi_rom } => {
                self.set_info_if_empty(
                    self.index_from_rom(hi_rom),
                    InstrAnalysisInfo::ConstantHi { constant },
                );
                InstrAnalysisInfo::ConstantLo {
                    constant,
                    upper_rom: hi_rom,
                }
            }
            InstructionOperation::UnpairedConstant { .. } => InstrAnalysisInfo::No,
            InstructionOperation::RegisterOperation { info } => match info {
                InstrOpRegisterOperation::SuspectedCpload { hi_rom, lo_rom } => {
                    self.set_info(self.index_from_rom(hi_rom), InstrAnalysisInfo::CploadHi);
                    self.set_info(self.index_from_rom(lo_rom), InstrAnalysisInfo::CploadLo);
                    InstrAnalysisInfo::CploadAddu
                }
                InstrOpRegisterOperation::RegisterAddition { .. }
                | InstrOpRegisterOperation::RegisterSubtraction { .. }
                | InstrOpRegisterOperation::Or { .. } => InstrAnalysisInfo::No,
            },

            InstructionOperation::UnhandledOpcode { .. }
            | InstructionOperation::InvalidInstr {} => InstrAnalysisInfo::No,
        };

        self.set_info(instr_index, info.clone());

        info
    }
}

impl InstructionAnalysisBuilder {
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

    fn apply_symbol_type(&mut self, address: Vram, type_info: TypeInfo, instr_rom: Rom) {
        self.type_info_per_address
            .entry(address)
            .or_default()
            .insert(type_info, instr_rom);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum GatheredTypeInfo {
    #[default]
    No,
    Function,
    Jumptable,
    BranchTarget,
    AccessInfoCounter(UnorderedMap<(AccessType, bool), UnorderedSet<Rom>>),
}

impl GatheredTypeInfo {
    fn insert(&mut self, type_info: TypeInfo, instr_rom: Rom) {
        let myself = mem::take(self);

        *self = match (myself, type_info) {
            (_, TypeInfo::Function) => Self::Function,
            (_, TypeInfo::Jumptable) => Self::Jumptable,
            (GatheredTypeInfo::No, TypeInfo::BranchTarget) => Self::BranchTarget,

            (Self::Function, _) => Self::Function,
            (Self::Jumptable, _) => Self::Jumptable,
            (
                GatheredTypeInfo::BranchTarget,
                TypeInfo::No | TypeInfo::BranchTarget | TypeInfo::AccessInfo(_),
            ) => GatheredTypeInfo::BranchTarget,
            (GatheredTypeInfo::AccessInfoCounter(_), TypeInfo::BranchTarget) => {
                GatheredTypeInfo::BranchTarget
            }

            (Self::No, TypeInfo::No) => Self::No,
            (Self::AccessInfoCounter(unordered_map), TypeInfo::No) => {
                Self::AccessInfoCounter(unordered_map)
            }

            (Self::No, TypeInfo::AccessInfo(access_info)) => {
                let mut counter = UnorderedMap::new();
                let mut aux = UnorderedSet::new();
                aux.insert(instr_rom);
                counter.insert(access_info, aux);
                Self::AccessInfoCounter(counter)
            }

            (Self::AccessInfoCounter(mut counter), TypeInfo::AccessInfo(access_info)) => {
                counter
                    .entry(access_info)
                    .and_modify(|v| {
                        v.insert(instr_rom);
                    })
                    .or_insert({
                        let mut aux = UnorderedSet::new();
                        aux.insert(instr_rom);
                        aux
                    });
                Self::AccessInfoCounter(counter)
            }
        };
    }

    #[expect(dead_code)]
    pub(crate) fn get_access_info(&self) -> Option<(AccessType, bool)> {
        if let Self::AccessInfoCounter(counter) = self {
            if counter.len() == 1 {
                counter.iter().next().map(|(k, _v)| *k)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum TypeInfo {
    No,
    Function,
    Jumptable,
    BranchTarget,
    AccessInfo((AccessType, bool)),
}

impl From<Option<(AccessType, bool)>> for TypeInfo {
    fn from(value: Option<(AccessType, bool)>) -> Self {
        match value {
            Some(x) => Self::AccessInfo(x),
            None => Self::No,
        }
    }
}

impl From<(AccessType, bool)> for TypeInfo {
    fn from(value: (AccessType, bool)) -> Self {
        Self::AccessInfo(value)
    }
}
