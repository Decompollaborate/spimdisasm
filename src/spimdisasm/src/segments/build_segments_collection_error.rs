/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::sync::Arc;
use core::{error, fmt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum BuildSegmentsCollectionErrorInner {
    ZeroGlobalSegments,
    PrioritisedOverlayNotFound(Arc<str>, Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildSegmentsCollectionError {
    inner: BuildSegmentsCollectionErrorInner,
}

impl BuildSegmentsCollectionError {
    pub(crate) const fn new_zero_global_segments() -> Self {
        Self {
            inner: BuildSegmentsCollectionErrorInner::ZeroGlobalSegments,
        }
    }

    pub(crate) const fn new_missing_prioritised_overlay(
        segment_name: Arc<str>,
        prioritised_overlay_name: Arc<str>,
    ) -> Self {
        Self {
            inner: BuildSegmentsCollectionErrorInner::PrioritisedOverlayNotFound(
                segment_name,
                prioritised_overlay_name,
            ),
        }
    }
}
impl fmt::Display for BuildSegmentsCollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failure while building the SegmentsCollections: ")?;
        match &self.inner {
            BuildSegmentsCollectionErrorInner::ZeroGlobalSegments => {
                write!(
                    f,
                    "At least one global segment must be added to the builder before building it."
                )
            }
            BuildSegmentsCollectionErrorInner::PrioritisedOverlayNotFound(
                segment_name,
                prioritised_overlay_name,
            ) => {
                write!(f, "The segment '{segment_name}' references the prioritised overlay segment '{prioritised_overlay_name}', but such name was not found in any overlay segment")
            }
        }
    }
}
impl error::Error for BuildSegmentsCollectionError {}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, BuildSegmentsCollectionError, PyRuntimeError);

    impl std::convert::From<super::BuildSegmentsCollectionError> for PyErr {
        fn from(err: super::BuildSegmentsCollectionError) -> PyErr {
            BuildSegmentsCollectionError::new_err(err.to_string())
        }
    }
}
