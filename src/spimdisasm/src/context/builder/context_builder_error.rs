/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::addresses::{AddressRange, Rom, Vram};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum AddGlobalToBuilderErrorInner {
    GlobalOverlappingRom(AddressRange<Rom>, Arc<str>, AddressRange<Rom>),
    GlobalOverlappingVram(AddressRange<Vram>, Arc<str>, AddressRange<Vram>),
    OverlayOverlappingRom(AddressRange<Rom>, Arc<str>, AddressRange<Rom>),
    OverlayOverlappingVram(AddressRange<Vram>, Arc<str>, AddressRange<Vram>),
    DuplicatedName,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct AddGlobalToBuilderError {
    segment_name: Arc<str>,
    inner: AddGlobalToBuilderErrorInner,
}

impl AddGlobalToBuilderError {
    pub(crate) const fn new_global_overlapping_rom(
        segment_name: Arc<str>,
        self_range: AddressRange<Rom>,
        other_name: Arc<str>,
        other_range: AddressRange<Rom>,
    ) -> Self {
        Self {
            segment_name,
            inner: AddGlobalToBuilderErrorInner::GlobalOverlappingRom(
                self_range,
                other_name,
                other_range,
            ),
        }
    }
    pub(crate) const fn new_global_overlapping_vram(
        segment_name: Arc<str>,
        self_range: AddressRange<Vram>,
        other_name: Arc<str>,
        other_range: AddressRange<Vram>,
    ) -> Self {
        Self {
            segment_name,
            inner: AddGlobalToBuilderErrorInner::GlobalOverlappingVram(
                self_range,
                other_name,
                other_range,
            ),
        }
    }
    pub(crate) const fn new_overlay_overlapping_rom(
        segment_name: Arc<str>,
        self_range: AddressRange<Rom>,
        other_name: Arc<str>,
        other_range: AddressRange<Rom>,
    ) -> Self {
        Self {
            segment_name,
            inner: AddGlobalToBuilderErrorInner::OverlayOverlappingRom(
                self_range,
                other_name,
                other_range,
            ),
        }
    }
    pub(crate) const fn new_overlay_overlapping_vram(
        segment_name: Arc<str>,
        self_range: AddressRange<Vram>,
        other_name: Arc<str>,
        other_range: AddressRange<Vram>,
    ) -> Self {
        Self {
            segment_name,
            inner: AddGlobalToBuilderErrorInner::OverlayOverlappingVram(
                self_range,
                other_name,
                other_range,
            ),
        }
    }
    pub(crate) const fn new_duplicated_name(segment_name: Arc<str>) -> Self {
        Self {
            segment_name,
            inner: AddGlobalToBuilderErrorInner::DuplicatedName,
        }
    }
}
impl fmt::Display for AddGlobalToBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unable to add global segment '{}' to the context builder because: ",
            self.segment_name
        )?;
        match &self.inner {
            AddGlobalToBuilderErrorInner::GlobalOverlappingRom(
                self_range,
                other_name,
                other_range,
            ) => {
                write!(f, "Its Rom range ({self_range:?}) overlaps with the Rom address range of the global segment {other_name} ({other_range:?}).")
            }
            AddGlobalToBuilderErrorInner::GlobalOverlappingVram(
                self_range,
                other_name,
                other_range,
            ) => {
                write!(f, "Its Vram range ({self_range:?}) overlaps with the Vram address range of the global segment {other_name} ({other_range:?}).")
            }
            AddGlobalToBuilderErrorInner::OverlayOverlappingRom(
                self_range,
                other_name,
                other_range,
            ) => {
                write!(f, "Its Rom range ({self_range:?}) overlaps with the Rom address range of the overlay segment {other_name} ({other_range:?}).")
            }
            AddGlobalToBuilderErrorInner::OverlayOverlappingVram(
                self_range,
                other_name,
                other_range,
            ) => {
                write!(f, "Its Vram range ({self_range:?}) overlaps with the Vram address range of the overlay segment {other_name} ({other_range:?}).")
            }
            AddGlobalToBuilderErrorInner::DuplicatedName => {
                write!(f, "Its name is already used by other segment.")
            }
        }
    }
}
impl error::Error for AddGlobalToBuilderError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum AddOverlayToBuilderErrorInner {
    GlobalOverlappingRom(AddressRange<Rom>, Arc<str>, AddressRange<Rom>),
    GlobalOverlappingVram(AddressRange<Vram>, Arc<str>, AddressRange<Vram>),
    DuplicatedName,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct AddOverlayToBuilderError {
    overlay_name: Arc<str>,
    inner: AddOverlayToBuilderErrorInner,
}

impl AddOverlayToBuilderError {
    pub(crate) const fn new_overlapping_rom(
        overlay_name: Arc<str>,
        self_range: AddressRange<Rom>,
        other_name: Arc<str>,
        other_range: AddressRange<Rom>,
    ) -> Self {
        Self {
            overlay_name,
            inner: AddOverlayToBuilderErrorInner::GlobalOverlappingRom(
                self_range,
                other_name,
                other_range,
            ),
        }
    }
    pub(crate) const fn new_overlapping_vram(
        overlay_name: Arc<str>,
        self_range: AddressRange<Vram>,
        other_name: Arc<str>,
        other_range: AddressRange<Vram>,
    ) -> Self {
        Self {
            overlay_name,
            inner: AddOverlayToBuilderErrorInner::GlobalOverlappingVram(
                self_range,
                other_name,
                other_range,
            ),
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
            "Unable to add overlay segment '{}' to the context builder because: ",
            self.overlay_name
        )?;
        match &self.inner {
            AddOverlayToBuilderErrorInner::GlobalOverlappingRom(
                self_range,
                other_name,
                other_range,
            ) => {
                write!(f, "Its Rom range ({self_range:?}) overlaps with the Rom address range of the overlay segment {other_name} ({other_range:?}).")
            }
            AddOverlayToBuilderErrorInner::GlobalOverlappingVram(
                self_range,
                other_name,
                other_range,
            ) => {
                write!(f, "Its Vram range ({self_range:?}) overlaps with the Vram address range of the overlay segment {other_name} ({other_range:?}).")
            }
            AddOverlayToBuilderErrorInner::DuplicatedName => {
                write!(f, "Its name is already used by other overlay segment.")
            }
        }
    }
}
impl error::Error for AddOverlayToBuilderError {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
enum BuildContextErrorInner {
    ZeroGlobalSegments,
    PrioritisedOverlayNotFound(Arc<str>, Arc<str>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildContextError {
    inner: BuildContextErrorInner,
}

impl BuildContextError {
    pub(crate) const fn new_zero_global_segments() -> Self {
        Self {
            inner: BuildContextErrorInner::ZeroGlobalSegments,
        }
    }

    pub(crate) const fn new_missing_prioritised_overlay(
        segment_name: Arc<str>,
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
            BuildContextErrorInner::ZeroGlobalSegments => {
                write!(f, "At least one global segment must be added to the ContextBuilder before building it.")
            }
            BuildContextErrorInner::PrioritisedOverlayNotFound(
                segment_name,
                prioritised_overlay_name,
            ) => {
                write!(f, "The segment '{segment_name}' references the prioritised overlay segment '{prioritised_overlay_name}', but such name was not found in any overlay segment")
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

    pyo3::create_exception!(spimdisasm, AddGlobalToBuilderError, PyRuntimeError);
    pyo3::create_exception!(spimdisasm, AddOverlayToBuilderError, PyRuntimeError);
    pyo3::create_exception!(spimdisasm, BuildContextError, PyRuntimeError);

    impl std::convert::From<super::AddGlobalToBuilderError> for PyErr {
        fn from(err: super::AddGlobalToBuilderError) -> PyErr {
            AddGlobalToBuilderError::new_err(err.to_string())
        }
    }
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
