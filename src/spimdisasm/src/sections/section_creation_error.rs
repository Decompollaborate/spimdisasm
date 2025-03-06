/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Rom, Vram},
    context::OwnedSegmentNotFoundError,
    metadata::{segment_metadata::AddSymbolError, AddLabelError},
    symbols::SymbolCreationError,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub enum SectionCreationError {
    OwnedSegmentNotFound(OwnedSegmentNotFoundError),
    AddSymbol(AddSymbolError),
    AddLabel(AddLabelError),
    EmptySection(EmptySectionError),
    BadBytesSize(BadBytesSizeError),
    UnalignedVram(UnalignedVramError),
    UnalignedRom(UnalignedRomError),
    RomVramAlignmentMismatch(RomVramAlignmentMismatchError),
    AlreadyCreated(SectionAlreadyCreatedError),
    NotPrehated(SectionNotPreheatedError),
}

impl fmt::Display for SectionCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionCreationError::OwnedSegmentNotFound(x) => {
                write!(f, "{}", x)
            }
            SectionCreationError::AddSymbol(x) => write!(f, "{}", x),
            SectionCreationError::AddLabel(x) => write!(f, "{}", x),
            SectionCreationError::EmptySection(x) => write!(f, "{}", x),
            SectionCreationError::BadBytesSize(x) => write!(f, "{}", x),
            SectionCreationError::UnalignedVram(x) => write!(f, "{}", x),
            SectionCreationError::UnalignedRom(x) => write!(f, "{}", x),
            SectionCreationError::RomVramAlignmentMismatch(x) => write!(f, "{}", x),
            SectionCreationError::AlreadyCreated(x) => write!(f, "{}", x),
            SectionCreationError::NotPrehated(x) => write!(f, "{}", x),
        }
    }
}
impl error::Error for SectionCreationError {}

