/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use core::hash;
use rabbitizer::Instruction;

use crate::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::{
        GatheredTypeInfo, InstrAnalysisInfo, InstrOpJumptable, InstructionAnalysisResult,
        InstructionAnalyzer,
    },
    collections::{
        addended_ordered_map::FindSettings, unordered_map::UnorderedMap,
        unordered_set::UnorderedSet,
    },
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

        let instr_analysis =
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

        Self::process_instr_analysis_result(
            context,
            &instr_analysis,
            &parent_segment_info,
            symbol_name_generation_settings,
        )?;

        Ok(Self {
            ranges,
            instructions,
            parent_segment_info,
            instr_analysis,
        })
    }

    fn process_instr_analysis_result(
        context: &mut Context,
        instr_analysis: &InstructionAnalysisResult,
        parent_segment_info: &ParentSegmentInfo,
        symbol_name_generation_settings: SymbolNameGenerationSettings,
    ) -> Result<(), SymbolCreationError> {
        let ranges = instr_analysis.ranges();
        let self_vram = ranges.vram().start();
        let self_rom = ranges.rom().start();

        let owned_segment = context.find_owned_segment(parent_segment_info)?;

        let mut paired_symbols = UnorderedMap::new();

        for (instr_index, info) in instr_analysis.instruction_infos().iter().enumerate() {
            let instr_rom = self_rom + Size::new(instr_index as u32 * 4);

            match info {
                InstrAnalysisInfo::No => {}

                InstrAnalysisInfo::DirectLink { target_vram } => {
                    if owned_segment.is_vram_ignored(*target_vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*target_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.inferred_type = Some(SymbolType::Function);
                    meta.is_direct_link = true;
                }
                InstrAnalysisInfo::BranchLink { target_vram } => {
                    if owned_segment.is_vram_ignored(*target_vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*target_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.inferred_type = Some(SymbolType::Function);
                    meta.generate_label = GenerateLabel::Yes(LabelType::AlternativeEntry);
                }
                InstrAnalysisInfo::JumpAndLinkRegisterRaw { .. } => { /* TODO? */ }
                InstrAnalysisInfo::JumpAndLinkRegisterDereferenced { .. } => { /* TODO? */ }
                InstrAnalysisInfo::MaybeDirectTailCall { target_vram } => {
                    if owned_segment.is_vram_ignored(*target_vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*target_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.inferred_type = Some(SymbolType::Function);
                    meta.generate_label = GenerateLabel::Yes(LabelType::AlternativeEntry);
                }
                InstrAnalysisInfo::RawRegisterTailCall { .. } => { /* TODO? */ }
                InstrAnalysisInfo::DereferencedRegisterTailCall { .. } => { /* TODO? */ }
                InstrAnalysisInfo::Jumptable {
                    jumptable_vram,
                    kind,
                } => {
                    if owned_segment.is_vram_ignored(*jumptable_vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*jumptable_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.inferred_type = Some(SymbolType::Jumptable);
                    meta.generate_label = GenerateLabel::No;

                    match kind {
                        InstrOpJumptable::Simple => {}
                        InstrOpJumptable::Pic => {
                            meta.add_gp_to_pointed_data = true;
                        }
                    }
                }
                InstrAnalysisInfo::Branch { target_vram } => {
                    let meta = paired_symbols
                        .entry(*target_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.generate_label = GenerateLabel::Yes(LabelType::Branch);
                }
                InstrAnalysisInfo::BranchOutside { target_vram } => {
                    let meta = paired_symbols
                        .entry(*target_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.generate_label = GenerateLabel::Maybe(LabelType::AlternativeEntry);
                }
                InstrAnalysisInfo::UnpairedHi { .. } | InstrAnalysisInfo::PairedHi { .. } => {}
                InstrAnalysisInfo::PairedLo {
                    addended_vram: _,
                    unaddended_vram,
                }
                | InstrAnalysisInfo::GpRel {
                    addended_vram: _,
                    unaddended_vram,
                } => {
                    if owned_segment.is_vram_ignored(*unaddended_vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*unaddended_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                }
                InstrAnalysisInfo::ConstantHi { .. } | InstrAnalysisInfo::ConstantLo { .. } => {}
                InstrAnalysisInfo::GotLazyResolver { .. } => {}
                InstrAnalysisInfo::GotGlobal {
                    addended_vram: vram,
                    unaddended_vram: _,
                }
                | InstrAnalysisInfo::GotLocal {
                    addended_vram: vram,
                    unaddended_vram: _,
                }
                | InstrAnalysisInfo::GotCall16 { vram } => {
                    if owned_segment.is_vram_ignored(*vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.got_access_kind = Some(GotAccessKind::Global);
                }
                InstrAnalysisInfo::GotLocalPaired {
                    addended_vram: _,
                    unaddended_vram,
                } => {
                    if owned_segment.is_vram_ignored(*unaddended_vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*unaddended_vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                    meta.got_access_kind = Some(GotAccessKind::Local);
                }
                InstrAnalysisInfo::PairedGotHi { .. } | InstrAnalysisInfo::GotCallHi { .. } => {}
                InstrAnalysisInfo::PairedGotLo { vram } | InstrAnalysisInfo::GotCallLo { vram } => {
                    if owned_segment.is_vram_ignored(*vram) {
                        continue;
                    }

                    let meta = paired_symbols
                        .entry(*vram)
                        .or_insert(PairedAddressMeta::new(instr_rom));
                    meta.rom_referencers.insert(instr_rom);
                }
                InstrAnalysisInfo::GpSetHi
                | InstrAnalysisInfo::GpSetLo
                | InstrAnalysisInfo::CploadHi
                | InstrAnalysisInfo::CploadLo
                | InstrAnalysisInfo::CploadAddu => {}
            }
        }

        for (vram, paired_meta) in paired_symbols {
            let referenced_segment = context.find_referenced_segment_mut(vram, parent_segment_info);

            let label_type = match paired_meta.generate_label {
                GenerateLabel::Maybe(label_type) => {
                    if label_type == LabelType::Branch
                        || Self::add_paired_symbol(
                            vram,
                            &paired_meta,
                            referenced_segment,
                            &symbol_name_generation_settings,
                            instr_analysis,
                            self_vram,
                            parent_segment_info,
                        )?
                    {
                        Some(label_type)
                    } else {
                        None
                    }
                }
                GenerateLabel::Yes(label_type) => {
                    if label_type != LabelType::Branch {
                        if let Some(sym_metadata) =
                            referenced_segment.find_symbol_mut(vram, FindSettings::new(false))
                        {
                            if let Some(inferred_type) = paired_meta.inferred_type {
                                sym_metadata.set_type(inferred_type, GeneratedBy::Autogenerated);
                            }
                            for rom in &paired_meta.rom_referencers {
                                sym_metadata.add_reference_function(
                                    self_vram,
                                    parent_segment_info.clone(),
                                    *rom,
                                );
                            }
                        }
                    }

                    Some(label_type)
                }
                GenerateLabel::No => {
                    Self::add_paired_symbol(
                        vram,
                        &paired_meta,
                        referenced_segment,
                        &symbol_name_generation_settings,
                        instr_analysis,
                        self_vram,
                        parent_segment_info,
                    )?;
                    None
                }
            };

            if let Some(label_type) = label_type {
                let referenced_info = ReferrerInfo::new_function(
                    self_vram,
                    parent_segment_info.clone(),
                    paired_meta.first_rom_referencer,
                );

                let label = referenced_segment.add_label(vram, label_type, referenced_info)?;

                for rom in paired_meta.rom_referencers {
                    let referenced_info =
                        ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), rom);
                    label.add_creator(referenced_info);
                }

                if label_type == LabelType::Branch {
                    let rom = self_rom
                        .inner()
                        .wrapping_add_signed((vram - self_vram).inner());
                    label.set_rom(Rom::new(rom));
                    label.set_defined();
                }
            }
        }

        Ok(())
    }

    fn add_paired_symbol(
        vram: Vram,
        paired_meta: &PairedAddressMeta,
        referenced_segment: &mut SegmentMetadata,
        symbol_name_generation_settings: &SymbolNameGenerationSettings,
        instr_analysis: &InstructionAnalysisResult,
        self_vram: Vram,
        parent_segment_info: &ParentSegmentInfo,
    ) -> Result<bool, SymbolCreationError> {
        let sym_metadata =
            referenced_segment.add_symbol(vram, true, symbol_name_generation_settings.clone())?;
        if sym_metadata.sym_type() != Some(SymbolType::Function) || sym_metadata.vram() == vram {
            if let Some(inferred_type) = paired_meta.inferred_type {
                sym_metadata.set_type(inferred_type, GeneratedBy::Autogenerated);
            }
            if let Some(got_kind) = paired_meta.got_access_kind {
                sym_metadata.set_got_access_kind(got_kind);
            }
            if paired_meta.add_gp_to_pointed_data {
                sym_metadata.set_add_gp_to_pointed_data();
            }

            for rom in &paired_meta.rom_referencers {
                sym_metadata.add_reference_function(self_vram, parent_segment_info.clone(), *rom);
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

            if let Some(GatheredTypeInfo::AccessInfoCounter(counter)) =
                instr_analysis.type_info_per_address().get(&vram)
            {
                for (k, _) in counter {
                    sym_metadata.set_access_type(*k);
                }
            }

            /*
            let sym_access =
            instr_analysis.type_info_per_address().get(vram).and_then(|x| x.get_access_info());

            if let Some(_sym_access) = sym_access {
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
            }
            */
        }

        if paired_meta.is_direct_link {
            Ok(sym_metadata.vram() != vram)
        } else {
            Ok(!sym_metadata.is_defined() || sym_metadata.vram() != vram)
        }
    }
}

struct PairedAddressMeta {
    first_rom_referencer: Rom,
    rom_referencers: UnorderedSet<Rom>,
    got_access_kind: Option<GotAccessKind>,
    inferred_type: Option<SymbolType>,
    add_gp_to_pointed_data: bool,
    generate_label: GenerateLabel,
    is_direct_link: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum GenerateLabel {
    Maybe(LabelType),
    Yes(LabelType),
    No,
}

impl PairedAddressMeta {
    fn new(first_rom_referencer: Rom) -> Self {
        Self {
            first_rom_referencer,
            rom_referencers: UnorderedSet::new(),
            got_access_kind: None,
            inferred_type: None,
            add_gp_to_pointed_data: false,
            generate_label: GenerateLabel::Maybe(LabelType::AlternativeEntry),
            is_direct_link: false,
        }
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
