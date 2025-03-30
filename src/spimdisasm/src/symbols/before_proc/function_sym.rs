/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use core::hash;
use rabbitizer::{access_type::AccessType, Instruction};

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::{InstructionAnalysisResult, InstructionAnalyzer},
    collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet},
    config::Compiler,
    context::Context,
    metadata::{
        GeneratedBy, GotAccessKind, LabelType, ParentSectionMetadata, ReferrerInfo,
        SegmentMetadata, SymbolMetadata, SymbolNameGenerationSettings, SymbolType,
    },
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
    symbols::{processed::FunctionSymProcessed, RomSymbolPreprocessed, SymbolPreprocessed},
};

use crate::symbols::{
    trait_symbol::RomSymbol, Symbol, SymbolCreationError, SymbolPostProcessError,
};

const SECTION_TYPE: SectionType = SectionType::Text;

#[derive(Debug, Clone)]
pub struct FunctionSym {
    ranges: RomVramRange,
    instructions: Arc<[Instruction]>,
    parent_segment_info: ParentSegmentInfo,
    instr_analysis: InstructionAnalysisResult,
}

impl FunctionSym {
    pub(crate) fn new(
        context: &mut Context,
        instructions: Arc<[Instruction]>,
        rom: Rom,
        vram: Vram,
        _in_section_offset: usize,
        parent_segment_info: ParentSegmentInfo,
        properties: FunctionSymProperties,
    ) -> Result<Self, SymbolCreationError> {
        let size = Size::new(instructions.len() as u32 * 4);
        let rom_range = AddressRange::new(rom, rom + size);
        let vram_range = AddressRange::new(vram, vram + size);
        let ranges = RomVramRange::new(rom_range, vram_range);

        let mut instr_analysis =
            InstructionAnalyzer::analyze(context, &parent_segment_info, ranges, &instructions)?;

        let symbol_name_generation_settings = context
            .global_config()
            .symbol_name_generation_settings()
            .clone();
        let owned_segment = context.find_owned_segment_mut(&parent_segment_info)?;
        let metadata = owned_segment.add_self_symbol(
            vram,
            Some(rom),
            size,
            SECTION_TYPE,
            Some(SymbolType::Function),
            |metadata| count_padding(&instructions, metadata.user_declared_size()),
            symbol_name_generation_settings.clone(),
        )?;

        properties.apply_to_metadata(metadata);

        Self::process_instr_analysis_result_owned(
            &instr_analysis,
            &ranges,
            &parent_segment_info,
            owned_segment,
        )?;
        Self::process_instr_analysis_result_referenced(
            &mut instr_analysis,
            &ranges,
            context,
            &parent_segment_info,
            &symbol_name_generation_settings,
        )?;

        Ok(Self {
            ranges,
            instructions,
            parent_segment_info,
            instr_analysis,
        })
    }

