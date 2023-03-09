use ::lophat::{rv_decompose, rv_decompose_lock_free, PersistenceDiagram, VecColumn};
use pyo3::prelude::*;
use pyo3::types::PyIterator;

#[pyfunction]
fn compute_pairings_serial(matrix: &PyIterator) -> PersistenceDiagram {
    rv_decompose(matrix.map(|col| {
        col.and_then(PyAny::extract::<Vec<usize>>)
            .map(VecColumn::from)
            .expect("Column is a list of unsigned integers")
    }))
    .diagram()
}

#[pyfunction]
fn compute_pairings(matrix: &PyIterator) -> PersistenceDiagram {
    rv_decompose_lock_free(
        matrix.map(|col| {
            col.and_then(PyAny::extract::<Vec<usize>>)
                .map(VecColumn::from)
                .expect("Column is a list of unsigned integers")
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
