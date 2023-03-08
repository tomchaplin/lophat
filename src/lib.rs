//! # LoPHAT
//! LoPHAT implements algorithms for computing persistent homology, in parallel, in a lockfree fashion.
//! The primary interface with this crate is the [`rv_decompose_lock_free`] function.
//! We also implement the traditional, serial algorithm in [`rv_decompose`].
//!
//! Both functions accept an iterator over columns.
//! These can have any representation in memory, so long as they impelement the [`Column`] trait.
//! For many applications, [`VecColumn`] is a good choice for your columns.
//!
//! Both functions return a [`RVDecomposition`], from which you can read off the persistence diagram.

use pyo3::prelude::*;
use pyo3::types::PyIterator;

mod column;
mod decomposition;
mod lock_free;

pub use column::{Column, VecColumn};
pub use decomposition::{rv_decompose, PersistenceDiagram, RVDecomposition};
pub use lock_free::rv_decompose_lock_free;

#[pyfunction]
fn compute_pairings_serial(matrix: &PyIterator) -> PersistenceDiagram {
    rv_decompose(matrix.map(|col| {
        VecColumn {
            internal: col
                .and_then(PyAny::extract::<Vec<usize>>)
                .expect("Column is a list of unsigned integers"),
        }
    }))
    .diagram()
}

#[pyfunction]
fn compute_pairings(matrix: &PyIterator) -> PersistenceDiagram {
    rv_decompose_lock_free(
        matrix.map(|col| VecColumn {
            internal: col
                .and_then(PyAny::extract::<Vec<usize>>)
                .expect("Column is a list of unsigned integers"),
        }),
        None,
    )
    .diagram()
}

/// A Python module implemented in Rust.
#[pymodule]
fn lophat(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_function(wrap_pyfunction!(compute_pairings_serial, m)?)?;
    Ok(())
}
