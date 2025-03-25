/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;
use rabbitizer::{registers_meta::Register, Instruction};

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::{GpSetInfo, InstructionAnalysisResult},
    collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet},
    config::GlobalConfig,
    context::Context,
    metadata::{GotAccessKind, ReferrerInfo, SymbolMetadata, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    relocation::{RelocReferencedSym, RelocationInfo, RelocationType},
    section_type::SectionType,
    symbols::{
        display::{
            FunctionDisplay, FunctionDisplaySettings, InternalSymDisplSettings, SymDisplayError,
        },
        InvalidRelocForSectionError, RomSymbol, RomSymbolProcessed, Symbol, SymbolPostProcessError,
        SymbolProcessed, UnalignedUserRelocError,
    },
};

const SECTION_TYPE: SectionType = SectionType::Text;

#[derive(Debug, Clone)]
pub struct FunctionSymProcessed {
    ranges: RomVramRange,
    instructions: Arc<[Instruction]>,
    parent_segment_info: ParentSegmentInfo,
    instr_analysis: InstructionAnalysisResult,

    relocs: Arc<[Option<RelocationInfo>]>,
    labels: Arc<[Vram]>,
}

impl FunctionSymProcessed {
    pub(crate) fn new(
        context: &mut Context,
        ranges: RomVramRange,
        instructions: Arc<[Instruction]>,
        parent_segment_info: ParentSegmentInfo,
        instr_analysis: InstructionAnalysisResult,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<Self, SymbolPostProcessError> {
        let labels = Self::find_and_update_labels(context, &ranges, &parent_segment_info)?;
        let mut relocs = Self::generate_relocs(
            context,
            &ranges,
            &instructions,
            &parent_segment_info,
            &instr_analysis,
        )?;

        if !relocs.is_empty() {
            for (reloc_rom, reloc_info) in user_relocs.range(*ranges.rom()) {
                if reloc_rom.inner() % 4 != 0 {
                    return Err(SymbolPostProcessError::UnalignedUserReloc(
                        UnalignedUserRelocError::new(*reloc_rom, reloc_info.reloc_type()),
                    ));
                }

                if !reloc_info.reloc_type().valid_for_function() {
                    return Err(SymbolPostProcessError::InvalidRelocForSection(
                        InvalidRelocForSectionError::new(
                            *reloc_rom,
                            reloc_info.reloc_type(),
                            SECTION_TYPE,
                        ),
                    ));
                }

                let reloc_index = (*reloc_rom - ranges.rom().start()).inner() as usize / 4;
                assert!(reloc_index < relocs.len());
                relocs[reloc_index] = if reloc_info.reloc_type().is_none() {
                    None
                } else {
                    Some(reloc_info.clone())
                };
            }
        }

        Ok(Self {
            ranges,
            instructions,
            parent_segment_info,
            instr_analysis,
            relocs: relocs.into(),
            labels: labels.into(),
        })
    }

    fn generate_relocs(
        context: &mut Context,
        ranges: &RomVramRange,
        instructions: &[Instruction],
        parent_segment_info: &ParentSegmentInfo,
        instr_analysis: &InstructionAnalysisResult,
    ) -> Result<Vec<Option<RelocationInfo>>, SymbolPostProcessError> {
        let mut relocs = vec![None; instructions.len()];

        let self_vram = ranges.vram().start();

        let mut referenced_labels_owned_segment = Vec::new();
        let mut referenced_labels_refer_segment = Vec::new();

        let owned_segment = context.find_owned_segment(parent_segment_info)?;

        for (instr_rom, target_vram) in instr_analysis.branch_targets() {
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(instrOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if let Some(branch_sym) = owned_segment.find_label(*target_vram) {
                debug_assert!(branch_sym.vram() == *target_vram);
                let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
                relocs[instr_index as usize] = Some(
                    RelocationType::R_MIPS_PC16
                        .new_reloc_info(RelocReferencedSym::Label(*target_vram)),
                );

                referenced_labels_owned_segment.push((
                    *target_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), *instr_rom),
                ));

                // TODO: keep here?
                // contextSym.branchLabels.add(labelSym.vram, labelSym)
            } else {
                // TODO: Add a comment with a warning saying that the target wasn't found
            }
        }
        for (instr_rom, target_vram) in instr_analysis.branch_targets_outside() {
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(instrOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if let Some(branch_sym) = owned_segment
                .find_symbol(*target_vram, FindSettings::new(false))
                .filter(|x| x.sym_type() == Some(SymbolType::Function))
            {
                debug_assert!(branch_sym.vram() == *target_vram);
                let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
                relocs[instr_index as usize] = Some(
                    RelocationType::R_MIPS_PC16
                        .new_reloc_info(RelocReferencedSym::Address(*target_vram)),
                );

                // TODO: keep here?
                // contextSym.branchLabels.add(labelSym.vram, labelSym)
            } else if owned_segment.find_label(*target_vram).is_some() {
                let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
                relocs[instr_index as usize] = Some(
                    RelocationType::R_MIPS_PC16
                        .new_reloc_info(RelocReferencedSym::Label(*target_vram)),
                );

                referenced_labels_owned_segment.push((
                    *target_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), *instr_rom),
                ));
            } else {
                // TODO: Add a comment with a warning saying that the target wasn't found
            }
            // TODO: add some kind of comment mentioning this instr is branching outside the current function.
        }

