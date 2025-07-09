use pyo3::prelude::*;
use pyo3::types::PyIterator;

use crate::algorithms::{Decomposition, DecompositionAlgo, LockFreeAlgorithm};
use crate::columns::Column;
use crate::columns::VecColumn;
use crate::options::LoPhatOptions;
use crate::utils::{anti_transpose, PersistenceDiagram};

fn compute_pairings_anti_transpose(
    py: Python<'_>,
    matrix: &PyAny,
    options: Option<LoPhatOptions>,
) -> PersistenceDiagram {
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
    let dgm = {
        let matrix = at.into_iter();
        LockFreeAlgorithm::init(options)
            .add_cols(matrix)
            .decompose()
            .diagram()
    };
    dgm.anti_transpose(width)
}

fn compute_pairings_non_transpose(
    py: Python<'_>,
    matrix: &PyAny,
    options: Option<LoPhatOptions>,
) -> PersistenceDiagram {
    if let Ok(matrix_as_vec) = matrix.extract::<Vec<(usize, Vec<usize>)>>() {
        let matrix_as_rs_iter = matrix_as_vec.into_iter().map(VecColumn::from);
        LockFreeAlgorithm::init(options)
            .add_cols(matrix_as_rs_iter)
            .decompose()
            .diagram()
    } else if let Ok(py_iter) = PyIterator::from_object(py, matrix) {
        let matrix_as_rs_iter = py_iter.map(|col| {
            col.and_then(PyAny::extract::<(usize, Vec<usize>)>)
                .map(VecColumn::from)
                .expect("Column is a list of unsigned integers")
        });
        LockFreeAlgorithm::init(options)
            .add_cols(matrix_as_rs_iter)
            .decompose()
            .diagram()
    } else {
        panic!("Could not coerce input matrix into List[List[int]] | Iterator[List[int]]");
    }
}

#[pyclass(get_all, set_all)]
struct PersistenceDiagramWithReps {
    paired: Vec<(usize, usize)>,
    unpaired: Vec<usize>,
    paired_reps: Vec<Vec<usize>>,
    unpaired_reps: Vec<Vec<usize>>,
}

#[pyfunction]
fn compute_pairings_with_reps(
    py: Python<'_>,
    matrix: &PyAny,
    options: Option<LoPhatOptions>,
) -> PersistenceDiagramWithReps {
    // Overwrite maintain_v in options
    let options = Some(LoPhatOptions {
        maintain_v: true,
        ..options.unwrap_or_default()
    });
    // Run R=DV decomposition
    let decomposition = if let Ok(matrix_as_vec) = matrix.extract::<Vec<(usize, Vec<usize>)>>() {
        let matrix_as_rs_iter = matrix_as_vec.into_iter().map(VecColumn::from);
        LockFreeAlgorithm::init(options)
            .add_cols(matrix_as_rs_iter)
            .decompose()
    } else if let Ok(py_iter) = PyIterator::from_object(py, matrix) {
        let matrix_as_rs_iter = py_iter.map(|col| {
            col.and_then(PyAny::extract::<(usize, Vec<usize>)>)
                .map(VecColumn::from)
                .expect("Column is a list of unsigned integers")
        });
        LockFreeAlgorithm::init(options)
            .add_cols(matrix_as_rs_iter)
            .decompose()
    } else {
        panic!("Could not coerce input matrix into List[List[int]] | Iterator[List[int]]");
    };
    // Read off diagram and pull out representatives
    let mut diagram = decomposition.diagram();
    let (paired, paired_reps): (Vec<_>, Vec<Vec<_>>) = diagram
        .paired
        .drain()
        .map(|pairing| {
            (
                pairing,
                decomposition.get_r_col(pairing.1).entries().collect(),
            )
        })
        .unzip();
    let (unpaired, unpaired_reps): (Vec<_>, Vec<Vec<_>>) = diagram
        .unpaired
        .drain()
        .map(|birth| {
            (
                birth,
                decomposition.get_v_col(birth).unwrap().entries().collect(),
            )
        })
        .unzip();
    PersistenceDiagramWithReps {
        paired,
        unpaired,
        paired_reps,
        unpaired_reps,
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
    m.add_function(wrap_pyfunction!(compute_pairings_with_reps, m)?)?;
    m.add_class::<LoPhatOptions>()?;
    Ok(())
}
