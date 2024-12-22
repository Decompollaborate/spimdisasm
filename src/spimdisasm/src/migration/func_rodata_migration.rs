/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::{
    borrow::Cow,
    collections::{btree_set::BTreeSet, vec_deque::VecDeque},
    string::ToString,
    vec::Vec,
};
use rabbitizer::Vram;

use crate::{
    context::Context,
    metadata::{RodataMigrationBehavior, SymbolMetadata, SymbolMetadataNameDisplay},
    sections::{Section, SectionData, SectionExecutable},
    symbols::{
        display::{FunctionDisplaySettings, SymDataDisplaySettings},
        Symbol, SymbolData, SymbolFunction,
    },
};

use super::{FuncRodataPairingDisplay, PairingError, RodataIterator};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FuncRodataPairing {
    SingleFunction {
        function_index: usize,
    },
    SingleRodata {
        rodata_index: usize,
    },
    Pairing {
        function_index: usize,
        rodata_indices: Vec<usize>,
        late_rodata_indices: Vec<usize>,
    },
}

impl FuncRodataPairing {
    pub fn pair_sections(
        context: &Context,
        text_section: Option<&SectionExecutable>,
        rodata_section: Option<&SectionData>,
    ) -> Vec<Self> {
        let mut all_entries = Vec::new();

        /*
        """
        Return all functions to rodata symbols pairings from the given
        sections.

        The returned list preserves original ordering of both the text and
        rodata sections.

        Either a function or a rodata symbol may be orphaned. This means the
        said symbol is paired to zero symbols of the other section. In these
        cases an entry containing the specific symbol is created and inserted
        in corresponding place of the list so it preserves the ordering of both
        sections.
        """
        */

        // # The simplest way to know which symbols has not been migrated yet and
        // # preserve order at the same time seem to be just keeping a list of the
        // # symbols and remove the ones that have been handled somehow (either by
        // # migrating to a function or adding an no-function entry for the given
        // # symbol).
        // # We use deque instead of a plain list because we want fast removal of
        // # the first symbol.
        let all_rodata_syms: &[SymbolData] = rodata_section.map_or(&[], |x| x.data_symbols());
        let mut remaining_rodata_symbols: VecDeque<(usize, &SymbolData)> = rodata_section
            .iter()
            .flat_map(|x| x.data_symbols().iter().enumerate())
            .collect();
        let mut handled_symbols = BTreeSet::new();

        for (func_index, func_sym) in text_section
            .iter()
            .flat_map(|x| x.functions().iter().enumerate())
        {
            let entry = Self::pair_function_to_rodata_section(
                context,
                func_index,
                func_sym,
                rodata_section,
            );

            for rodata in entry.iter_rodata(rodata_section) {
                handled_symbols.insert(rodata.vram_range().start());
            }

            // Preserve the order of rodata symbols by looking for symbols that has not been migrated yet
            if let FuncRodataPairing::Pairing { rodata_indices, .. } = &entry {
                if !rodata_indices.is_empty() {
                    let first_rodata_index = rodata_indices[0];
                    let first_rodata_sym = &all_rodata_syms[first_rodata_index];

                    while !remaining_rodata_symbols.is_empty() {
                        let (rodata_index, rodata_sym) = &remaining_rodata_symbols[0];

                        if handled_symbols.contains(&rodata_sym.vram_range().start()) {
                            // Drop migrated symbols
                            remaining_rodata_symbols.pop_front();
                            continue;
                        }

                        if rodata_sym.vram_range().start() >= first_rodata_sym.vram_range().start()
                        {
                            // Take all the symbols up to symbols referenced by the current function
                            break;
                        }

                        all_entries.push(FuncRodataPairing::SingleRodata {
                            rodata_index: *rodata_index,
                        });
                        handled_symbols.insert(rodata_sym.vram_range().start());
                        remaining_rodata_symbols.pop_front();
                    }
                }
            }

            all_entries.push(entry)
        }

        // Check if there's any rodata symbol remaining and add it to the list
        for (rodata_index, rodata_sym) in remaining_rodata_symbols {
            if !handled_symbols.contains(&rodata_sym.vram_range().start()) {
                all_entries.push(FuncRodataPairing::SingleRodata { rodata_index });
            }
        }

        all_entries
    }

