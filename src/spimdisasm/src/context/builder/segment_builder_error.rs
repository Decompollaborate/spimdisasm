/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use super::SegmentBuilderKind;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum AddPrioritisedOverlayErrorInner {
    SameNameAsCurrentOverlay,
    DuplicatedPrioritised { overlay_name: Arc<str> },
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm", from_py_object))]
pub struct AddPrioritisedOverlayError {
    segment_kind: SegmentBuilderKind,
    inner: AddPrioritisedOverlayErrorInner,
}

impl AddPrioritisedOverlayError {
    pub(crate) const fn new_self_name(segment_kind: SegmentBuilderKind) -> Self {
        Self {
            segment_kind,
            inner: AddPrioritisedOverlayErrorInner::SameNameAsCurrentOverlay,
        }
    }

    pub(crate) const fn new_duplicated(
        segment_kind: SegmentBuilderKind,
        overlay_name: Arc<str>,
    ) -> Self {
        Self {
            segment_kind,
            inner: AddPrioritisedOverlayErrorInner::DuplicatedPrioritised { overlay_name },
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
            AddPrioritisedOverlayErrorInner::SameNameAsCurrentOverlay => {
                write!(f, "Trying to add a prioritised overlay to itself (",)?;
                self.segment_kind.write_verbose(f)?;
                write!(f, ").")
            }
            AddPrioritisedOverlayErrorInner::DuplicatedPrioritised { overlay_name } => {
                write!(
                    f,
                    "Trying to add duplicated prioritised overlay name '{}' to ",
                    overlay_name,
                )?;
                self.segment_kind.write_verbose(f)
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
