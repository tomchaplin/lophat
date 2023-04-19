use pyo3::prelude::*;
use pyo3::types::PyIterator;

use crate::algorithms::{LockFreeAlgorithm, RVDecomposition, SerialAlgorithm};
use crate::columns::{Column, VecColumn};
use crate::options::LoPhatOptions;
use crate::utils::{anti_transpose, PersistenceDiagram};

fn compute_pairings_rs<C: Column + 'static>(
    matrix: impl Iterator<Item = C>,
    options: LoPhatOptions,
) -> PersistenceDiagram {
    if options.num_threads == 1 {
        SerialAlgorithm::decompose(matrix, options).diagram()
    } else {
        LockFreeAlgorithm::decompose(matrix, options).diagram()
    }
}

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
    dgm.anti_transpose(width)
}

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

// A Python module implemented in Rust.
#[pymodule]
fn lophat(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_class::<LoPhatOptions>()?;
    Ok(())
}