    fn pair_function_to_rodata_section(
        context: &Context,
        function_index: usize,
        function: &SymbolFunction,
        rodata_section: Option<&SectionData>,
    ) -> Self {
        /*
        """
        Pairs the given function to the migrable rodata symbols of the given
        rodata section.

        If `rodataSection` is `None` or if the function does not reference any
        of the symbols from the given `rodataSection` then a
        `FunctionRodataEntry` containing the given function.
        """
        */

        if let Some(rodata_section) = rodata_section {
            let mut rodata_indices = Vec::new();
            let mut late_rodata_indices = Vec::new();

            let intersection: BTreeSet<Vram> = function
                .referenced_vrams()
                .intersection(rodata_section.symbols_vrams())
                .copied()
                .collect();

            let func_metadata = function.find_own_metadata(context);
            let func_name = func_metadata.display_name();
            let compiler_has_late_rodata = func_metadata
                .compiler()
                .is_some_and(|x| x.has_late_rodata());

            let mut migrable_rodata_syms = BTreeSet::new();
            let mut migrable_late_rodata_syms = BTreeSet::new();
            let mut maybe_migrable_rodata_syms = BTreeSet::new();
            let mut maybe_migrable_late_rodata_syms = BTreeSet::new();
            let mut rodata_migrated_somewhere_else: bool = false;
            let mut late_rodata_migrated_somewhere_else: bool = false;
            for rodata_sym in rodata_section.data_symbols() {
                if rodata_migrated_somewhere_else {
                    if !compiler_has_late_rodata {
                        break;
                    }
                    if late_rodata_migrated_somewhere_else {
                        break;
                    }
                }

                let rodata_metadata = rodata_sym.find_own_metadata(context);

                if rodata_metadata.is_late_rodata() {
                    late_rodata_migrated_somewhere_else = Self::update_migrable_symbols_sets(
                        rodata_metadata,
                        &intersection,
                        &func_name,
                        &mut migrable_late_rodata_syms,
                        &mut maybe_migrable_late_rodata_syms,
                        late_rodata_migrated_somewhere_else,
                    )
                } else {
                    rodata_migrated_somewhere_else = Self::update_migrable_symbols_sets(
                        rodata_metadata,
                        &intersection,
                        &func_name,
                        &mut migrable_rodata_syms,
                        &mut maybe_migrable_rodata_syms,
                        rodata_migrated_somewhere_else,
                    )
                }
            }

            for (rodata_index, rodata_sym) in rodata_section.data_symbols().iter().enumerate() {
                let rodata_vram = rodata_sym.vram_range().start();

                if migrable_late_rodata_syms.contains(&rodata_vram) {
                    late_rodata_indices.push(rodata_index);
                } else if migrable_rodata_syms.contains(&rodata_vram) {
                    rodata_indices.push(rodata_index);
                }
            }

            FuncRodataPairing::Pairing {
                function_index,
                rodata_indices,
                late_rodata_indices,
            }
        } else {
            FuncRodataPairing::SingleFunction { function_index }
        }
    }

    fn update_migrable_symbols_sets(
        rodata_metadata: &SymbolMetadata,
        intersection: &BTreeSet<Vram>,
        func_name: &SymbolMetadataNameDisplay,
        migrable_rodata_syms: &mut BTreeSet<Vram>,
        maybe_migrable_rodata_syms: &mut BTreeSet<Vram>,
        mut rodata_migrated_somewhere_else: bool,
    ) -> bool {
        /*
        # We try to decide which symbols should be migrated by checking from left
        # to right.
        # Completely unreferenced symbols may get migrated to the current
        # function if they are between two symbols that do get migrated to this
        # function.
        # This is acomplished by keeping a second set of tentative symbols to
        # migrate (`maybe_migrable_rodata_syms`) which gets added to the main set
        # when we see the next migrable symbol.
        */

        if rodata_migrated_somewhere_else {
            return rodata_migrated_somewhere_else;
        }

        if Self::should_migrate_rodata_symbol_to_function(rodata_metadata, intersection, func_name)
        {
            migrable_rodata_syms.insert(rodata_metadata.vram());

            // Move everything from `maybe_migrable_rodata_syms` to `migrable_rodata_syms`
            migrable_rodata_syms.append(maybe_migrable_rodata_syms);
        } else if !migrable_rodata_syms.is_empty() {
            if rodata_metadata.reference_counter() > 0 {
                rodata_migrated_somewhere_else = true;
            } else if rodata_metadata.is_migrable() {
                maybe_migrable_rodata_syms.insert(rodata_metadata.vram());
            } else {
                rodata_migrated_somewhere_else = true;
            }
        }

        rodata_migrated_somewhere_else
    }

