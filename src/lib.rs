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
mod matrix;
mod options;

pub use column::{Column, VecColumn};
pub use decomposition::{rv_decompose, PersistenceDiagram, RVDecomposition};
pub use lock_free::rv_decompose_lock_free;
pub use matrix::*;
pub use options::LoPhatOptions;

fn matrix_from_pyiterator<'a>(matrix: &'a PyIterator) -> impl Iterator<Item = VecColumn> + 'a {
    matrix.map(|col| {
        col.and_then(PyAny::extract::<Vec<usize>>)
            .map(VecColumn::from)
            .expect("Column is a list of unsigned integers")
    })
}

#[pyfunction]
#[pyo3(signature = (matrix))]
fn compute_pairings_serial(matrix: &PyIterator) -> PersistenceDiagram {
    let options = LoPhatOptions {
        maintain_v: false,
        num_threads: 1,
    };
    rv_decompose(matrix_from_pyiterator(matrix), options).diagram()
}

#[pyfunction]
#[pyo3(signature = (matrix, num_threads=0))]
fn compute_pairings(matrix: &PyIterator, num_threads: usize) -> PersistenceDiagram {
    let options = LoPhatOptions {
        maintain_v: false,
        num_threads,
    };
    rv_decompose_lock_free(matrix_from_pyiterator(matrix), None, options).diagram()
}

/// A Python module implemented in Rust.
#[pymodule]
fn lophat(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_function(wrap_pyfunction!(compute_pairings_serial, m)?)?;
    Ok(())
}