    fn process_instr_analysis_result_owned(
        instr_analysis: &InstructionAnalysisResult,
        ranges: &RomVramRange,
        parent_segment_info: &ParentSegmentInfo,
        owned_segment: &mut SegmentMetadata,
    ) -> Result<(), SymbolCreationError> {
        for (instr_rom, target_vram) in instr_analysis.branch_targets() {
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if self.getVromOffset(instrOffset) in self.context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            let referenced_info = ReferrerInfo::new_function(
                ranges.vram().start(),
                parent_segment_info.clone(),
                *instr_rom,
            );
            let branch_label =
                owned_segment.add_label(*target_vram, LabelType::Branch, referenced_info)?;
            branch_label.set_rom(ranges.rom_from_vram(*target_vram).unwrap());
            branch_label.set_defined();

            /*
            labelSym.referenceCounter += 1
            labelSym.referenceFunctions.add(self.contextSym)
            labelSym.parentFunction = self.contextSym
            labelSym.parentFileName = self.contextSym.parentFileName
            self.contextSym.branchLabels.add(labelSym.vram, labelSym)
            */
        }
        for (instr_rom, target_vram) in instr_analysis.branch_targets_outside() {
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if self.getVromOffset(instrOffset) in self.context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            if owned_segment.in_vram_range(*target_vram)
                && owned_segment
                    .find_symbol(*target_vram, FindSettings::new(false))
                    .is_none()
            {
                // Only generate a label if this outside-branch is not branching to the start of a function

                let referenced_info = ReferrerInfo::new_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );
                let branch_label = owned_segment.add_label(
                    *target_vram,
                    LabelType::AlternativeEntry,
                    referenced_info,
                )?;
                let rom = ranges
                    .rom()
                    .start()
                    .inner()
                    .wrapping_add_signed((*target_vram - ranges.vram().start()).inner());
                branch_label.set_rom(Rom::new(rom));
                branch_label.set_defined();

                /*
                labelSym.referenceCounter += 1
                labelSym.referenceFunctions.add(self.contextSym)
                labelSym.parentFunction = self.contextSym
                labelSym.parentFileName = self.contextSym.parentFileName
                self.contextSym.branchLabels.add(labelSym.vram, labelSym)
                */
                // TODO: add some kind of comment mentioning this instr is branching outside the current function.
            }
        }
        for (instr_rom, target_vram) in instr_analysis.branch_calls() {
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if self.getVromOffset(instrOffset) in self.context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            let referenced_info = ReferrerInfo::new_function(
                ranges.vram().start(),
                parent_segment_info.clone(),
                *instr_rom,
            );
            owned_segment.add_label(*target_vram, LabelType::AlternativeEntry, referenced_info)?;

            if let Some(func_sym) =
                owned_segment.find_symbol_mut(*target_vram, FindSettings::new(false))
            {
                func_sym.set_type(SymbolType::Function, GeneratedBy::Autogenerated);
                func_sym.add_reference_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );
            }

