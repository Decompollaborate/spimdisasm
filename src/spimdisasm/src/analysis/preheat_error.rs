/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{error, fmt};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct PreheatError {}
impl fmt::Display for PreheatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n\n")?;
        writeln!(f, " ================================== ")?;
        writeln!(f, "| This should be unreachable code! |")?;
        writeln!(f, "| How did you trigger this??       |")?;
        writeln!(f, "|                                  |")?;
        writeln!(f, "| How ahead and implement me!      |")?;
        writeln!(f, "| I live in file:                  |",)?;
        writeln!(f, "| {} |", file!())?;
        writeln!(f, " ================================== ")?;
        writeln!(f, "\n\n")?;
        todo!()
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