impl From<SymbolCreationError> for SectionCreationError {
    fn from(value: SymbolCreationError) -> Self {
        match value {
            SymbolCreationError::OwnedSegmentNotFound(x) => {
                SectionCreationError::OwnedSegmentNotFound(x)
            }
            SymbolCreationError::AddSymbol(x) => SectionCreationError::AddSymbol(x),
            SymbolCreationError::AddLabel(x) => SectionCreationError::AddLabel(x),
        }
    }
}
impl From<OwnedSegmentNotFoundError> for SectionCreationError {
    fn from(value: OwnedSegmentNotFoundError) -> Self {
        SectionCreationError::OwnedSegmentNotFound(value)
    }
}
impl From<AddSymbolError> for SectionCreationError {
    fn from(value: AddSymbolError) -> Self {
        SectionCreationError::AddSymbol(value)
    }
}
impl From<EmptySectionError> for SectionCreationError {
    fn from(value: EmptySectionError) -> Self {
        SectionCreationError::EmptySection(value)
    }
}
impl From<BadBytesSizeError> for SectionCreationError {
    fn from(value: BadBytesSizeError) -> Self {
        SectionCreationError::BadBytesSize(value)
    }
}
impl From<UnalignedVramError> for SectionCreationError {
    fn from(value: UnalignedVramError) -> Self {
        SectionCreationError::UnalignedVram(value)
    }
}
impl From<UnalignedRomError> for SectionCreationError {
    fn from(value: UnalignedRomError) -> Self {
        SectionCreationError::UnalignedRom(value)
    }
}
impl From<RomVramAlignmentMismatchError> for SectionCreationError {
    fn from(value: RomVramAlignmentMismatchError) -> Self {
        SectionCreationError::RomVramAlignmentMismatch(value)
    }
}
impl From<SectionAlreadyCreatedError> for SectionCreationError {
    fn from(value: SectionAlreadyCreatedError) -> Self {
        SectionCreationError::AlreadyCreated(value)
    }
}
impl From<SectionNotPreheatedError> for SectionCreationError {
    fn from(value: SectionNotPreheatedError) -> Self {
        SectionCreationError::NotPrehated(value)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct EmptySectionError {
    name: Arc<str>,
    vram: Vram,
}
impl EmptySectionError {
    pub(crate) fn new(name: Arc<str>, vram: Vram) -> Self {
        Self { name, vram }
    }
}
impl fmt::Display for EmptySectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Can't initialize section '{}' ({:?}) with empty bytes.",
            self.name, self.vram
        )
    }
}
impl error::Error for EmptySectionError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct BadBytesSizeError {
    name: Arc<str>,
    size: usize,
    multiple_of: usize,
}
impl BadBytesSizeError {
    pub(crate) fn new(name: Arc<str>, size: usize, multiple_of: usize) -> Self {
        Self {
            name,
            size,
            multiple_of,
        }
    }
}
impl fmt::Display for BadBytesSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't create section {} because the bytes length (0x{:X}) is not a multiple of 0x{:X}.", self.name, self.size, self.multiple_of)
    }
}
impl error::Error for BadBytesSizeError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UnalignedVramError {
    name: Arc<str>,
    vram: Vram,
    multiple_of: usize,
}
impl UnalignedVramError {
    pub(crate) fn new(name: Arc<str>, vram: Vram, multiple_of: usize) -> Self {
        Self {
            name,
            vram,
            multiple_of,
        }
    }
}
impl fmt::Display for UnalignedVramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Can't create section {} because the vram ({:?}) is not aligned to 0x{:X}.",
            self.name, self.vram, self.multiple_of
        )
    }
}
impl error::Error for UnalignedVramError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UnalignedRomError {
    name: Arc<str>,
    rom: Rom,
    multiple_of: usize,
}
impl UnalignedRomError {
    pub(crate) fn new(name: Arc<str>, rom: Rom, multiple_of: usize) -> Self {
        Self {
            name,
            rom,
            multiple_of,
        }
    }
}
impl fmt::Display for UnalignedRomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Can't create section {} because the rom (0x{:X}) is not aligned to 0x{:X}.",
            self.name,
            self.rom.inner(),
            self.multiple_of
        )
    }
}
impl error::Error for UnalignedRomError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct RomVramAlignmentMismatchError {
    name: Arc<str>,
    rom: Rom,
    vram: Vram,
    multiple_of: usize,
}
impl RomVramAlignmentMismatchError {
    pub(crate) fn new(name: Arc<str>, rom: Rom, vram: Vram, multiple_of: usize) -> Self {
        Self {
            name,
            rom,
            vram,
            multiple_of,
        }
    }
}
impl fmt::Display for RomVramAlignmentMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Can't create section {} because the alignment of its rom ({:?}) and vram ({:?}) mod {} does not match.", self.name, self.rom, self.vram, self.multiple_of)
    }
}
impl error::Error for RomVramAlignmentMismatchError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionAlreadyCreatedError {
    name: Arc<str>,
    rom: Option<Rom>,
    vram: Vram,
}
impl SectionAlreadyCreatedError {
    pub(crate) fn new(name: Arc<str>, rom: Option<Rom>, vram: Vram) -> Self {
        Self { name, rom, vram }
    }
}
impl fmt::Display for SectionAlreadyCreatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Can't create section {} ({:?} / {:?}) because it has been created already.",
            self.name, self.rom, self.vram
        )
    }
}
impl error::Error for SectionAlreadyCreatedError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct SectionNotPreheatedError {
    name: Arc<str>,
    rom: Rom,
    vram: Vram,
}
impl SectionNotPreheatedError {
    pub(crate) fn new(name: Arc<str>, rom: Rom, vram: Vram) -> Self {
        Self { name, rom, vram }
    }
}
impl fmt::Display for SectionNotPreheatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Can't create section {} ({:?} / {:?}) because it wasn't preheated.",
            self.name, self.rom, self.vram
        )
    }
}
impl error::Error for SectionNotPreheatedError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, SectionCreationError, PyRuntimeError);

    impl std::convert::From<super::SectionCreationError> for PyErr {
        fn from(err: super::SectionCreationError) -> PyErr {
            SectionCreationError::new_err(err.to_string())
        }
    }
}
