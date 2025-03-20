/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, Size, Vram};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum PreheatErrorInner {
    WrongRom {
        segment_range: AddressRange<Rom>,
        section_end: Rom,
    },
    WrongVram {
        segment_range: AddressRange<Vram>,
        section_end: Vram,
    },
    AlreadyPreheated,
    OverlapsWithAlreadyPreheated {
        other_name: Arc<str>,
        other_range: AddressRange<Vram>,
    },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct PreheatError {
    segment_name: Option<Arc<str>>,
    section_name: Arc<str>,
    section_rom: Rom,
    section_vram: Vram,
    inner: PreheatErrorInner,
}

impl PreheatError {
    pub(crate) const fn new_wrong_rom(
        segment_name: Option<Arc<str>>,
        section_name: Arc<str>,
        section_rom: Rom,
        section_vram: Vram,
        segment_range: AddressRange<Rom>,
        section_end: Rom,
    ) -> Self {
        Self {
            segment_name,
            section_name,
            section_rom,
            section_vram,
            inner: PreheatErrorInner::WrongRom {
                segment_range,
                section_end,
            },
        }
    }
    pub(crate) const fn new_wrong_vram(
        segment_name: Option<Arc<str>>,
        section_name: Arc<str>,
        section_rom: Rom,
        section_vram: Vram,
        segment_range: AddressRange<Vram>,
        section_end: Vram,
    ) -> Self {
        Self {
            segment_name,
            section_name,
            section_rom,
            section_vram,
            inner: PreheatErrorInner::WrongVram {
                segment_range,
                section_end,
            },
        }
    }
    pub(crate) const fn new_already_preheated(
        segment_name: Option<Arc<str>>,
        section_name: Arc<str>,
        section_rom: Rom,
        section_vram: Vram,
    ) -> Self {
        Self {
            segment_name,
            section_name,
            section_rom,
            section_vram,
            inner: PreheatErrorInner::AlreadyPreheated,
        }
    }
    pub(crate) fn new_overlaps_with_already_preheated(
        segment_name: Option<Arc<str>>,
        section_name: Arc<str>,
        section_rom: Rom,
        section_vram: Vram,
        other_name: Arc<str>,
        other_vram: Vram,
        other_size: Size,
    ) -> Self {
        Self {
            segment_name,
            section_name,
            section_rom,
            section_vram,
            inner: PreheatErrorInner::OverlapsWithAlreadyPreheated {
                other_name,
                other_range: AddressRange::new(other_vram, other_vram + other_size),
            },
        }
    }
}

impl fmt::Display for PreheatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while preheating the section '{}' ({:?} / {:?}) from the ",
            self.section_name, self.section_rom, self.section_vram
        )?;
        if let Some(name) = &self.segment_name {
            write!(f, "overlay segment '{}' ", name)?;
        } else {
            write!(f, "global segment ")?;
        }
        write!(f, ": ")?;

        match &self.inner {
            PreheatErrorInner::WrongRom { segment_range, section_end } => write!(f, "This section does not belong to this segment, since its rom ranges ({:?} ~ {:?}) are outside of the segment's ranges ({:?} ~ {:?})", self.section_rom, section_end, segment_range.start(), segment_range.end()),
            PreheatErrorInner::WrongVram { segment_range, section_end } => write!(f, "This section does not belong to this segment, since its vram ranges ({:?} ~ {:?}) are outside of the segment's ranges ({:?} ~ {:?})", self.section_vram, section_end, segment_range.start(), segment_range.end()),
            PreheatErrorInner::AlreadyPreheated => write!(f, "This section has been preheated already"),
            PreheatErrorInner::OverlapsWithAlreadyPreheated {
                other_name,
                other_range,
            } => write!(f, "This section's vram overlaps with the vram range of the section '{}', which has a vram of 0x{} ~ {}", other_name, other_range.start(), other_range.end()),
        }
    }
}
impl error::Error for PreheatError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, PreheatError, PyRuntimeError);

    impl std::convert::From<super::PreheatError> for PyErr {
        fn from(err: super::PreheatError) -> PyErr {
            PreheatError::new_err(err.to_string())
        }
    }
}