        for (instr_rom, target_vram) in instr_analysis.func_calls() {
            if owned_segment.is_vram_ignored(*target_vram) {
                continue;
            }

            /*
            if context.isAddressBanned(targetVram):
                continue
            */

            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(instrOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;

            let referenced_sym = if context
                .find_symbol_from_any_segment(
                    *target_vram,
                    parent_segment_info,
                    FindSettings::new(false),
                    |x| x.is_defined() && x.sym_type().is_some_and(|y| y.valid_call_target()),
                )
                .is_some()
            {
                RelocReferencedSym::Address(*target_vram)
            } else if context
                .find_label_from_any_segment(*target_vram, parent_segment_info, |_| true)
                .is_some()
            {
                referenced_labels_refer_segment.push((
                    *target_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), *instr_rom),
                ));

                RelocReferencedSym::Label(*target_vram)
            } else {
                // whatever
                RelocReferencedSym::Address(*target_vram)
            };

            relocs[instr_index as usize] =
                Some(RelocationType::R_MIPS_26.new_reloc_info(referenced_sym));
        }

        for (instr_rom, symbol_vram) in instr_analysis.address_per_lo_instr() {
            let instr_index = (*instr_rom - ranges.rom().start()).inner() as usize / 4;

            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(loOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if owned_segment.is_vram_ignored(*symbol_vram) {
                continue;
            }

            let metadata = context.find_symbol_from_any_segment(
                *symbol_vram,
                parent_segment_info,
                FindSettings::new(true),
                |x| x.sym_type() != Some(SymbolType::Function) || x.vram() == *symbol_vram,
            );

            let reloc_type = Self::reloc_for_instruction(
                context.global_config(),
                &instructions[instr_index],
                metadata,
                instr_analysis,
                *instr_rom,
            );

            let referenced_sym = if metadata.is_some() {
                RelocReferencedSym::Address(*symbol_vram)
            } else {
                referenced_labels_refer_segment.push((
                    *symbol_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), *instr_rom),
                ));

                RelocReferencedSym::Label(*symbol_vram)
            };

            relocs[instr_index] = Some(reloc_type.new_reloc_info(referenced_sym));
        }

        for (instr_rom, symbol_vram) in instr_analysis.address_per_hi_instr() {
            let instr_index = (*instr_rom - ranges.rom().start()).inner() as usize / 4;

            if owned_segment.is_vram_ignored(*symbol_vram) {
                if let Some(imm) = instructions[instr_index].get_processed_immediate() {
                    relocs[instr_index] =
                        Some(RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                            RelocReferencedSym::SymName(
                                Arc::from(format!("0x{:08X}", imm << 16)),
                                0,
                            ),
                        ));
                }
                continue;
            }

            let metadata = context.find_symbol_from_any_segment(
                *symbol_vram,
                parent_segment_info,
                FindSettings::new(true),
                |x| x.sym_type() != Some(SymbolType::Function) || x.vram() == *symbol_vram,
            );

            let reloc_type = Self::reloc_for_instruction(
                context.global_config(),
                &instructions[instr_index],
                metadata,
                instr_analysis,
                *instr_rom,
            );

            let referenced_sym = if metadata.is_some() {
                RelocReferencedSym::Address(*symbol_vram)
            } else {
                referenced_labels_refer_segment.push((
                    *symbol_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), *instr_rom),
                ));

                RelocReferencedSym::Label(*symbol_vram)
            };

            relocs[instr_index] = Some(reloc_type.new_reloc_info(referenced_sym));
        }

        for (instr_rom, symbol_got_vram) in instr_analysis.calculated_got_addresses() {
            let instr_index = (*instr_rom - ranges.rom().start()).inner() as usize / 4;

            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(loOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if owned_segment.is_vram_ignored(*symbol_got_vram) {
                continue;
            }

            let metadata = context.find_symbol_from_any_segment(
                *symbol_got_vram,
                parent_segment_info,
                FindSettings::new(true),
                |x| x.sym_type() != Some(SymbolType::Function) || x.vram() == *symbol_got_vram,
            );

            let reloc_type = Self::reloc_for_instruction(
                context.global_config(),
                &instructions[instr_index],
                metadata,
                instr_analysis,
                *instr_rom,
            );

            // TODO: is this necessary??
            /*
            let referenced_sym = if metadata.is_some() {
                RelocReferencedSym::Address(*symbol_got_vram)
            } else {
                referenced_labels_refer_segment.push((
                    *symbol_got_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), *instr_rom),
                ));

                RelocReferencedSym::Label(*symbol_got_vram)
            };
            */
            let referenced_sym = RelocReferencedSym::Address(*symbol_got_vram);

            relocs[instr_index] = Some(reloc_type.new_reloc_info(referenced_sym));
        }

        /*
        for instrOffset, address in instrAnalyzer.symbolInstrOffset.items():
            if context.isAddressBanned(address):
                continue

            contextSym = getSymbol(address)

            gotHiLo = False
            gotSmall = False
            if contextSym is None and address < 0 and common.GlobalConfig.PIC and common.GlobalConfig.GP_VALUE is not None:
                # Negative pointer may mean it is a weird GOT access
                gotAccess = common.GlobalConfig.GP_VALUE + address
                gpAccess = context.gpAccesses.requestAddress(gotAccess)
                if gpAccess is not None:
                    address = gpAccess.address
                    contextSym = getSymbol(address)
                    gotHiLo = True
                    gotSmall = gpAccess.isSmallSection
                else:
                    common.Utils.eprint(4, f"0x{instructions[instrOffset//4].vram:08X}", f"0x{gotAccess:08X}", instructions[instrOffset//4].disassemble())
                    pass

            if contextSym is None:
                continue

            if contextSym.isGotGlobal:
                if instrOffset not in instrAnalyzer.gotAccessAddresses and not gotHiLo:
                    continue

            instr = instructions[instrOffset//4]

            relocType = _getRelocTypeForInstruction(instr, instrOffset, contextSym, gotHiLo, gotSmall)
            if relocType == common.RelocType.MIPS_GPREL16:
                contextSym.accessedAsGpRel = True
            relocs[instrOffset] = common.RelocationInfo(relocType, contextSym, address - contextSym.vram)
        */

        /*
        for instrOffset in instrAnalyzer.cploadOffsets:
            # .cpload directive is meant to use the `_gp_disp` pseudo-symbol
            instr = instructions[instrOffset//4]

            relocType = _getRelocTypeForInstruction(instr, instrOffset)
            relocs[instrOffset] = common.RelocationInfo(relocType, "_gp_disp")
        */

        for gp_set_info in instr_analysis.gp_sets().values() {
            if let GpSetInfo::Address(info) = gp_set_info {
                let hi_index = (info.hi_rom() - ranges.rom().start()).inner() as usize / 4;
                let lo_index = (info.lo_rom() - ranges.rom().start()).inner() as usize / 4;
                let hi_instr = &instructions[hi_index];
                let lo_instr = &instructions[lo_index];

                let hi_reloc_type = Self::reloc_for_instruction(
                    context.global_config(),
                    hi_instr,
                    None,
                    instr_analysis,
                    info.hi_rom(),
                );
                let lo_reloc_type = Self::reloc_for_instruction(
                    context.global_config(),
                    lo_instr,
                    None,
                    instr_analysis,
                    info.lo_rom(),
                );
                if context
                    .global_config()
                    .gp_config()
                    .is_some_and(|x| !x.pic() && x.gp_value() == info.value())
                {
                    relocs[hi_index] = Some(
                        hi_reloc_type
                            .new_reloc_info(RelocReferencedSym::SymName(Arc::from("_gp"), 0)),
                    );
                    relocs[lo_index] = Some(
                        lo_reloc_type
                            .new_reloc_info(RelocReferencedSym::SymName(Arc::from("_gp"), 0)),
                    );
                } else {
                    // TODO: some kind of conversion method for GpValue -> Vram?
                    let address = Vram::new(info.value().inner());
                    if owned_segment.is_vram_ignored(address) {
                        continue;
                    }

                    relocs[hi_index] =
                        Some(hi_reloc_type.new_reloc_info(RelocReferencedSym::Address(address)));
                    relocs[lo_index] =
                        Some(lo_reloc_type.new_reloc_info(RelocReferencedSym::Address(address)));
                }
            }
        }

        for (instr_rom, constant) in instr_analysis.constant_per_instr() {
            let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
            let instr = &instructions[instr_index as usize];
            // TODO: proper reloc inference
            let reloc_type = if instr.opcode().can_be_hi() {
                RelocationType::R_CUSTOM_CONSTANT_HI
            } else {
                RelocationType::R_CUSTOM_CONSTANT_LO
            };

            // TODO: use `:08X`.
            relocs[instr_index as usize] = Some(reloc_type.new_reloc_info(
                RelocReferencedSym::SymName(Arc::from(format!("0x{:X}", constant)), 0),
            ));
        }
        /*
        for instrOffset, constant in instrAnalyzer.constantInstrOffset.items():
            instr = instructions[instrOffset//4]
            relocType = _getRelocTypeForInstruction(instr, instrOffset)

            if relocType in {common.RelocType.MIPS_HI16, common.RelocType.MIPS_LO16}:
                # We can only do this kind of shenanigans for normal %hi/%lo relocs

                symbol = getConstant(constant)
                if symbol is not None:
                    relocs[instrOffset] = common.RelocationInfo(relocType, symbol.getName())
                elif common.GlobalConfig.SYMBOL_FINDER_FILTERED_ADDRESSES_AS_HILO:
                    relocs[instrOffset] = common.RelocationInfo(relocType, f"0x{constant:X}")
                else:
                    # Pretend this pair is a constant
                    loInstr = instr
                    if instr.canBeHi():
                        loInstr = instructions[instrAnalyzer.hiToLowDict[instrOffset] // 4]

                    generatedReloc = _generateHiLoConstantReloc(constant, instr, loInstr)
                    if generatedReloc is not None:
                        relocs[instrOffset] = generatedReloc
            else:
                comment = f"Failed to symbolize address 0x{constant:08X} for {relocType.getPercentRel()}. Make sure this address is within the recognized valid address space."
                if relocType in {common.RelocType.MIPS_GPREL16, common.RelocType.MIPS_GOT16}:
                    if common.GlobalConfig.GP_VALUE is None:
                        comment += f" Please specify a gp_value."
                    elif not context.isInTotalVramRange(common.GlobalConfig.GP_VALUE):
                        comment += f" The provided gp_value (0x{common.GlobalConfig.GP_VALUE:08X}) seems wrong."
                endOfLineComment[instrOffset//4] = f" /* {comment} */
"
        */

        /*
        for instrOffset, targetVram in instrAnalyzer.funcCallInstrOffsets.items():
            funcSym = getSymbol(targetVram, tryPlusOffset=False)
            if funcSym is None:
                continue
            relocs[instrOffset] = common.RelocationInfo(common.RelocType.MIPS_26, funcSym)

        */

        // Handle unpaired `lui`s
        for (instr_rom, (_hi_reg, hi_imm)) in instr_analysis.hi_instrs() {
            if !instr_analysis
                .address_per_hi_instr()
                .contains_key(instr_rom)
                && !instr_analysis.constant_per_instr().contains_key(instr_rom)
            {
                let instr_index = (*instr_rom - ranges.rom().start()).inner() as usize / 4;
                let constant = (*hi_imm as u32) << 16;

                if relocs[instr_index].is_none() {
                    // TODO: use `:08X`.
                    relocs[instr_index] =
                        Some(RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                            RelocReferencedSym::SymName(Arc::from(format!("0x{:X}", constant)), 0),
                        ));
                }
            }
        }

        // Tell labels they have been referenced
        let owned_segment_mut = context.find_owned_segment_mut(parent_segment_info)?;
        for (label_vram, referrer) in referenced_labels_owned_segment {
            if let Some(label) = owned_segment_mut.find_label_mut(label_vram) {
                label.add_referenced_info(referrer);
            }
        }

        for (label_vram, referrer) in referenced_labels_refer_segment {
            let referenced_segment_mut =
                context.find_referenced_segment_mut(label_vram, parent_segment_info);
            if let Some(label) = referenced_segment_mut.find_label_mut(label_vram) {
                label.add_referenced_info(referrer);
            }
        }

        Ok(relocs)
    }

    // Maybe split this function into two, one for hi and another for lo?
    fn reloc_for_instruction(
        global_config: &GlobalConfig,
        instr: &Instruction,
        sym_metadata: Option<&SymbolMetadata>,
        instr_analysis: &InstructionAnalysisResult,
        instr_rom: Rom,
    ) -> RelocationType {
        let opcode = instr.opcode();
        let is_pic = global_config.gp_config().is_some_and(|x| x.pic());

        if opcode.can_be_hi() {
            if is_pic {
                /*
                if contextSym is not None and gotHiLo:
                    if contextSym.isGotGlobal and contextSym.getTypeSpecial() == common.SymbolSpecialType.function:
                        return common.RelocType.MIPS_CALL_HI16
                    else:
                        return common.RelocType.MIPS_GOT_HI16
                */
            }
            RelocationType::R_MIPS_HI16
        } else if opcode.can_be_lo() {
            let rs = instr.field_rs();

            if rs.is_some_and(|x| x.is_global_pointer(instr.abi())) {
                if !is_pic
                /* || gotSmall */
                {
                    return if instr
                        .get_destination_gpr()
                        .is_some_and(|x| x.is_global_pointer(instr.abi()))
                    {
                        // Shouldn't make a gprel access if the dst register is $gp too
                        RelocationType::R_MIPS_LO16
                    } else {
                        RelocationType::R_MIPS_GPREL16
                    };
                }
                if let Some(sym_metadata) = sym_metadata {
                    return if let Some(got_info) = sym_metadata.got_info() {
                        if got_info.access_kind() == GotAccessKind::Global
                            && sym_metadata.sym_type() == Some(SymbolType::Function)
                            && instr_analysis
                                .indirect_function_call()
                                .contains_key(&instr_rom)
                        {
                            RelocationType::R_MIPS_CALL16
                        } else {
                            RelocationType::R_MIPS_GOT16
                        }
                    } else {
                        RelocationType::R_MIPS_GPREL16
                    };
                }
            } else if is_pic {
                /*
                if contextSym is not None and gotHiLo:
                    if contextSym.isGotGlobal and contextSym.getTypeSpecial() == common.SymbolSpecialType.function:
                        return common.RelocType.MIPS_CALL_LO16
                    else:
                        return common.RelocType.MIPS_GOT_LO16
                */
            }
            RelocationType::R_MIPS_LO16
        } else {
            panic!("what")
        }
    }

    pub fn find_and_update_labels(
        context: &mut Context,
        ranges: &RomVramRange,
        parent_segment_info: &ParentSegmentInfo,
    ) -> Result<Vec<Vram>, SymbolPostProcessError> {
        let mut labels = Vec::new();

        let owned_segment = context.find_owned_segment_mut(parent_segment_info)?;

        for (vram, sym) in owned_segment.find_label_range_mut(*ranges.vram()) {
            sym.set_defined();

            let rom =
                Size::new((*vram - ranges.vram().start()).inner() as u32) + ranges.rom().start();
            sym.set_rom(rom);

            labels.push(*vram);
        }

        Ok(labels)
    }
}