            /*
            labelSym.referenceCounter += 1
            labelSym.referenceFunctions.add(self.contextSym)
            labelSym.parentFunction = self.contextSym
            labelSym.parentFileName = self.contextSym.parentFileName
            self.contextSym.branchLabels.add(labelSym.vram, labelSym)
            */
        }

        Ok(())
    }

    fn process_instr_analysis_result_referenced(
        instr_analysis: &mut InstructionAnalysisResult,
        ranges: &RomVramRange,
        context: &mut Context,
        parent_segment_info: &ParentSegmentInfo,
        symbol_name_generation_settings: &SymbolNameGenerationSettings,
    ) -> Result<(), SymbolCreationError> {
        Self::process_got_syms(
            instr_analysis,
            ranges,
            context,
            parent_segment_info,
            symbol_name_generation_settings,
        )?;

        // Jumptables
        for (instr_rom, target_vram) in instr_analysis.referenced_jumptables() {
            if context
                .find_owned_segment(parent_segment_info)?
                .is_vram_ignored(*target_vram)
            {
                continue;
            }

            let referenced_segment =
                context.find_referenced_segment_mut(*target_vram, parent_segment_info);
            let jumptable = referenced_segment.add_symbol(
                *target_vram,
                false,
                symbol_name_generation_settings.clone(),
            )?;
            jumptable.set_type(SymbolType::Jumptable, GeneratedBy::Autogenerated);
            jumptable.add_reference_function(
                ranges.vram().start(),
                parent_segment_info.clone(),
                *instr_rom,
            );
            /*
            jumpTable.parentFunction = self.contextSym
            self.contextSym.jumpTables.add(jumpTable.vram, jumpTable)
            */
        }

        for (instr_rom, target_vram) in instr_analysis.func_calls() {
            if context
                .find_owned_segment(parent_segment_info)?
                .is_vram_ignored(*target_vram)
            {
                continue;
            }

            /*
            if self.context.isAddressBanned(targetVram):
                continue
            */

            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if self.getVromOffset(instrOffset) in self.context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            let referenced_segment =
                context.find_referenced_segment_mut(*target_vram, parent_segment_info);
            let func_sym = referenced_segment.add_symbol(
                *target_vram,
                true,
                symbol_name_generation_settings.clone(),
            )?;
            if func_sym.vram() == *target_vram {
                func_sym.set_type(SymbolType::Function, GeneratedBy::Autogenerated);
                func_sym.add_reference_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );
            } else {
                let referenced_info = ReferrerInfo::new_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );

                referenced_segment.add_label(
                    *target_vram,
                    LabelType::AlternativeEntry,
                    referenced_info,
                )?;
            }
        }
        for (instr_rom, target_vram) in instr_analysis.maybe_tail_calls() {
            if context
                .find_owned_segment(parent_segment_info)?
                .is_vram_ignored(*target_vram)
            {
                continue;
            }

            /*
            if self.context.isAddressBanned(targetVram):
                continue
            */

            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if self.getVromOffset(instrOffset) in self.context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            let referenced_segment =
                context.find_referenced_segment_mut(*target_vram, parent_segment_info);

            let referenced_info = ReferrerInfo::new_function(
                ranges.vram().start(),
                parent_segment_info.clone(),
                *instr_rom,
            );
            referenced_segment.add_label(
                *target_vram,
                LabelType::AlternativeEntry,
                referenced_info,
            )?;

            if let Some(func_sym) =
                referenced_segment.find_symbol_mut(*target_vram, FindSettings::new(false))
            {
                func_sym.set_type(SymbolType::Function, GeneratedBy::Autogenerated);
                func_sym.add_reference_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );
            }
        }

        for (instr_rom, symbol_vram) in instr_analysis.address_per_lo_instr() {
            /*
            if self.context.isAddressBanned(symVram):
                continue
            */
            /*
            if common.GlobalConfig.INPUT_FILE_TYPE == common.InputFileType.ELF:
                if self.getVromOffset(loOffset) in self.context.globalRelocationOverrides:
                    # Avoid creating wrong symbols on elf files
                    continue
            */

            // Avoid symbolizing `_gp_disp`
            if instr_analysis.cpload_roms().contains(instr_rom) {
                continue;
            }

            let sym_access = if let Some(sym_access_info) =
                instr_analysis.type_info_per_address().get(symbol_vram)
            {
                if sym_access_info.len() == 1 {
                    sym_access_info.iter().next().map(|(k, _v)| k)
                } else {
                    None
                }
            } else {
                None
            };

            let realigned_symbol_vram = match sym_access {
                // Align down the Vram
                Some((AccessType::UNALIGNED_WORD, _)) => {
                    Vram::new(symbol_vram.inner() - (symbol_vram.inner() % 4))
                }
                Some((AccessType::UNALIGNED_DOUBLEWORD, _)) => {
                    Vram::new(symbol_vram.inner() - (symbol_vram.inner() % 8))
                }
                None | Some(_) => *symbol_vram,
            };
            if context
                .find_owned_segment(parent_segment_info)?
                .is_vram_ignored(realigned_symbol_vram)
            {
                continue;
            }

            let referenced_segment =
                context.find_referenced_segment_mut(realigned_symbol_vram, parent_segment_info);

            let sym_metadata = referenced_segment.add_symbol(
                realigned_symbol_vram,
                true,
                symbol_name_generation_settings.clone(),
            )?;
            if sym_metadata.sym_type() != Some(SymbolType::Function)
                || sym_metadata.vram() == realigned_symbol_vram
            {
                sym_metadata.add_reference_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );
                if sym_metadata.owner_segment_kind().is_unknown_segment() {
                    match sym_access {
                        // Set a dummy min size to allow relocs to properly reference this symbol from the unknown segment.
                        // This may not be real tho, I need to properly check.
                        Some((AccessType::UNALIGNED_WORD, _)) => {
                            let siz = sym_metadata
                                .autodetected_size()
                                .unwrap_or(Size::new(4))
                                .max(Size::new(4));
                            *sym_metadata.autodetected_size_mut() = Some(siz);
                        }
                        Some((AccessType::UNALIGNED_DOUBLEWORD, _)) => {
                            let siz = sym_metadata
                                .autodetected_size()
                                .unwrap_or(Size::new(8))
                                .max(Size::new(8));
                            *sym_metadata.autodetected_size_mut() = Some(siz);
                        }
                        None | Some(_) => {}
                    }
                }
                /*
                contextSym = sym_metadata
                # TODO: do this in a less ugly way
                if contextSym.address != symVram:
                    if contextSym.address % 4 != 0 or symVram % 4 != 0:
                        if contextSym.getType() in {"u16", "s16", "u8", "u8"} or (symAccess is not None and symAccess.accessType in {rabbitizer.AccessType.BYTE, rabbitizer.AccessType.SHORT}):
                            if not (contextSym.getSize() > 4):
                                if contextSym.userDeclaredSize is None or symVram >= contextSym.address + contextSym.userDeclaredSize:
                                    if symAccess is not None:
                                        contextSym.setAccessTypeIfUnset(symAccess.accessType, symAccess.unsignedMemoryAccess)
                                    contextSym.setFirstLoAccessIfUnset(loOffset)
                                    contextSym = self.addSymbol(symVram, isAutogenerated=True)
                */

                /*
                contextSym.setFirstLoAccessIfUnset(loOffset)
                */

                if let Some(sym_access_info) =
                    instr_analysis.type_info_per_address().get(symbol_vram)
                {
                    for (k, _) in sym_access_info {
                        sym_metadata.set_access_type(*k);
                    }
                }

                if let Some(_sym_access) = sym_access {
                    /*
                    if contextSym.isAutogenerated:
                        # Handle mips1 doublefloats
                        if contextSym.accessType == rabbitizer.AccessType.FLOAT and common.GlobalConfig.ABI == common.Abi.O32:
                            instr = self.instructions[loOffset//4]
                            if instr.doesDereference() and instr.isFloat() and not instr.isDouble():
                                if instr.ft.value % 2 != 0:
                                    # lwc1/swc1 with an odd fpr means it is an mips1 doublefloats reference
                                    if symVram % 8 != 0:
                                        # We need to remove the the symbol pointing to the middle of this doublefloats
                                        got = contextSym.isGot
                                        gotLocal = contextSym.isGotLocal
                                        gotGlobal = contextSym.isGotGlobal
                                        self.removeSymbol(symVram)

                                        # Align down to 8
                                        symVram = (symVram >> 3) << 3
                                        contextSym = self.addSymbol(symVram, isAutogenerated=True)
                                        contextSym.referenceCounter += 1
                                        contextSym.referenceFunctions.add(self.contextSym)
                                        contextSym.setFirstLoAccessIfUnset(loOffset)
                                        contextSym.isGot = got
                                        contextSym.isGotLocal = gotLocal
                                        contextSym.isGotGlobal = gotGlobal
                                    contextSym.accessType = rabbitizer.AccessType.DOUBLEFLOAT
                                    contextSym.unsignedAccessType = False
                                    contextSym.isMips1Double = True
                    */
                }
            }

            if !sym_metadata.is_defined() || sym_metadata.vram() != realigned_symbol_vram {
                let referenced_info = ReferrerInfo::new_function(
                    ranges.vram().start(),
                    parent_segment_info.clone(),
                    *instr_rom,
                );

                referenced_segment.add_label(
                    realigned_symbol_vram,
                    LabelType::AlternativeEntry,
                    referenced_info,
                )?;
            }
        }

        /*
        # To debug jumptable rejection change this check to `True`
        if False:
            for jrInstrOffset, (referenceOffset, jtblAddress, branchOffset) in self.instrAnalyzer.rejectedjumpRegisterIntrOffset.items():
                self.endOfLineComment[jrInstrOffset//4] = f" /* Jumping to something at address 0x{jtblAddress:08X} (inferred from 0x{self.getVromOffset(referenceOffset):X}). Jumptable rejected by instruction at vrom 0x{self.getVromOffset(branchOffset):X} */
        "
        */

        /*
        if self.isLikelyHandwritten:
            for instr in self.instructions:
                instr.inHandwrittenFunction = self.isLikelyHandwritten
        */

        Ok(())
    }

    fn process_got_syms(
        instr_analysis: &mut InstructionAnalysisResult,
        ranges: &RomVramRange,
        context: &mut Context,
        parent_segment_info: &ParentSegmentInfo,
        symbol_name_generation_settings: &SymbolNameGenerationSettings,
    ) -> Result<(), SymbolCreationError> {
        for (instr_rom, symbol_vram) in instr_analysis
            .global_got_addresses()
            .iter()
            .chain(instr_analysis.unpaired_local_got_addresses())
        {
            let referenced_segment =
                context.find_referenced_segment_mut(*symbol_vram, parent_segment_info);
            let sym_metadata = referenced_segment.add_symbol(
                *symbol_vram,
                true,
                symbol_name_generation_settings.clone(),
            )?;

            sym_metadata.set_got_access_kind(GotAccessKind::Global);
            sym_metadata.add_reference_function(
                ranges.vram().start(),
                parent_segment_info.clone(),
                *instr_rom,
            );
        }

        // TODO
        // for (instr_rom, symbol_vram) in instr_analysis.unpaired_local_got_addresses() {
        //     panic!("\n\n\n{:?} {:?}\n", instr_rom, symbol_vram);
        // }

        for (instr_rom, symbol_vram) in instr_analysis.paired_local_got_addresses() {
            let referenced_segment =
                context.find_referenced_segment_mut(*symbol_vram, parent_segment_info);
            let sym_metadata = referenced_segment.add_symbol(
                *symbol_vram,
                true,
                symbol_name_generation_settings.clone(),
            )?;

            sym_metadata.set_got_access_kind(GotAccessKind::Local);
            sym_metadata.add_reference_function(
                ranges.vram().start(),
                parent_segment_info.clone(),
                *instr_rom,
            );

            if instr_analysis.lo_rom_added_with_gp().contains(instr_rom) {
                sym_metadata.set_add_gp_to_pointed_data();
            }
        }

        Ok(())
    }
}

