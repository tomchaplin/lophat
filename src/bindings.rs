use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::algorithms::{Decomposition, DecompositionAlgo, LockFreeAlgorithm};
use crate::columns::Column;
use crate::columns::VecColumn;
use crate::options::LoPhatOptions;
use crate::utils::PersistenceDiagram;

fn try_matrix_from_iter(matrix: &Bound<'_, PyAny>) -> PyResult<Vec<VecColumn>> {
    if let Ok(py_iter) = matrix.try_iter() {
        let mut matrix_as_vec = Vec::<VecColumn>::new();
        for col in py_iter {
            let veccol = col
                .and_then(|col_ok| col_ok.extract::<(usize, Vec<usize>)>())
                .map(VecColumn::from)?;
            matrix_as_vec.push(veccol);
        }
        Ok(matrix_as_vec)
    } else {
        Err(PyValueError::new_err(()))
    }
}

fn try_matrix_as_vec(matrix: &Bound<'_, PyAny>) -> PyResult<Vec<VecColumn>> {
    matrix
        .extract::<Vec<(usize, Vec<usize>)>>()
        .map(|extracted| extracted.into_iter().map(VecColumn::from).collect())
        .or_else(|_| try_matrix_from_iter(matrix))
        .map_err(|_| {
            let message = "Could not coerce input matrix into list[tuple[int, list[int]]] | Iterator[tuple[int, list[int]]]";
            PyValueError::new_err(message)
        })
}

#[pyclass(skip_from_py_object, get_all, set_all, module = "lophat")]
struct PersistenceDiagramWithReps {
    paired: Vec<(usize, usize)>,
    unpaired: Vec<usize>,
    paired_reps: Vec<Vec<usize>>,
    unpaired_reps: Vec<Vec<usize>>,
}

#[pyfunction]
#[pyo3(signature = (matrix, options=None))]
fn compute_pairings_with_reps(
    py: Python<'_>,
    matrix: &Bound<'_, PyAny>,
    options: Option<Bound<'_, LoPhatOptions>>,
) -> PyResult<PersistenceDiagramWithReps> {
    // Overwrite maintain_v in options
    let options = Some(LoPhatOptions {
        maintain_v: true,
        ..options.map_or(LoPhatOptions::default(), |bound_opt| *bound_opt.borrow())
    });
    // Get all the data from Python into Rust so we can detach
    let matrix_as_vec = try_matrix_as_vec(matrix)?;

    // Run R=DV decomposition
    py.detach(|| {
        let decomposition = LockFreeAlgorithm::init(options)
            .add_cols(matrix_as_vec.into_iter())
            .decompose();
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
        Ok(PersistenceDiagramWithReps {
            paired,
            unpaired,
            paired_reps,
            unpaired_reps,
        })
    })
}

#[pyfunction]
#[pyo3(signature = (matrix,anti_transpose= true, options=None))]
fn compute_pairings(
    py: Python<'_>,
    matrix: &Bound<'_, PyAny>,
    anti_transpose: bool,
    options: Option<&LoPhatOptions>,
) -> PyResult<PersistenceDiagram> {
    let matrix_as_vec = try_matrix_as_vec(matrix)?;
    let algo = LockFreeAlgorithm::init(options.copied());
    py.detach(|| {
        if anti_transpose {
            let width = matrix_as_vec.len();
            let at: Vec<_> = crate::utils::anti_transpose(&matrix_as_vec);
            let dgm = {
                let matrix = at.into_iter();
                algo.add_cols(matrix).decompose().diagram()
            };
            Ok(dgm.anti_transpose(width))
        } else {
            Ok(algo
                .add_cols(matrix_as_vec.into_iter())
                .decompose()
                .diagram())
        }
    })
}

// A Python module implemented in Rust.
#[pymodule(gil_used = true)]
fn lophat(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compute_pairings, m)?)?;
    m.add_function(wrap_pyfunction!(compute_pairings_with_reps, m)?)?;
    m.add_class::<LoPhatOptions>()?;
    m.add_class::<PersistenceDiagram>()?;
    Ok(())
}