impl FunctionSymProcessed {
    #[must_use]
    pub(crate) fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    #[must_use]
    pub(crate) fn handwritten_instrs(&self) -> &UnorderedSet<Rom> {
        self.instr_analysis.handwritten_instrs()
    }

    #[must_use]
    pub fn referenced_vrams(&self) -> &UnorderedSet<Vram> {
        self.instr_analysis.referenced_vrams()
    }

    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn labels(&self) -> &[Vram] {
        &self.labels
    }
}

impl<'ctx, 'sym, 'flg> FunctionSymProcessed {
    pub fn display(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg FunctionDisplaySettings,
    ) -> Result<FunctionDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        self.display_internal(context, settings, InternalSymDisplSettings::new(false))
    }

    pub(crate) fn display_internal(
        &'sym self,
        context: &'ctx Context,
        settings: &'flg FunctionDisplaySettings,
        internal_settings: InternalSymDisplSettings,
    ) -> Result<FunctionDisplay<'ctx, 'sym, 'flg>, SymDisplayError> {
        FunctionDisplay::new(context, self, settings, internal_settings)
    }
}

impl Symbol for FunctionSymProcessed {
    fn vram_range(&self) -> &AddressRange<Vram> {
        self.ranges.vram()
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        &self.parent_segment_info
    }

    #[must_use]
    fn section_type(&self) -> SectionType {
        SECTION_TYPE
    }
}
impl RomSymbol for FunctionSymProcessed {
    #[must_use]
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SymbolProcessed for FunctionSymProcessed {}
impl RomSymbolProcessed for FunctionSymProcessed {
    fn relocs(&self) -> &[Option<RelocationInfo>] {
        &self.relocs
    }
}

impl hash::Hash for FunctionSymProcessed {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for FunctionSymProcessed {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for FunctionSymProcessed {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // Compare segment info first, so symbols get sorted by segment
        match self
            .parent_segment_info
            .partial_cmp(&other.parent_segment_info)
        {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.ranges.partial_cmp(&other.ranges)
    }
}
