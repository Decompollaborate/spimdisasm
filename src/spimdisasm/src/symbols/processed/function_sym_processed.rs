/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};
use core::hash;
use rabbitizer::Instruction;

use crate::{
    addresses::{AddressRange, GotGlobalEntry, Rom, RomVramRange, Size, Vram},
    analysis::{InstrAnalysisInfo, InstructionAnalysisResult},
    collections::{addended_ordered_map::FindSettings, unordered_set::UnorderedSet},
    context::Context,
    metadata::{ReferrerInfo, SegmentMetadata, SymbolType},
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
        instructions: &[Instruction],
        parent_segment_info: &ParentSegmentInfo,
        instr_analysis: &InstructionAnalysisResult,
    ) -> Result<Vec<Option<RelocationInfo>>, SymbolPostProcessError> {
        let ranges = instr_analysis.ranges();
        let self_vram = ranges.vram().start();
        let self_rom = ranges.rom().start();

        let mut relocs = vec![None; instructions.len()];

        let mut referenced_symbols_owned_segment = Vec::new();

        let mut referenced_labels_owned_segment = Vec::new();
        let mut referenced_labels_refer_segment = Vec::new();

        let owned_segment = context.find_owned_segment(parent_segment_info)?;

        for (instr_index, info) in instr_analysis.instruction_infos().iter().enumerate() {
            let instr_rom = self_rom + Size::new(instr_index as u32 * 4);

            let reloc = match info {
                InstrAnalysisInfo::No => None,
                InstrAnalysisInfo::DirectLink { target_vram }
                | InstrAnalysisInfo::MaybeDirectTailCall { target_vram } => {
                    if owned_segment.is_vram_ignored(*target_vram) {
                        None
                    } else {
                        let referenced_sym = if context
                            .find_symbol_from_any_segment(
                                *target_vram,
                                parent_segment_info,
                                FindSettings::new(false),
                                |x| {
                                    x.is_defined()
                                        && x.sym_type().is_some_and(|y| y.valid_call_target())
                                },
                            )
                            .is_some()
                        {
                            RelocReferencedSym::new_address(*target_vram)
                        } else if context
                            .find_label_from_any_segment(*target_vram, parent_segment_info, |_| {
                                true
                            })
                            .is_some()
                        {
                            referenced_labels_refer_segment.push((
                                *target_vram,
                                ReferrerInfo::new_function(
                                    self_vram,
                                    parent_segment_info.clone(),
                                    instr_rom,
                                ),
                            ));

                            RelocReferencedSym::Label(*target_vram)
                        } else {
                            // whatever
                            RelocReferencedSym::new_address(*target_vram)
                        };

                        let reloc_type = RelocationType::R_MIPS_26;
                        Some(reloc_type.new_reloc_info(referenced_sym))
                    }
                }
                InstrAnalysisInfo::BranchLink { target_vram } => {
                    let reloc_type = RelocationType::R_MIPS_PC16;
                    let referrer_info = ReferrerInfo::new_function(
                        self_vram,
                        parent_segment_info.clone(),
                        instr_rom,
                    );

                    if let Some(branch_sym) = owned_segment
                        .find_symbol(*target_vram, FindSettings::new(false))
                        .filter(|x| x.sym_type() == Some(SymbolType::Function))
                    {
                        debug_assert!(branch_sym.vram() == *target_vram);
                        referenced_symbols_owned_segment.push((*target_vram, referrer_info));

                        Some(
                            reloc_type
                                .new_reloc_info(RelocReferencedSym::new_address(*target_vram)),
                        )
                    } else if owned_segment.find_label(*target_vram).is_some() {
                        referenced_labels_owned_segment.push((*target_vram, referrer_info));

                        Some(reloc_type.new_reloc_info(RelocReferencedSym::Label(*target_vram)))
                    } else {
                        None
                    }
                }
                InstrAnalysisInfo::JumpAndLinkRegisterRaw { .. } => None,
                InstrAnalysisInfo::JumpAndLinkRegisterDereferenced { .. } => None,
                InstrAnalysisInfo::RawRegisterTailCall { .. } => None,
                InstrAnalysisInfo::DereferencedRegisterTailCall { .. } => None,
                InstrAnalysisInfo::Jumptable { .. } => None,
                InstrAnalysisInfo::Branch { target_vram } => {
                    let reloc_type = RelocationType::R_MIPS_PC16;
                    if let Some(branch_sym) = owned_segment.find_label(*target_vram) {
                        debug_assert!(branch_sym.vram() == *target_vram);

                        referenced_labels_owned_segment.push((
                            *target_vram,
                            ReferrerInfo::new_function(
                                self_vram,
                                parent_segment_info.clone(),
                                instr_rom,
                            ),
                        ));

                        Some(reloc_type.new_reloc_info(RelocReferencedSym::Label(*target_vram)))
                    } else {
                        // TODO: Add a comment with a warning saying that the target wasn't found
                        None
                    }
                }
                InstrAnalysisInfo::BranchOutside { target_vram } => {
                    // TODO: add some kind of comment mentioning this instr is branching outside the current function.
                    let reloc_type = RelocationType::R_MIPS_PC16;
                    if let Some(branch_sym) = owned_segment
                        .find_symbol(*target_vram, FindSettings::new(false))
                        .filter(|x| x.sym_type() == Some(SymbolType::Function))
                    {
                        debug_assert!(branch_sym.vram() == *target_vram);

                        Some(
                            reloc_type
                                .new_reloc_info(RelocReferencedSym::new_address(*target_vram)),
                        )
                    } else if owned_segment.find_label(*target_vram).is_some() {
                        referenced_labels_owned_segment.push((
                            *target_vram,
                            ReferrerInfo::new_function(
                                self_vram,
                                parent_segment_info.clone(),
                                instr_rom,
                            ),
                        ));

                        Some(reloc_type.new_reloc_info(RelocReferencedSym::Label(*target_vram)))
                    } else {
                        // TODO: Add a comment with a warning saying that the target wasn't found
                        None
                    }
                }
                InstrAnalysisInfo::UnpairedHi { value } => {
                    let constant = *value;

                    Some(RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                        RelocReferencedSym::SymName(Arc::from(format!("0x{:X}", constant)), 0),
                    ))
                }
                InstrAnalysisInfo::PairedHi {
                    addended_vram,
                    unaddended_vram,
                } => {
                    if let Some(referenced_sym) = Self::get_reloc_ref_from_sym(
                        *addended_vram,
                        *unaddended_vram,
                        context,
                        owned_segment,
                        parent_segment_info,
                        &mut referenced_labels_refer_segment,
                        self_vram,
                        instr_rom,
                        None,
                    ) {
                        let reloc_type = RelocationType::R_MIPS_HI16;
                        Some(reloc_type.new_reloc_info(referenced_sym))
                    } else {
                        let instr = &instructions[instr_index];
                        instr.get_processed_immediate().map(|imm| {
                            RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                                RelocReferencedSym::SymName(
                                    Arc::from(format!("0x{:08X}", imm << 16)),
                                    0,
                                ),
                            )
                        })
                    }
                }
                InstrAnalysisInfo::PairedLo {
                    addended_vram,
                    unaddended_vram,
                    upper_rom: _,
                } => Self::get_reloc_ref_from_sym(
                    *addended_vram,
                    *unaddended_vram,
                    context,
                    owned_segment,
                    parent_segment_info,
                    &mut referenced_labels_refer_segment,
                    self_vram,
                    instr_rom,
                    None,
                )
                .map(|referenced_sym| {
                    let reloc_type = RelocationType::R_MIPS_LO16;
                    reloc_type.new_reloc_info(referenced_sym)
                }),
                InstrAnalysisInfo::ConstantHi { constant } => {
                    // TODO: use `:08X`.
                    Some(RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                        RelocReferencedSym::SymName(Arc::from(format!("0x{:X}", constant)), 0),
                    ))
                }
                InstrAnalysisInfo::ConstantLo {
                    constant,
                    upper_rom: _,
                } => {
                    // TODO: use `:08X`.
                    Some(RelocationType::R_CUSTOM_CONSTANT_LO.new_reloc_info(
                        RelocReferencedSym::SymName(Arc::from(format!("0x{:X}", constant)), 0),
                    ))
                }
                InstrAnalysisInfo::GpRel {
                    addended_vram,
                    unaddended_vram,
                } => Self::get_reloc_ref_from_sym(
                    *addended_vram,
                    *unaddended_vram,
                    context,
                    owned_segment,
                    parent_segment_info,
                    &mut referenced_labels_refer_segment,
                    self_vram,
                    instr_rom,
                    None,
                )
                .map(|referenced_sym| {
                    let reloc_type = RelocationType::R_MIPS_GPREL16;
                    reloc_type.new_reloc_info(referenced_sym)
                }),
                InstrAnalysisInfo::GotLazyResolver {
                    addended_vram,
                    unaddended_vram,
                }
                | InstrAnalysisInfo::GotLocal {
                    addended_vram,
                    unaddended_vram,
                }
                | InstrAnalysisInfo::GotLocalPaired {
                    addended_vram,
                    unaddended_vram,
                } => Self::get_reloc_ref_from_sym(
                    *addended_vram,
                    *unaddended_vram,
                    context,
                    owned_segment,
                    parent_segment_info,
                    &mut referenced_labels_refer_segment,
                    self_vram,
                    instr_rom,
                    None,
                )
                .map(|referenced_sym| {
                    let reloc_type = RelocationType::R_MIPS_GOT16;
                    reloc_type.new_reloc_info(referenced_sym)
                }),
                InstrAnalysisInfo::GotGlobal {
                    addended_vram,
                    unaddended_vram,
                    global_entry,
                } => Self::get_reloc_ref_from_sym(
                    *addended_vram,
                    *unaddended_vram,
                    context,
                    owned_segment,
                    parent_segment_info,
                    &mut referenced_labels_refer_segment,
                    self_vram,
                    instr_rom,
                    Some(global_entry),
                )
                .map(|referenced_sym| {
                    let reloc_type = RelocationType::R_MIPS_GOT16;
                    reloc_type.new_reloc_info(referenced_sym)
                }),
                InstrAnalysisInfo::GotGlobalCall16 { vram, global_entry } => {
                    Self::get_reloc_ref_from_sym(
                        *vram,
                        *vram,
                        context,
                        owned_segment,
                        parent_segment_info,
                        &mut referenced_labels_refer_segment,
                        self_vram,
                        instr_rom,
                        Some(global_entry),
                    )
                    .map(|referenced_sym| {
                        let reloc_type = RelocationType::R_MIPS_CALL16;
                        reloc_type.new_reloc_info(referenced_sym)
                    })
                }
                InstrAnalysisInfo::GotLocalCall16 { vram } => Self::get_reloc_ref_from_sym(
                    *vram,
                    *vram,
                    context,
                    owned_segment,
                    parent_segment_info,
                    &mut referenced_labels_refer_segment,
                    self_vram,
                    instr_rom,
                    None,
                )
                .map(|referenced_sym| {
                    let reloc_type = RelocationType::R_MIPS_CALL16;
                    reloc_type.new_reloc_info(referenced_sym)
                }),
                InstrAnalysisInfo::PairedGotHi { vram, global_entry } => {
                    if let Some(referenced_sym) = Self::get_reloc_ref_from_sym(
                        *vram,
                        *vram,
                        context,
                        owned_segment,
                        parent_segment_info,
                        &mut referenced_labels_refer_segment,
                        self_vram,
                        instr_rom,
                        global_entry.as_ref(),
                    ) {
                        let reloc_type = RelocationType::R_MIPS_GOT_HI16;
                        Some(reloc_type.new_reloc_info(referenced_sym))
                    } else {
                        let instr = &instructions[instr_index];
                        instr.get_processed_immediate().map(|imm| {
                            RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                                RelocReferencedSym::SymName(
                                    Arc::from(format!("0x{:08X}", imm << 16)),
                                    0,
                                ),
                            )
                        })
                    }
                }
                InstrAnalysisInfo::PairedGotLo { vram, global_entry } => {
                    Self::get_reloc_ref_from_sym(
                        *vram,
                        *vram,
                        context,
                        owned_segment,
                        parent_segment_info,
                        &mut referenced_labels_refer_segment,
                        self_vram,
                        instr_rom,
                        global_entry.as_ref(),
                    )
                    .map(|referenced_sym| {
                        let reloc_type = RelocationType::R_MIPS_GOT_LO16;
                        reloc_type.new_reloc_info(referenced_sym)
                    })
                }
                InstrAnalysisInfo::GotCallHi { vram, global_entry } => {
                    if let Some(referenced_sym) = Self::get_reloc_ref_from_sym(
                        *vram,
                        *vram,
                        context,
                        owned_segment,
                        parent_segment_info,
                        &mut referenced_labels_refer_segment,
                        self_vram,
                        instr_rom,
                        global_entry.as_ref(),
                    ) {
                        let reloc_type = RelocationType::R_MIPS_CALL_HI16;
                        Some(reloc_type.new_reloc_info(referenced_sym))
                    } else {
                        let instr = &instructions[instr_index];
                        instr.get_processed_immediate().map(|imm| {
                            RelocationType::R_CUSTOM_CONSTANT_HI.new_reloc_info(
                                RelocReferencedSym::SymName(
                                    Arc::from(format!("0x{:08X}", imm << 16)),
                                    0,
                                ),
                            )
                        })
                    }
                }
                InstrAnalysisInfo::GotCallLo { vram, global_entry } => {
                    Self::get_reloc_ref_from_sym(
                        *vram,
                        *vram,
                        context,
                        owned_segment,
                        parent_segment_info,
                        &mut referenced_labels_refer_segment,
                        self_vram,
                        instr_rom,
                        global_entry.as_ref(),
                    )
                    .map(|referenced_sym| {
                        let reloc_type = RelocationType::R_MIPS_CALL_LO16;

                        reloc_type.new_reloc_info(referenced_sym)
                    })
                }
                InstrAnalysisInfo::GpSetHi => Some(
                    RelocationType::R_MIPS_HI16
                        .new_reloc_info(RelocReferencedSym::SymName(Arc::from("_gp"), 0)),
                ),
                InstrAnalysisInfo::GpSetLo => Some(
                    RelocationType::R_MIPS_LO16
                        .new_reloc_info(RelocReferencedSym::SymName(Arc::from("_gp"), 0)),
                ),

                // `.cpload`` directive uses the `_gp_disp` pseudo-symbol
                InstrAnalysisInfo::CploadHi => Some(
                    RelocationType::R_MIPS_HI16
                        .new_reloc_info(RelocReferencedSym::SymName("_gp_disp".into(), 0)),
                ),
                InstrAnalysisInfo::CploadLo => Some(
                    RelocationType::R_MIPS_LO16
                        .new_reloc_info(RelocReferencedSym::SymName("_gp_disp".into(), 0)),
                ),
                InstrAnalysisInfo::CploadAddu => None,
            };

            relocs[instr_index] = reloc;
        }

        // Tell symbols they have been referenced
        let owned_segment_mut = context.find_owned_segment_mut(parent_segment_info)?;
        for (sym_vram, referrer) in referenced_symbols_owned_segment {
            if let Some(sym) = owned_segment_mut.find_symbol_mut(sym_vram, FindSettings::new(true))
            {
                sym.add_referenced_info(referrer);
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

    // TODO: reduce amount of arguments
    #[expect(clippy::too_many_arguments)]
    fn get_reloc_ref_from_sym(
        addended_vram: Vram,
        unaddended_vram: Vram,
        context: &Context,
        owned_segment: &SegmentMetadata,
        parent_segment_info: &ParentSegmentInfo,
        referenced_labels_refer_segment: &mut Vec<(Vram, ReferrerInfo)>,
        self_vram: Vram,
        instr_rom: Rom,
        global_entry: Option<&GotGlobalEntry>,
    ) -> Option<RelocReferencedSym> {
        let sym_name = global_entry.and_then(|x| {
            let sym_name = x.sym_name();
            if !sym_name.is_empty() {
                Some(sym_name)
            } else {
                None
            }
        });

        if let Some(sym_name) = sym_name {
            // Prioritize symbols from the got table
            Some(RelocReferencedSym::SymName(sym_name, 0))
        } else if owned_segment.is_vram_ignored(unaddended_vram) {
            None
        } else {
            let metadata = context.find_symbol_from_any_segment(
                unaddended_vram,
                parent_segment_info,
                FindSettings::new(true),
                |x| x.sym_type() != Some(SymbolType::Function) || x.vram() == unaddended_vram,
            );

            // Check in case we are referencing a label/aent/etc
            let referenced_sym = if metadata.is_some() {
                RelocReferencedSym::Address {
                    addended_vram,
                    unaddended_vram,
                }
            } else {
                referenced_labels_refer_segment.push((
                    addended_vram,
                    ReferrerInfo::new_function(self_vram, parent_segment_info.clone(), instr_rom),
                ));

                RelocReferencedSym::Label(addended_vram)
            };

            Some(referenced_sym)
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
    #[expect(dead_code)]
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
