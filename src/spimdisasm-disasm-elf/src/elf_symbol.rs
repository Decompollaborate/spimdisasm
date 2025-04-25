/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{cmp, hash::Hash};
use std::sync::Arc;

use object::{read::elf::Sym, StringTable};
use spimdisasm::addresses::Size;

use crate::utils;

#[derive(Debug, Clone)]
pub struct ElfSymbol {
    value: u32,
    size: Option<Size>,
    typ: ElfSymType,
    bind: ElfSymBind,
    visibility: ElfSymVisibility,
    section_index: ElfSymSectionIndex,
    name: Arc<str>,
    is_got_global: bool,
}

impl ElfSymbol {
    #[must_use]
    pub fn new(
        sym: &object::elf::Sym32<object::endian::Endianness>,
        elf_endian: object::endian::Endianness,
        string_table: StringTable,
    ) -> Self {
        let value = sym.st_value(elf_endian);

        let size = {
            let s = sym.st_size(elf_endian);
            if s == 0 {
                None
            } else {
                Some(Size::new(s))
            }
        };

        let typ = sym.st_type().into();

        let bind = sym.st_bind().into();
        let visibility = sym.st_visibility().into();

        let section_index = sym.st_shndx(elf_endian).into();

        let raw_name = utils::pretty_unwrap(sym.name(elf_endian, string_table));
        let name = utils::pretty_unwrap(std::str::from_utf8(raw_name)).into();

        Self {
            value,
            size,
            typ,
            bind,
            visibility,
            section_index,
            name,
            is_got_global: false,
        }
    }

    #[must_use]
    pub fn value(&self) -> u32 {
        self.value
    }
    #[must_use]
    pub fn size(&self) -> Option<Size> {
        self.size
    }
    #[must_use]
    pub fn typ(&self) -> ElfSymType {
        self.typ
    }
    #[must_use]
    pub fn bind(&self) -> ElfSymBind {
        self.bind
    }
    #[must_use]
    #[expect(dead_code)]
    pub fn visibility(&self) -> ElfSymVisibility {
        self.visibility
    }
    #[must_use]
    pub fn section_index(&self) -> ElfSymSectionIndex {
        self.section_index
    }
    #[must_use]
    pub fn name(&self) -> Arc<str> {
        self.name.clone()
    }

    #[must_use]
    pub fn is_got_global(&self) -> bool {
        self.is_got_global
    }
    pub fn set_is_got_global(&mut self) {
        self.is_got_global = true;
    }
}

impl PartialEq for ElfSymbol {
    fn eq(&self, other: &Self) -> bool {
        // This ensures we'll get a compilation error if new members are added and are not handled here.
        let ElfSymbol {
            value,
            size,
            typ,
            bind,
            visibility,
            section_index: _section_index,
            name,
            is_got_global,
        } = self;
        let ElfSymbol {
            value: other_value,
            size: other_size,
            typ: other_typ,
            bind: other_bind,
            visibility: other_visibility,
            section_index: _other_section_index,
            name: other_name,
            is_got_global: other_is_got_global,
        } = other;

        value == other_value
            && size == other_size
            && typ == other_typ
            && bind == other_bind
            && visibility == other_visibility
            && name == other_name
            && is_got_global == other_is_got_global
    }
}

impl Eq for ElfSymbol {}

impl Hash for ElfSymbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ElfSymbol {
            value,
            size,
            typ,
            bind,
            visibility,
            section_index: _section_index,
            name,
            is_got_global,
        } = self;
        value.hash(state);
        size.hash(state);
        typ.hash(state);
        bind.hash(state);
        visibility.hash(state);
        name.hash(state);
        is_got_global.hash(state);
    }
}

impl PartialOrd for ElfSymbol {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ElfSymbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // We want to prioritize global binding symbols over everything else, as a way to minimize
        // symbols from using weak or local.
        // Then we want to see got_global symbols first.
        // Then we want to sort by visibility. (Not completely sure about this tho)
        // Then symbols with sizes to appear before the size-less symbols.
        // Then name ordering
        // and finally the rest.

        // De-structure the structs.
        // This ensures we'll get a compilation error if new members are added and are not handled here.
        let ElfSymbol {
            value,
            size,
            typ,
            bind,
            visibility,
            section_index: _section_index,
            name,
            is_got_global,
        } = self;
        let ElfSymbol {
            value: other_value,
            size: other_size,
            typ: other_typ,
            bind: other_bind,
            visibility: other_visibility,
            section_index: _other_section_index,
            name: other_name,
            is_got_global: other_is_got_global,
        } = other;

        match value.cmp(other_value) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match bind.cmp(other_bind) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match is_got_global.cmp(other_is_got_global) {
            cmp::Ordering::Equal => {}
            cmp::Ordering::Greater => return cmp::Ordering::Less,
            cmp::Ordering::Less => return cmp::Ordering::Greater,
        }
        match visibility.cmp(other_visibility) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match size.cmp(other_size) {
            cmp::Ordering::Equal => {}
            cmp::Ordering::Greater => return cmp::Ordering::Less,
            cmp::Ordering::Less => return cmp::Ordering::Greater,
        }
        match name.cmp(other_name) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        typ.cmp(other_typ)
    }
}

