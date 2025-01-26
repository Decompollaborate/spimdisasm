/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::vec::Vec;
use core::hash;
use rabbitizer::Instruction;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::InstructionAnalysisResult,
    collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet},
    context::Context,
    parent_segment_info::ParentSegmentInfo,
    relocation::{RelocReferencedSym, RelocationInfo, RelocationType},
    section_type::SectionType,
    symbols::{
        display::{
            FunctionDisplay, FunctionDisplaySettings, InternalSymDisplSettings, SymDisplayError,
        },
        RomSymbol, RomSymbolProcessed, Symbol, SymbolPostProcessError, SymbolProcessed,
    },
};

const SECTION_TYPE: SectionType = SectionType::Text;

#[derive(Debug, Clone)]
pub struct FunctionSymProcessed {
    ranges: RomVramRange,
    instructions: Vec<Instruction>,
    parent_segment_info: ParentSegmentInfo,
    instr_analysis: InstructionAnalysisResult,

    relocs: Vec<Option<RelocationInfo>>,
    labels: Vec<Vram>,
}

impl FunctionSymProcessed {
    pub(crate) fn new(
        context: &mut Context,
        ranges: RomVramRange,
        instructions: Vec<Instruction>,
        parent_segment_info: ParentSegmentInfo,
        instr_analysis: InstructionAnalysisResult,
    ) -> Result<Self, SymbolPostProcessError> {
        let relocs = Self::generate_relocs(
            context,
            &ranges,
            &instructions,
            &parent_segment_info,
            &instr_analysis,
        )?;
        let labels = Self::find_and_update_labels(context, &ranges, &parent_segment_info)?;

        Ok(Self {
            ranges,
            instructions,
            parent_segment_info,
            instr_analysis,
            relocs,
            labels,
        })
    }

