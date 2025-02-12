/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{fmt, hash::Hash};

use alloc::string::String;
use rabbitizer::access_type::AccessType;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Size, SizedAddress, Vram},
    collections::{unordered_map::UnorderedMap, unordered_set::UnorderedSet},
    config::Compiler,
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
};

use super::{ParentSectionMetadata, SymbolMetadataNameDisplay, SymbolType};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub /*(crate)*/ enum GeneratedBy {
    /// This symbol was automatically generated by the disassembler
    Autogenerated,
    /// Declared externally by the user, but it may have not been found yet
    UserDeclared,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StringInfo {
    is_maybe_string: bool,
    failed_string_decoding: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GotInfo {
    is_got: bool, // TODO: maybe redundant?
    is_got_global: bool,
    is_got_local: bool,
    got_index: Option<usize>, // TODO: maybe remove Option?
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", eq))]
pub enum RodataMigrationBehavior {
    /// Let spimdisasm handle if it should migrate this rodata symbol.
    Default(),

    /// Ignore rules for migrating rodata and force migration of this symbol to any
    /// function which references it.
    ForceMigrate(),
    /// Ignore rules for migrating rodata and prevent migration of this symbol to
    /// any function which references it.
    ForceNotMigrate(),

    /// Force migrating to the function that matches the specified name.
    ///
    /// Overrides all logic for determining if this symbol should be migrated or
    /// not and to which function should be migrated.
    ///
    /// This can be specially useful for unreferenced symbols that should be
    /// defined in-between actually referenced symbols.
    ///
    /// This field is ignored if applied on anything that is not a rodata symbol.
    ///
    /// WARNING: It is undefined behavior if during rodata migration the listed
    /// function does not exists on the given text section. For example this symbol
    /// may get lost in limbo.
    MigrateToSpecificFunction(String),
}

impl Default for RodataMigrationBehavior {
    fn default() -> Self {
        Self::Default()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum OwnerSegmentKind {
    Global,
    // The string corresponds to the name of the segment.
    Overlay(String),
    Unknown,
    User,
}

impl OwnerSegmentKind {
    pub fn is_unknown_segment(&self) -> bool {
        matches!(self, OwnerSegmentKind::Unknown)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct SymbolMetadata {
    generated_by: GeneratedBy,
    vram: Vram,
    rom: Option<Rom>,

    user_declared_name: Option<String>,
    user_declared_name_end: Option<String>,

    // TODO: Is this still necessary?
    /// Used to register a name of a symbol which may change in the future.
    ///
    /// The only parameter is the ContextSymbol itself, and it should return a string containing the name of the symbol.
    ///
    /// Used by .get_name() instead of using the setted name or the default generated name.
    // name_get_callback: Option<Box<dyn FnOnce(&SymbolMetadata) -> String>>,
    user_declared_size: Option<Size>,
    autodetected_size: Option<Size>,

    user_declared_type: Option<SymbolType>,
    autodetected_type: Option<SymbolType>,

    section_type: Option<SectionType>,

    /// This symbol exists (was found) in any of the analyzed sections
    is_defined: bool,

    // TODO: merge these two
    access_type: Option<(AccessType, bool)>,
    access_types: UnorderedMap<AccessType, u32>,

    c_string_info: Option<StringInfo>,
    // pascal_string_info: Option<StringInfo>,
    /// Which functions reference this symbol
    /// Key is the vram of the function and the segment it is contained on (since vrams can overlap
    /// on different segments).
    /// Value is the rom of the instruction that references this symbol, so we can know how many
    /// times a function references the same symbol.
    reference_functions: UnorderedMap<(Vram, ParentSegmentInfo), UnorderedSet<Rom>>,
    /// Which symbols reference this symbol
    /// Key is the vram of the non-function symbol and the segment it is contained on (since vrams
    /// can overlap on different segments).
    /// Value is the rom of the word that references this symbol, so we can know how many
    /// times a function references the same symbol.
    reference_symbols: UnorderedMap<(Vram, ParentSegmentInfo), UnorderedSet<Rom>>,

    // TODO: how to reimplement these crossreferences?
    // parentFunction: ContextSymbol|None = None
    // "Parent function for branch labels, jump tables, and jump table labels"
    // branchLabels: SortedDict[ContextSymbol] = dataclasses.field(default_factory=SortedDict)
    // "For functions, the branch and jump table labels which are contained in this function"
    // jumpTables: SortedDict[ContextSymbol] = dataclasses.field(default_factory=SortedDict)
    // "For functions, the jump tables which are contained in this function"

    // parentFileName: str|None = None
    // "Name of the file containing this symbol"
    // inFileOffset: int|None = None
    // "Offset relative to the start of the file"
    owner_segment_kind: OwnerSegmentKind,

    //
    got_info: Option<GotInfo>,

    accessed_as_gp_rel: bool,

    // _isStatic: bool = False

    // TODO: These two are kinda redundant
    auto_created_pad_by: Option<Vram>,

    // firstLoAccess: int|None = None

    // isElfNotype: bool = False

    //
    rodata_migration_behavior: RodataMigrationBehavior,

    allow_ref_with_addend: bool,
    /*
    allowedToReferenceAddends: bool = False
    notAllowedToReferenceAddends: bool = False

    allowedToReferenceConstants: bool = False
    notAllowedToReferenceConstants: bool = False

    allowedToReferenceSymbols: bool = True
    """
    Allow or prohibit this symbol to reference other symbols.
    """

    allowedToBeReferenced: bool = True
    """
    Allow or prohibit this symbol to be referenced by other symbols.
    """
    */
    is_mips1_double: bool,

    visibility: Option<String>,

    compiler: Option<Compiler>,
    parent_metadata: Option<ParentSectionMetadata>,

    trailing_padding_size: Option<Size>,
}

impl SymbolMetadata {
    pub(crate) fn new(
        generated_by: GeneratedBy,
        vram: Vram,
        owner_segment_kind: OwnerSegmentKind,
    ) -> Self {
        Self {
            generated_by,
            vram,
            rom: None,

            user_declared_name: None,
            user_declared_name_end: None,

            user_declared_size: None,
            autodetected_size: None,
            user_declared_type: None,
            autodetected_type: None,

            section_type: None,

            is_defined: false,

            access_type: None,
            access_types: UnorderedMap::new(),
            c_string_info: None,
            // pascal_string_info: None,
            reference_functions: UnorderedMap::new(),
            reference_symbols: UnorderedMap::new(),
            // name_get_callback: None,
            owner_segment_kind,
            got_info: None,
            accessed_as_gp_rel: false,
            auto_created_pad_by: None,
            rodata_migration_behavior: RodataMigrationBehavior::Default(),
            allow_ref_with_addend: true,
            is_mips1_double: false,
            visibility: None,

            compiler: None,
            parent_metadata: None,

            trailing_padding_size: None,
        }
    }

    pub fn generated_by(&self) -> GeneratedBy {
        self.generated_by
    }

    pub const fn vram(&self) -> Vram {
        self.vram
    }

    pub fn rom(&self) -> Option<Rom> {
        self.rom
    }
    pub(crate) fn rom_mut(&mut self) -> &mut Option<Rom> {
        &mut self.rom
    }

    pub fn display_name(&self) -> SymbolMetadataNameDisplay {
        SymbolMetadataNameDisplay::new(self)
    }

    pub(crate) fn user_declared_name(&self) -> Option<&str> {
        self.user_declared_name.as_deref()
    }
    pub fn user_declared_name_mut(&mut self) -> &mut Option<String> {
        &mut self.user_declared_name
    }

    pub fn user_declared_name_end_mut(&mut self) -> &mut Option<String> {
        &mut self.user_declared_name_end
    }

    pub fn user_declared_size(&self) -> Option<Size> {
        self.user_declared_size
    }
    pub fn user_declared_size_mut(&mut self) -> &mut Option<Size> {
        &mut self.user_declared_size
    }
    pub fn autodetected_size(&self) -> Option<Size> {
        self.autodetected_size
    }
    pub(crate) fn autodetected_size_mut(&mut self) -> &mut Option<Size> {
        &mut self.autodetected_size
    }
    pub fn size(&self) -> Option<Size> {
        // TODO

        if let Some(siz) = self.user_declared_size {
            return Some(siz);
        }
        if let Some(siz) = self.autodetected_size {
            return Some(siz);
        }

        // TODO: Infer size based on user-declared type

        // TODO: Infer size based on instruction access type

        None
    }

    pub fn sym_type(&self) -> Option<SymbolType> {
        if let Some(t) = &self.user_declared_type {
            Some(*t)
        } else {
            self.autodetected_type
        }
    }
    pub fn user_declared_type(&self) -> Option<SymbolType> {
        self.user_declared_type
    }
    pub fn user_declared_type_mut(&mut self) -> &mut Option<SymbolType> {
        &mut self.user_declared_type
    }
    pub fn autodetected_type(&self) -> Option<SymbolType> {
        self.autodetected_type
    }
    pub(crate) fn set_type(&mut self, new_type: SymbolType, generated_by: GeneratedBy) {
        match generated_by {
            GeneratedBy::Autogenerated => self.autodetected_type = Some(new_type),
            GeneratedBy::UserDeclared => self.user_declared_type = Some(new_type),
        }
    }
    pub(crate) fn set_type_with_priorities(
        &mut self,
        new_type: SymbolType,
        generated_by: GeneratedBy,
    ) {
        match new_type {
            SymbolType::BranchLabel => match self.sym_type() {
                Some(SymbolType::Function)
                | Some(SymbolType::JumptableLabel)
                | Some(SymbolType::GccExceptTableLabel) => {
                    // Functions, jumptable labels and gccexcepttable labels take precedence over plain labels.

                    if generated_by == GeneratedBy::Autogenerated {
                        // This type may come from the user, so if we haven't set what type we detected this type to be then set it anyways.
                        // Setting it doesn't really matter since user-declread info takes precendence anyways
                        match self.autodetected_type() {
                            Some(SymbolType::Function)
                            | Some(SymbolType::JumptableLabel)
                            | Some(SymbolType::GccExceptTableLabel) => {}
                            _ => self.set_type(SymbolType::BranchLabel, generated_by),
                        }
                    }
                }
                _ => self.set_type(SymbolType::BranchLabel, generated_by),
            },
            SymbolType::JumptableLabel => match self.sym_type() {
                Some(SymbolType::Function) | Some(SymbolType::GccExceptTableLabel) => {
                    // Functions and gccexcepttable labels take precedence over jumptable labels.

                    if generated_by == GeneratedBy::Autogenerated {
                        // This type may come from the user, so if we haven't set what type we detected this type to be then set it anyways.
                        // Setting it doesn't really matter since user-declread info takes precendence anyways
                        match self.autodetected_type() {
                            Some(SymbolType::Function) | Some(SymbolType::GccExceptTableLabel) => {}
                            _ => self.set_type(SymbolType::JumptableLabel, generated_by),
                        }
                    }
                }
                _ => self.set_type(SymbolType::JumptableLabel, generated_by),
            },
            SymbolType::GccExceptTableLabel => match self.sym_type() {
                Some(SymbolType::Function) => {
                    // Functions take precedence over gccexcepttable labels.

                    if generated_by == GeneratedBy::Autogenerated {
                        // This type may come from the user, so if we haven't set what type we detected this type to be then set it anyways.
                        // Setting it doesn't really matter since user-declread info takes precendence anyways
                        match self.autodetected_type() {
                            Some(SymbolType::Function) => {}
                            _ => self.set_type(SymbolType::GccExceptTableLabel, generated_by),
                        }
                    }
                }
                _ => self.set_type(SymbolType::GccExceptTableLabel, generated_by),
            },
            typ => self.set_type(typ, generated_by),
        }
    }

    pub fn section_type(&self) -> Option<SectionType> {
        self.section_type
    }
    pub(crate) fn section_type_mut(&mut self) -> &mut Option<SectionType> {
        &mut self.section_type
    }

    pub fn is_defined(&self) -> bool {
        self.is_defined
    }
    pub(crate) fn set_defined(&mut self) {
        self.is_defined = true;
    }

    pub fn access_type(&self) -> Option<(AccessType, bool)> {
        self.access_type
    }
    pub(crate) fn all_access_types(&self) -> &UnorderedMap<AccessType, u32> {
        &self.access_types
    }
    pub(crate) fn set_access_type_if_unset(&mut self, access_type: (AccessType, bool)) {
        *self.access_types.entry(access_type.0).or_default() += 1;
        if self.access_type.is_none() {
            self.access_type = Some(access_type);
            if self.autodetected_type.is_none() {
                self.autodetected_type = SymbolType::from_access_type(access_type.0);
            }
        }
    }

    /// How much this symbol is referenced by something else
    pub fn reference_counter(&self) -> usize {
        self.reference_functions.values().count() + self.reference_symbols.values().count()
    }
    pub(crate) fn add_reference_function(
        &mut self,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
        rom: Rom,
    ) {
        self.reference_functions
            .entry((vram, parent_segment_info))
            .or_default()
            .insert(rom);
    }
    pub(crate) fn add_reference_symbol(
        &mut self,
        vram: Vram,
        parent_segment_info: ParentSegmentInfo,
        rom: Rom,
    ) {
        self.reference_symbols
            .entry((vram, parent_segment_info))
            .or_default()
            .insert(rom);
    }

    pub(crate) fn owner_segment_kind(&self) -> &OwnerSegmentKind {
        &self.owner_segment_kind
    }

    pub(crate) fn autogenerated_pad_info(&self) -> Option<Vram> {
        if self.reference_counter() == 0 && self.generated_by == GeneratedBy::Autogenerated {
            self.auto_created_pad_by
        } else {
            None
        }
    }
    pub(crate) fn set_auto_created_pad_by(&mut self, vram: Vram) {
        self.auto_created_pad_by = Some(vram);
    }

    #[must_use]
    pub fn rodata_migration_behavior(&self) -> &RodataMigrationBehavior {
        &self.rodata_migration_behavior
    }
    #[must_use]
    pub fn rodata_migration_behavior_mut(&mut self) -> &mut RodataMigrationBehavior {
        &mut self.rodata_migration_behavior
    }

    pub fn allow_ref_with_addend(&self) -> bool {
        self.allow_ref_with_addend
    }
    pub fn set_allow_ref_with_addend(&mut self, val: bool) {
        self.allow_ref_with_addend = val;
    }

    pub fn visibility(&self) -> Option<&str> {
        self.visibility.as_deref()
    }
    pub fn visibility_mut(&mut self) -> &mut Option<String> {
        &mut self.visibility
    }

    pub(crate) fn compiler(&self) -> Option<Compiler> {
        self.compiler
    }
    pub(crate) fn set_compiler(&mut self, compiler: Compiler) {
        self.compiler = Some(compiler);
    }

    pub(crate) fn parent_metadata(&self) -> Option<&ParentSectionMetadata> {
        self.parent_metadata.as_ref()
    }
    pub(crate) fn set_parent_metadata(&mut self, parent_metadata: ParentSectionMetadata) {
        self.parent_metadata = Some(parent_metadata);
    }

    pub fn trailing_padding_size(&self) -> Option<Size> {
        self.trailing_padding_size
    }
    pub(crate) fn set_trailing_padding_size(&mut self, size: Size) {
        self.trailing_padding_size = Some(size);
    }
}

impl SymbolMetadata {
    pub fn is_trustable_function(&self) -> bool {
        // """Checks if the function symbol should be trusted based on the current disassembler settings"""

        /*
        if self.unknownSegment:
            return False

        if self.isGotLocal:
            return False

        if self.isAutocreatedSymFromOtherSizedSym:
            return True
        */

        match self.sym_type() {
            Some(SymbolType::Function) => true,
            None => {
                // Users may not type the symbol as a function.
                self.generated_by() == GeneratedBy::UserDeclared
            }
            Some(_) => false,
        }
        /*
        currentType = self.getTypeSpecial()

        if self.isUserDeclared:
            if currentType == SymbolSpecialType.branchlabel:
                return False
            return True

        if self.isAutogenerated and currentType == SymbolSpecialType.function:
            return True

        if rsp:
            return True

        return False
        */
    }

    fn is_maybe_const_variable(&self) -> bool {
        if self.section_type != Some(SectionType::Rodata) {
            return false;
        }

        if let Some(sym_type) = self.sym_type() {
            match sym_type {
                SymbolType::Function => false,
                SymbolType::BranchLabel => false,
                SymbolType::Jumptable => false,
                SymbolType::JumptableLabel => false,
                SymbolType::GccExceptTable => false,
                SymbolType::GccExceptTableLabel => false,
                SymbolType::Byte => true,
                SymbolType::Short => true,
                SymbolType::Word => true,
                SymbolType::DWord => true,
                SymbolType::Float32 => {
                    if let (Some(size), Some(padding)) = (self.size(), self.trailing_padding_size())
                    {
                        // Check if we have more than a single float here. If that's the case, then
                        // let's say it is a const variable instead of a float literal.
                        size != padding + Size::new(4)
                    } else {
                        false
                    }
                }
                SymbolType::Float64 => {
                    if let (Some(size), Some(padding)) = (self.size(), self.trailing_padding_size())
                    {
                        // Check if we have more than a single double here. If that's the case, then
                        // let's say it is a const variable instead of a double literal.
                        size != padding + Size::new(8)
                    } else {
                        false
                    }
                }
                SymbolType::CString => false,
                SymbolType::VirtualTable => true,
                SymbolType::UserCustom => !self.compiler.is_some_and(|x| x.forbids_const_structs()),
            }
        } else {
            true
        }
    }

    pub(crate) fn is_late_rodata(&self) -> bool {
        self.sym_type()
            .is_some_and(|x| x.is_late_rodata(self.compiler))
    }

    pub(crate) fn is_migrable(&self) -> bool {
        match self.rodata_migration_behavior {
            RodataMigrationBehavior::MigrateToSpecificFunction(_) => true,
            RodataMigrationBehavior::ForceMigrate() => true,
            RodataMigrationBehavior::ForceNotMigrate() => false,
            RodataMigrationBehavior::Default() => {
                if self.is_mips1_double {
                    true
                } else if !self.reference_symbols.is_empty() || self.reference_functions.len() > 1 {
                    false
                } else if self.is_maybe_const_variable() {
                    self.compiler.is_some_and(|x| x.allow_rdata_migration())
                } else {
                    true
                }
            }
        }
    }
}

impl PartialEq for SymbolMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.vram == other.vram && self.rom == other.rom
    }
}
impl PartialOrd for SymbolMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.vram.partial_cmp(&other.vram) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        };
        self.rom.partial_cmp(&other.rom)
    }
}
impl Hash for SymbolMetadata {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.vram.hash(state);
        self.rom.hash(state);
    }
}

impl fmt::Debug for SymbolMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SymbolMetadata {{ vram: 0x{}, name: \"{}\" }}",
            self.vram,
            self.display_name()
        )
    }
}

impl SizedAddress for SymbolMetadata {
    fn size(&self) -> Option<Size> {
        self.size()
    }
}
