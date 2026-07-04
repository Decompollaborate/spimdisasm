/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

use address_space::{AddressRange, Rom, Vram};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum AddGlobalToBuilderErrorInner {
    GlobalOverlappingRom(AddressRange<Rom>, Arc<str>, AddressRange<Rom>),
    GlobalOverlappingVram(AddressRange<Vram>, Arc<str>, AddressRange<Vram>),
    OverlayOverlappingRom(AddressRange<Rom>, Arc<str>, AddressRange<Rom>),
    OverlayOverlappingVram(AddressRange<Vram>, Arc<str>, AddressRange<Vram>),
    DuplicatedName,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            "Unable to add global segment '{}' to builder because: ",
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

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddGlobalToBuilderError, PyRuntimeError);

    impl std::convert::From<super::AddGlobalToBuilderError> for PyErr {
        fn from(err: super::AddGlobalToBuilderError) -> PyErr {
            AddGlobalToBuilderError::new_err(err.to_string())
        }
    }
}
