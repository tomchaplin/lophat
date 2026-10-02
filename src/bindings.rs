use pyo3::prelude::*;

use crate::algorithms::{Decomposition, DecompositionAlgo, LockFreeAlgorithm};
use crate::columns::Column;
use crate::columns::VecColumn;
use crate::options::LoPhatOptions;
use crate::utils::PersistenceDiagram;

fn try_matrix_as_vec(matrix: &Bound<'_, PyAny>) -> PyResult<Vec<VecColumn>> {
    matrix
        .try_iter()?
        .map(|col| col?.extract::<(usize, Vec<usize>)>().map(VecColumn::from))
        .collect()
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
        ..options
            .map(|bound_opt| {
                bound_opt
                    .try_borrow() // get PyRef with runtime borrow-checking
                    .map(|val| *val)
            }) // Option<Result<LoPhatOptions, PyErr>>
            .transpose()?
            .unwrap_or(LoPhatOptions::default())
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
    options: Option<Bound<'_, LoPhatOptions>>,
) -> PyResult<PersistenceDiagram> {
    let options = options
        .map(|bound_opt| {
            bound_opt
                .try_borrow() // get PyRef with runtime borrow-checking
                .map(|val| *val)
        }) // Option<Result<LoPhatOptions, PyErr>>
        .transpose()?;
    let matrix_as_vec = try_matrix_as_vec(matrix)?;
    let algo = LockFreeAlgorithm::init(options);
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
