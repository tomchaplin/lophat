//! # LoPHAT
//! LoPHAT implements algorithms for computing persistent homology, in parallel, in a lockfree fashion.
//! The primary interface with this crate is the [`rv_decompose`] function.
//! This function accepts an iterator over columns.
//! These can have any representation in memory, so long as they impelement the [`Column`] trait.
//! For many applications, [`VecColumn`] is a good choice for your columns.
//! The function returns a [`RVDecomposition`], from which you can read off the persistence diagram.
//! Additional options can be set by providing a [`LoPhatOptions`] struct.
//!
//! If you would like to force the use of the serial or parallel algorithm,
//! use [`rv_decompose_serial`] or [`rv_decompose_lock_free`] respectively.

use pyo3::prelude::*;
use pyo3::types::PyIterator;

mod column;
mod decomposition;
mod lock_free;
mod matrix;
mod options;

pub use column::{Column, VecColumn};
pub use decomposition::{rv_decompose_serial, PersistenceDiagram, RVDecomposition};
pub use lock_free::rv_decompose_lock_free;
pub use matrix::*;
pub use options::LoPhatOptions;

/// Decomposes the input matrix, choosing between the serial and parallel
/// algorithms depending on `options.num_threads`.
///
/// * `matrix` - iterator over columns of the matrix you wish to decompose.
/// * `options` - additional options to control decompositon, see [`LoPhatOptions`].
pub fn rv_decompose<C: Column + 'static>(
    matrix: impl Iterator<Item = C>,
    options: LoPhatOptions,
) -> RVDecomposition<C> {
    if options.num_threads == 1 {
        rv_decompose_serial(matrix, options)
    } else {
        rv_decompose_lock_free(matrix, options)
    }
}

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
        column_height: None,
        num_threads: 1,
        min_chunk_len: 1,
    };
    rv_decompose_serial(matrix_from_pyiterator(matrix), options).diagram()
}

#[pyfunction]
#[pyo3(signature = (matrix, num_threads=0))]
fn compute_pairings_lock_free(matrix: &PyIterator, num_threads: usize) -> PersistenceDiagram {
    let options = LoPhatOptions {
        maintain_v: false,
        column_height: None,
        num_threads,
        min_chunk_len: 1,
    };
    rv_decompose_lock_free(matrix_from_pyiterator(matrix), options).diagram()
}

#[pyfunction]
#[pyo3(signature = (matrix, options=None))]
fn compute_pairings(matrix: &PyIterator, options: Option<LoPhatOptions>) -> PersistenceDiagram {
    let options = options.unwrap_or(LoPhatOptions {
        maintain_v: false,
        num_threads: 0,
        column_height: None,
        min_chunk_len: 1,
    });
    rv_decompose(matrix_from_pyiterator(matrix), options).diagram()
}

/// A Python module implemented in Rust.
#[pymodule]
fn lophat(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings_lock_free, m)?)?;
    m.add_function(wrap_pyfunction!(compute_pairings_serial, m)?)?;
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_class::<LoPhatOptions>()?;
    Ok(())
}