    fn should_migrate_rodata_symbol_to_function(
        rodata_metadata: &SymbolMetadata,
        intersection: &BTreeSet<Vram>,
        func_name: &SymbolMetadataNameDisplay,
    ) -> bool {
        if let RodataMigrationBehavior::MigrateToSpecificFunction(owner_name) =
            rodata_metadata.rodata_migration_behavior()
        {
            // If a function owner was specified for this symbol then it is only
            // allowed to be migrated to that function and none other
            *owner_name == func_name.to_string()
        } else if !intersection.contains(&rodata_metadata.vram()) {
            false
        } else {
            rodata_metadata.is_migrable()
        }
    }
}

impl<'ctx> FuncRodataPairing {
    pub fn display_name(
        &self,
        context: &'ctx Context,
        text_section: Option<&SectionExecutable>,
        rodata_section: Option<&SectionData>,
    ) -> Result<SymbolMetadataNameDisplay<'ctx>, PairingError> {
        let metadata = match &self {
            FuncRodataPairing::Pairing { function_index, .. }
            | FuncRodataPairing::SingleFunction { function_index } => {
                if let Some(text_section) = text_section {
                    let functions = text_section.functions();

                    if let Some(func) = functions.get(*function_index) {
                        func.find_own_metadata(context)
                    } else {
                        return Err(PairingError::FunctionOutOfBounds {
                            index: *function_index,
                            len: functions.len(),
                            section_name: text_section.name().into(),
                        });
                    }
                } else {
                    return Err(PairingError::MissingTextSection);
                }
            }
            FuncRodataPairing::SingleRodata { rodata_index } => {
                if let Some(rodata_section) = rodata_section {
                    let data_symbols = rodata_section.data_symbols();

                    if let Some(rodata) = data_symbols.get(*rodata_index) {
                        rodata.find_own_metadata(context)
                    } else {
                        return Err(PairingError::RodataOutOfBounds {
                            index: *rodata_index,
                            len: data_symbols.len(),
                            section_name: rodata_section.name().into(),
                        });
                    }
                } else {
                    return Err(PairingError::MissingRodataSection);
                }
            }
        };

        Ok(metadata.display_name())
    }
}

impl<'pairing, 'rodata> FuncRodataPairing {
    pub fn iter_rodata(
        &'pairing self,
        rodata_section: Option<&'rodata SectionData>,
    ) -> RodataIterator<'pairing, 'rodata> {
        RodataIterator::new(self, rodata_section)
    }
}

impl<
        'ctx,
        'text,
        'text_settings,
        'rodata,
        'rodata_settings,
        'text_label,
        'ro_label,
        'late_ro_label,
    > FuncRodataPairing
{
    pub fn display(
        &self,
        context: &'ctx Context,
        text_section: Option<&'text SectionExecutable>,
        function_display_settings: &'text_settings FunctionDisplaySettings,
        rodata_section: Option<&'rodata SectionData>,
        rodata_display_settings: &'rodata_settings SymDataDisplaySettings,
        section_label_text: Option<Cow<'text_label, str>>,
        section_label_rodata: Option<Cow<'ro_label, str>>,
        section_label_late_rodata: Option<Cow<'late_ro_label, str>>,
    ) -> Result<
        FuncRodataPairingDisplay<
            'ctx,
            'text,
            'text_settings,
            'rodata,
            'rodata_settings,
            'text_label,
            'ro_label,
            'late_ro_label,
        >,
        PairingError,
    > {
        FuncRodataPairingDisplay::new(
            self,
            context,
            text_section,
            function_display_settings,
            rodata_section,
            rodata_display_settings,
            section_label_text,
            section_label_rodata,
            section_label_late_rodata,
        )
    }
}
