/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

use address_space::{AddressRange, Rom, Vram};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

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
            "Unable to add overlay segment '{}' to builder because: ",
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

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddOverlayToBuilderError, PyRuntimeError);

    impl std::convert::From<super::AddOverlayToBuilderError> for PyErr {
        fn from(err: super::AddOverlayToBuilderError) -> PyErr {
            AddOverlayToBuilderError::new_err(err.to_string())
        }
    }
}