/// Corresponds to `STT_*`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElfSymType {
    /// 0. Symbol type is unspecified.
    NoType,
    /// 1. Symbol is a data object.
    Object,
    /// 2. Symbol is a code object.
    Function,
    /// 3. Symbol is associated with a section.
    Section,
    /// 4. Symbol's name is a file name.
    File,
    /// 5. Symbol is a common data object.
    Common,
    /// 6. Symbol is a thread-local storage object.
    Tls,
    Unknown(u8),
}

impl From<u8> for ElfSymType {
    fn from(val: u8) -> Self {
        match val {
            object::elf::STT_NOTYPE => ElfSymType::NoType,
            object::elf::STT_OBJECT => ElfSymType::Object,
            object::elf::STT_FUNC => ElfSymType::Function,
            object::elf::STT_SECTION => ElfSymType::Section,
            object::elf::STT_FILE => ElfSymType::File,
            object::elf::STT_COMMON => ElfSymType::Common,
            object::elf::STT_TLS => ElfSymType::Tls,
            x => ElfSymType::Unknown(x),
        }
    }
}

/// Corresponds to `STB_*`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ElfSymBind {
    /// 0. Local symbol.
    Local,
    /// 1. Global symbol.
    Global,
    /// 2. Weak symbol.
    Weak,
    Unknown(u8),
}

impl ElfSymBind {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            ElfSymBind::Local => Some("local"),
            ElfSymBind::Weak => Some("weak"),
            // Default binding is global, so there's no need to specify it explicitly
            ElfSymBind::Global | ElfSymBind::Unknown(_) => None,
        }
    }
}

impl From<u8> for ElfSymBind {
    fn from(val: u8) -> Self {
        match val {
            object::elf::STB_LOCAL => ElfSymBind::Local,
            object::elf::STB_GLOBAL => ElfSymBind::Global,
            object::elf::STB_WEAK => ElfSymBind::Weak,
            x => ElfSymBind::Unknown(x),
        }
    }
}

impl PartialOrd for ElfSymBind {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ElfSymBind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Order goes: Global < Weak < Local < Unknown

        match (self, other) {
            (ElfSymBind::Local, ElfSymBind::Local) => cmp::Ordering::Equal,
            (ElfSymBind::Global, ElfSymBind::Global) => cmp::Ordering::Equal,
            (ElfSymBind::Weak, ElfSymBind::Weak) => cmp::Ordering::Equal,
            (ElfSymBind::Unknown(_), ElfSymBind::Unknown(_)) => cmp::Ordering::Equal,

            (ElfSymBind::Global, _) => cmp::Ordering::Less,
            (_, ElfSymBind::Global) => cmp::Ordering::Greater,

            (ElfSymBind::Weak, _) => cmp::Ordering::Less,
            (_, ElfSymBind::Weak) => cmp::Ordering::Greater,

            (ElfSymBind::Local, _) => cmp::Ordering::Less,
            (_, ElfSymBind::Local) => cmp::Ordering::Greater,
        }
    }
}

/// Corresponds to `STV_*`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElfSymVisibility {
    /// 0. Default symbol visibility rules.
    Default,
    /// 1. Processor specific hidden class.
    Internal,
    /// 2. Symbol is not visible to other components.
    Hidden,
    /// 3. Symbol is visible to other components, but is not preemptible.
    Protected,
}

impl From<u8> for ElfSymVisibility {
    fn from(val: u8) -> Self {
        match val {
            object::elf::STV_DEFAULT => ElfSymVisibility::Default,
            object::elf::STV_INTERNAL => ElfSymVisibility::Internal,
            object::elf::STV_HIDDEN => ElfSymVisibility::Hidden,
            object::elf::STV_PROTECTED => ElfSymVisibility::Protected,
            x => panic!("Unknown sym visibility found: {:?}", x),
        }
    }
}

/// Corresponds to `SHN_*`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElfSymSectionIndex {
    /// 0. Undefined section.
    Undef,
    /// 0xFF00. Allocated common symbols.
    MipsACommon,
    /// 0xFF01. Allocated test symbols.
    MipsText,
    /// 0xFF02. Allocated data symbols.
    MipsData,
    /// 0xFF03. Small common symbols.
    MipsSCommon,
    /// 0xFF04. Small undefined symbols.
    MipsSUndefined,
    /// 0xFFF1. Associated symbol is absolute.
    Abs,
    /// 0xFFF2. Associated symbol is common.
    Common,
    ActualSectionIndex(u16),
}

impl From<u16> for ElfSymSectionIndex {
    fn from(val: u16) -> Self {
        match val {
            object::elf::SHN_UNDEF => ElfSymSectionIndex::Undef,
            object::elf::SHN_MIPS_ACOMMON => ElfSymSectionIndex::MipsACommon,
            object::elf::SHN_MIPS_TEXT => ElfSymSectionIndex::MipsText,
            object::elf::SHN_MIPS_DATA => ElfSymSectionIndex::MipsData,
            object::elf::SHN_MIPS_SCOMMON => ElfSymSectionIndex::MipsSCommon,
            object::elf::SHN_MIPS_SUNDEFINED => ElfSymSectionIndex::MipsSUndefined,
            object::elf::SHN_ABS => ElfSymSectionIndex::Abs,
            object::elf::SHN_COMMON => ElfSymSectionIndex::Common,
            x => ElfSymSectionIndex::ActualSectionIndex(x),
        }
    }
}
