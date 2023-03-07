use column::VecColumn;
use pyo3::prelude::*;
use pyo3::types::PyIterator;

mod column;
mod decomposition;
mod lock_free;

pub use column::Column;
pub use decomposition::{rv_decompose, PersistenceDiagram, RVDecomposition};
pub use lock_free::rv_decompose_lock_free;

#[pyfunction]
fn compute_pairings_serial(matrix: &PyIterator) -> PersistenceDiagram {
    rv_decompose(matrix.into_iter().map(|col| {
        VecColumn {
            internal: col
                .and_then(PyAny::extract::<Vec<usize>>)
                .expect("Column is a vector of unsigned integers"),
        }
    }))
    .diagram()
}

#[pyfunction]
fn compute_pairings(matrix: &PyIterator) -> PersistenceDiagram {
    rv_decompose_lock_free(matrix.into_iter().map(|col| {
        VecColumn {
            internal: col
                .and_then(PyAny::extract::<Vec<usize>>)
                .expect("Column is a vector of unsigned integers"),
        }
    }))
    .diagram()
}

/// A Python module implemented in Rust.
#[pymodule]
fn lophat(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_function(wrap_pyfunction!(compute_pairings_serial, m)?)?;
    Ok(())
}