impl FunctionSym {
    pub(crate) fn post_process(
        self,
        context: &mut Context,
        user_relocs: &BTreeMap<Rom, RelocationInfo>,
    ) -> Result<FunctionSymProcessed, SymbolPostProcessError> {
        FunctionSymProcessed::new(
            context,
            self.ranges,
            self.instructions,
            self.parent_segment_info,
            self.instr_analysis,
            user_relocs,
        )
    }
}

impl FunctionSym {
    #[must_use]
    pub fn referenced_vrams(&self) -> &UnorderedSet<Vram> {
        self.instr_analysis.referenced_vrams()
    }
}

impl Symbol for FunctionSym {
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
impl RomSymbol for FunctionSym {
    #[must_use]
    fn rom_vram_range(&self) -> &RomVramRange {
        &self.ranges
    }
}
impl SymbolPreprocessed for FunctionSym {}
impl RomSymbolPreprocessed for FunctionSym {}

impl hash::Hash for FunctionSym {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.parent_segment_info.hash(state);
        self.ranges.hash(state);
    }
}
impl PartialEq for FunctionSym {
    fn eq(&self, other: &Self) -> bool {
        self.parent_segment_info == other.parent_segment_info && self.ranges == other.ranges
    }
}
impl PartialOrd for FunctionSym {
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

#[derive(Debug, Clone, Hash, PartialEq)]
pub(crate) struct FunctionSymProperties {
    pub parent_metadata: ParentSectionMetadata,
    pub compiler: Option<Compiler>,
    pub auto_pad_by: Option<Vram>,
}

impl FunctionSymProperties {
    fn apply_to_metadata(self, metadata: &mut SymbolMetadata) {
        metadata.set_parent_metadata(self.parent_metadata);

        if let Some(compiler) = self.compiler {
            metadata.set_compiler(compiler);
        }

        if let Some(auto_pad_by) = self.auto_pad_by {
            metadata.set_auto_created_pad_by(auto_pad_by);
        }
    }
}

fn count_padding(instructions: &[Instruction], user_declared_size: Option<Size>) -> Size {
    // We don't consider padding if the user specified the size, or if the function is composed of only nops
    if user_declared_size.is_some() || instructions.iter().all(|x| x.is_nop()) {
        return Size::new(0);
    }

    let mut count = 0;

    for pair in instructions.windows(2).rev() {
        let prev_instr = pair[0];
        let instr = pair[1];

        if prev_instr.opcode().has_delay_slot() {
            break;
        }
        if !instr.is_nop() {
            break;
        }

        count += 4;
    }

    Size::new(count)
}
