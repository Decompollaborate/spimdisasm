/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum AddPrioritisedOverlayErrorInner {
    SameNameAsCurrentOverlay,
    DuplicatedPrioritised,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct AddPrioritisedOverlayError {
    segment_name: Option<Arc<str>>,
    overlay_name: Arc<str>,
    inner: AddPrioritisedOverlayErrorInner,
}

impl AddPrioritisedOverlayError {
    pub(crate) const fn new_self_name(
        segment_name: Option<Arc<str>>,
        overlay_name: Arc<str>,
    ) -> Self {
        Self {
            segment_name,
            overlay_name,
            inner: AddPrioritisedOverlayErrorInner::SameNameAsCurrentOverlay,
        }
    }

    pub(crate) const fn new_duplicated(
        segment_name: Option<Arc<str>>,
        overlay_name: Arc<str>,
    ) -> Self {
        Self {
            segment_name,
            overlay_name,
            inner: AddPrioritisedOverlayErrorInner::DuplicatedPrioritised,
        }
    }
}
impl fmt::Display for AddPrioritisedOverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error while trying to add a prioritised overlay to a segment: "
        )?;
        match &self.inner {
            AddPrioritisedOverlayErrorInner::SameNameAsCurrentOverlay => write!(
                f,
                "Trying to add a prioritised overlay to itself (segment {}).",
                self.overlay_name
            ),
            AddPrioritisedOverlayErrorInner::DuplicatedPrioritised => {
                write!(
                    f,
                    "Trying to add duplicated prioritised overlay name '{}' to ",
                    self.overlay_name
                )?;
                if let Some(name) = &self.segment_name {
                    write!(f, "overlay segment '{name}'")
                } else {
                    write!(f, "the global segment")
                }
            }
        }
    }
}
impl error::Error for AddPrioritisedOverlayError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, AddPrioritisedOverlayError, PyRuntimeError);

    impl std::convert::From<super::AddPrioritisedOverlayError> for PyErr {
        fn from(err: super::AddPrioritisedOverlayError) -> PyErr {
            AddPrioritisedOverlayError::new_err(err.to_string())
        }
    }
}
