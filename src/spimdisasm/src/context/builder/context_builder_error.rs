/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

use crate::segments::BuildSegmentsCollectionError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum BuildContextErrorInner {
    BuildSegmentsCollection(BuildSegmentsCollectionError),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildContextError {
    inner: BuildContextErrorInner,
}

impl BuildContextError {
    pub(crate) const fn new_zero_global_segments() -> Self {
        Self {
            inner: BuildContextErrorInner::BuildSegmentsCollection(
                BuildSegmentsCollectionError::new_zero_global_segments(),
            ),
        }
    }
}

impl fmt::Display for BuildContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failure during Context building: ")?;
        match &self.inner {
            BuildContextErrorInner::BuildSegmentsCollection(build_segments_collection_error) => {
                write!(f, "{}", build_segments_collection_error)
            }
        }
    }
}

impl error::Error for BuildContextError {}

impl From<BuildSegmentsCollectionError> for BuildContextError {
    fn from(value: BuildSegmentsCollectionError) -> Self {
        Self {
            inner: BuildContextErrorInner::BuildSegmentsCollection(value),
        }
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    // TODO: make a generic spimdisasm exception and make every other error to inherit from it

    pyo3::create_exception!(spimdisasm, BuildContextError, PyRuntimeError);

    impl std::convert::From<super::BuildContextError> for PyErr {
        fn from(err: super::BuildContextError) -> PyErr {
            BuildContextError::new_err(err.to_string())
        }
    }
}
