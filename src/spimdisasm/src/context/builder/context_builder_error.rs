/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, Vram};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum AddOverlayToBuilderErrorInner {
    GlobalOverlappingRom(AddressRange<Rom>, AddressRange<Rom>),
    GlobalOverlappingVram(AddressRange<Vram>, AddressRange<Vram>),
    DuplicatedName,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct AddOverlayToBuilderError {
    overlay_name: Arc<str>,
    inner: AddOverlayToBuilderErrorInner,
}

impl AddOverlayToBuilderError {
    pub(crate) const fn new_overlapping_rom(
        overlay_name: Arc<str>,
        ovl_range: AddressRange<Rom>,
        global_range: AddressRange<Rom>,
    ) -> Self {
        Self {
            overlay_name,
            inner: AddOverlayToBuilderErrorInner::GlobalOverlappingRom(ovl_range, global_range),
        }
    }
    pub(crate) const fn new_overlapping_vram(
        overlay_name: Arc<str>,
        ovl_range: AddressRange<Vram>,
        global_range: AddressRange<Vram>,
    ) -> Self {
        Self {
            overlay_name,
            inner: AddOverlayToBuilderErrorInner::GlobalOverlappingVram(ovl_range, global_range),
        }
    }
    pub(crate) const fn new_duplicated_name(overlay_name: Arc<str>) -> Self {
        Self {
            overlay_name,
            inner: AddOverlayToBuilderErrorInner::DuplicatedName,
        }
    }
}
impl fmt::Display for AddOverlayToBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unable to add overlay segment '{}' to context builder because: ",
            self.overlay_name
        )?;
        match &self.inner {
            AddOverlayToBuilderErrorInner::GlobalOverlappingRom(ovl_range, global_range) => write!(f, "Its Rom range ({ovl_range:?}) overlaps with the global segment's Rom address range ({global_range:?})."),
            AddOverlayToBuilderErrorInner::GlobalOverlappingVram(ovl_range, global_range) => write!(f, "Its Vram range ({ovl_range:?}) overlaps with the global segment's Vram address range ({global_range:?})."),
            AddOverlayToBuilderErrorInner::DuplicatedName => write!(f, "Its name is already used by other overlay segment."),
        }
    }
}
impl error::Error for AddOverlayToBuilderError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum BuildContextErrorInner {
    PrioritisedOverlayNotFound(Option<Arc<str>>, Arc<str>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildContextError {
    inner: BuildContextErrorInner,
}

impl BuildContextError {
    pub(crate) const fn new_missing_prioritised_overlay(
        segment_name: Option<Arc<str>>,
        prioritised_overlay_name: Arc<str>,
    ) -> Self {
        Self {
            inner: BuildContextErrorInner::PrioritisedOverlayNotFound(
                segment_name,
                prioritised_overlay_name,
            ),
        }
    }
}
impl fmt::Display for BuildContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failure during Context building: ")?;
        match &self.inner {
            BuildContextErrorInner::PrioritisedOverlayNotFound(
                segment_name,
                prioritised_overlay_name,
            ) => {
                match segment_name {
                    None => write!(f, "The global segment ")?,
                    Some(x) => write!(f, "The overlay '{x}' ")?,
                }
                write!(f, "references the prioritised overlay segment '{prioritised_overlay_name}', but such name was not found in any overlay segment")
            }
        }
    }
}
impl error::Error for BuildContextError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddOverlayToBuilderError, PyRuntimeError);
    pyo3::create_exception!(spimdisasm, BuildContextError, PyRuntimeError);

    impl std::convert::From<super::AddOverlayToBuilderError> for PyErr {
        fn from(err: super::AddOverlayToBuilderError) -> PyErr {
            AddOverlayToBuilderError::new_err(err.to_string())
        }
    }
    impl std::convert::From<super::BuildContextError> for PyErr {
        fn from(err: super::BuildContextError) -> PyErr {
            BuildContextError::new_err(err.to_string())
        }
    }
}
