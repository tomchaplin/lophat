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

mod anti_transpose;
mod column;
mod decomposition;
mod diagram;
mod indexing;
mod lock_free;
mod locking;
mod options;

pub use crate::column::{BitSetColumn, Column, VecColumn};
pub use crate::decomposition::{rv_decompose_serial, RVDecomposition};
pub use crate::diagram::{DiagramReadOff, PersistenceDiagram};
//pub use crate::indexing::{IndexMap, VecIndexMap};
pub use crate::anti_transpose::*;
pub use crate::lock_free::{rv_decompose_lock_free, LockFreeAlgorithm};
pub use crate::locking::{rv_decompose_locking, LockingAlgorithm};
pub use crate::options::LoPhatOptions;

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
        rv_decompose_lock_free(matrix, options).into()
    }
}

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::PyIterator;

#[cfg(feature = "python")]
fn compute_pairings_rs<C: Column + 'static>(
    matrix: impl Iterator<Item = C>,
    options: LoPhatOptions,
) -> PersistenceDiagram {
    if options.num_threads == 1 {
        rv_decompose_serial(matrix, options).diagram()
    } else {
        rv_decompose_lock_free(matrix, options).diagram()
    }
}

#[cfg(feature = "python")]
fn compute_pairings_anti_transpose(
    py: Python<'_>,
    matrix: &PyAny,
    options: Option<LoPhatOptions>,
) -> PersistenceDiagram {
    let options = options.unwrap_or(LoPhatOptions::default());
    let matrix_as_vec: Vec<_> =
        if let Ok(matrix_as_vec) = matrix.extract::<Vec<(usize, Vec<usize>)>>() {
            matrix_as_vec.into_iter().map(VecColumn::from).collect()
        } else if let Ok(py_iter) = PyIterator::from_object(py, matrix) {
            py_iter
                .map(|col| {
                    col.and_then(PyAny::extract::<(usize, Vec<usize>)>)
                        .map(VecColumn::from)
                        .expect("Column is a list of unsigned integers")
                })
                .collect()
        } else {
            panic!("Could not coerce input matrix into List[List[int]] | Iterator[List[int]]");
        };
    let width = matrix_as_vec.len();
    let at: Vec<_> = anti_transpose(&matrix_as_vec);
    let dgm = compute_pairings_rs(at.into_iter(), options);
    anti_transpose_diagram(dgm, width)
}

#[cfg(feature = "python")]
fn compute_pairings_non_transpose(
    py: Python<'_>,
    matrix: &PyAny,
    options: Option<LoPhatOptions>,
) -> PersistenceDiagram {
    let options = options.unwrap_or(LoPhatOptions::default());
    if let Ok(matrix_as_vec) = matrix.extract::<Vec<(usize, Vec<usize>)>>() {
        let matrix_as_rs_iter = matrix_as_vec.into_iter().map(VecColumn::from);
        compute_pairings_rs(matrix_as_rs_iter, options)
    } else if let Ok(py_iter) = PyIterator::from_object(py, matrix) {
        let matrix_as_rs_iter = py_iter.map(|col| {
            col.and_then(PyAny::extract::<(usize, Vec<usize>)>)
                .map(VecColumn::from)
                .expect("Column is a list of unsigned integers")
        });
        compute_pairings_rs(matrix_as_rs_iter, options)
    } else {
        panic!("Could not coerce input matrix into List[List[int]] | Iterator[List[int]]");
    }
}

#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (matrix,anti_transpose= true, options=None))]
fn compute_pairings(
    py: Python<'_>,
    matrix: &PyAny,
    anti_transpose: bool,
    options: Option<LoPhatOptions>,
) -> PersistenceDiagram {
    if anti_transpose {
        compute_pairings_anti_transpose(py, matrix, options)
    } else {
        compute_pairings_non_transpose(py, matrix, options)
    }
}

/// A Python module implemented in Rust.
#[pymodule]
#[cfg(feature = "python")]
fn lophat(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_class::<LoPhatOptions>()?;
    Ok(())
}