    fn generate_relocs(
        context: &Context,
        ranges: &RomVramRange,
        instructions: &[Instruction],
        parent_segment_info: &ParentSegmentInfo,
        instr_analysis: &InstructionAnalysisResult,
    ) -> Result<Vec<Option<RelocationInfo>>, SymbolPostProcessError> {
        let mut relocs = vec![None; instructions.len()];

        let owned_segment = context.find_owned_segment(parent_segment_info)?;

        for (instr_rom, target_vram) in instr_analysis.branch_targets() {
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(instrOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if let Some(branch_sym) =
                owned_segment.find_symbol(*target_vram, FindSettings::new(false))
            {
                debug_assert!(branch_sym.vram() == *target_vram);
                if branch_sym
                    .sym_type()
                    .is_some_and(|x| x.valid_branch_target())
                {
                    let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
                    relocs[instr_index as usize] = Some(
                        RelocationType::R_MIPS_PC16
                            .new_reloc_info(RelocReferencedSym::Address(*target_vram)),
                    );

                    // TODO: keep here?
                    // contextSym.branchLabels.add(labelSym.vram, labelSym)
                } else {
                    // TODO: Warning saying this is not a valid branch target?
                    // TODO: maybe still emit the relocation?
                }
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

            if let Some(branch_sym) =
                owned_segment.find_symbol(*target_vram, FindSettings::new(false))
            {
                debug_assert!(branch_sym.vram() == *target_vram);
                if branch_sym
                    .sym_type()
                    .is_some_and(|x| x.valid_branch_target())
                {
                    let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
                    relocs[instr_index as usize] = Some(
                        RelocationType::R_MIPS_PC16
                            .new_reloc_info(RelocReferencedSym::Address(*target_vram)),
                    );

                    // TODO: keep here?
                    // contextSym.branchLabels.add(labelSym.vram, labelSym)
                } else {
                    // TODO: Warning saying this is not a valid branch target?
                    // TODO: maybe still emit the relocation?
                }
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
            relocs[instr_index as usize] = Some(
                RelocationType::R_MIPS_26.new_reloc_info(RelocReferencedSym::Address(*target_vram)),
            );
        }

        for (instr_rom, symbol_vram) in instr_analysis.address_per_lo_instr() {
            let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;

            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if getVromOffset(loOffset) in context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if owned_segment.is_vram_ignored(*symbol_vram) {
                relocs[instr_index as usize] =
                    Some(RelocationType::R_CUSTOM_CONSTANT_LO.new_reloc_info(
                        RelocReferencedSym::SymName(format!("0x{:08X}", symbol_vram.inner()), 0),
                    ));
                continue;
            }

            relocs[instr_index as usize] = Some(
                RelocationType::R_MIPS_LO16
                    .new_reloc_info(RelocReferencedSym::Address(*symbol_vram)),
            );
        }

        for (instr_rom, symbol_vram) in instr_analysis.address_per_hi_instr() {
            let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;

            if owned_segment.is_vram_ignored(*symbol_vram) {
                relocs[instr_index as usize] =
                    Some(RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                        RelocReferencedSym::SymName(format!("0x{:08X}", symbol_vram.inner()), 0),
                    ));
                continue;
            }

            let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
            relocs[instr_index as usize] = Some(
                RelocationType::R_MIPS_HI16
                    .new_reloc_info(RelocReferencedSym::Address(*symbol_vram)),
            );
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

        /*
        for instrOffset, gpInfo in instrAnalyzer.gpSets.items():
            hiInstrOffset = gpInfo.hiOffset
            hiInstr = instructions[hiInstrOffset//4]
            instr = instructions[instrOffset//4]

            hiRelocType = _getRelocTypeForInstruction(hiInstr, hiInstrOffset)
            relocType = _getRelocTypeForInstruction(instr, instrOffset)
            if not common.GlobalConfig.PIC and gpInfo.value == common.GlobalConfig.GP_VALUE:
                relocs[hiInstrOffset] = common.RelocationInfo(hiRelocType, "_gp")
                relocs[instrOffset] = common.RelocationInfo(relocType, "_gp")
            else:
                # TODO: consider reusing the logic of the instrAnalyzer.symbolInstrOffset loop
                address = gpInfo.value
                if context.isAddressBanned(address):
                    continue

                contextSym = getSymbol(address)
                if contextSym is None:
                    continue

                relocs[hiInstrOffset] = common.RelocationInfo(hiRelocType, contextSym)
                relocs[instrOffset] = common.RelocationInfo(relocType, contextSym)
        */

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
            relocs[instr_index as usize] = Some(
                reloc_type
                    .new_reloc_info(RelocReferencedSym::SymName(format!("0x{:X}", constant), 0)),
            );
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
                let instr_index = (*instr_rom - ranges.rom().start()).inner() / 4;
                let constant = (*hi_imm as u32) << 16;

                // TODO: use `:08X`.
                relocs[instr_index as usize] =
                    Some(RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                        RelocReferencedSym::SymName(format!("0x{:X}", constant), 0),
                    ));
            }
        }

        Ok(relocs)
    }

    pub fn find_and_update_labels(
        context: &mut Context,
        ranges: &RomVramRange,
        parent_segment_info: &ParentSegmentInfo,
    ) -> Result<Vec<Vram>, SymbolPostProcessError> {
        let mut labels = Vec::new();

        let owned_segment = context.find_owned_segment_mut(parent_segment_info)?;
        let owned_metadata = owned_segment
            .find_symbol(ranges.vram().start(), FindSettings::new(false))
            .unwrap();
        let in_overlay = owned_metadata.in_overlay();

        for (vram, sym) in owned_segment.find_symbol_ranges_mut(*ranges.vram()) {
            sym.set_defined();
            *sym.section_type_mut() = Some(SECTION_TYPE);

            let rom =
                Size::new((*vram - ranges.vram().start()).inner() as u32) + ranges.rom().start();
            *sym.rom_mut() = Some(rom);

            if let Some(in_overlay) = in_overlay {
                sym.set_in_overlay(in_overlay);
            }

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
