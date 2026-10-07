use pyo3::prelude::pymodule;

// Preserve reStructuredText indentation while formatting the surrounding code.
#[rustfmt::skip::attributes(doc)]
#[doc = r#"Compute persistence pairings and representative cycles over the field with two elements.

Matrices are iterables of ``(degree, boundary)`` tuples. A boundary lists the
strictly increasing, distinct row indices with coefficient one. Column indices
specify filtration order. Use :py:func:`~lophat.compute_pairings` for pairings
alone, or :py:func:`~lophat.compute_pairings_with_reps` for pairings and
representative cycles. Customize either computation with
:py:class:`~lophat.LoPhatOptions`.

Input iterators are consumed once. Keep the matrix and its columns unchanged
while a call reads them, and use a separate iterator for each concurrent call.
Computations can run concurrently, including on free-threaded Python."#]
#[pymodule(name = "lophat", gil_used = false)]
mod inner {
	use std::collections::HashMap;

	use pyo3::prelude::*;

	#[pymodule_export]
	pub use crate::options::LoPhatOptions;
	use crate::{
		algorithms::{Decomposition, DecompositionAlgo, LockFreeAlgorithm, NoVMatrixError},
		columns::VecColumn,
		utils::PersistenceDiagram,
	};

	// Carry the accepted input shape into PyO3's introspection metadata without
	// consuming the iterable or validating it before the options snapshot.
	struct MatrixInput<'py>(Bound<'py, PyAny>);

	impl<'a, 'py> FromPyObject<'a, 'py> for MatrixInput<'py> {
		type Error = PyErr;

		const INPUT_TYPE: pyo3::inspect::PyStaticExpr = pyo3::type_hint_subscript!(
			pyo3::type_hint_identifier!("collections.abc", "Iterable"),
			pyo3::type_hint_subscript!(
				pyo3::type_hint_identifier!("builtins", "tuple"),
				pyo3::type_hint_identifier!("builtins", "int"),
				pyo3::type_hint_subscript!(
					pyo3::type_hint_identifier!("collections.abc", "Sequence"),
					pyo3::type_hint_identifier!("builtins", "int")
				)
			)
		);

		fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
			Ok(Self(obj.to_owned()))
		}
	}

	fn try_matrix_as_vec(matrix: &Bound<'_, PyAny>) -> PyResult<Vec<VecColumn>> {
		matrix
			.try_iter()?
			.map(|col| col?.extract::<(usize, Vec<usize>)>().map(VecColumn::from))
			.collect()
	}

	#[doc = r#"Compute persistence pairings and representative cycles.

:param matrix: Iterable of ``(degree, boundary)`` tuples in filtration order.
    Each degree and row index must be a nonnegative integer. Each boundary must
    be a sequence of strictly increasing, distinct row indices with coefficient
    one. For a filtered boundary matrix, each row index precedes its column.
:param options: A :py:class:`~lophat.LoPhatOptions`, or ``None`` to use defaults.
    Settings are selected before consuming the matrix. The supplied options are
    not modified, and later changes to them do not affect this computation.
    :py:attr:`~lophat.LoPhatOptions.maintain_v` is ignored; representatives are
    always computed.
:returns: A :py:class:`tuple` of two dictionaries ``(diagram, representatives)``.
    The diagram maps each birth column index to its death column index, or
    ``None`` for a feature that persists beyond the supplied filtration.
    The representatives map has exactly the same birth keys. Each value lists
    the nonzero input basis indices of a representative cycle, with coefficient
    one over the field with two elements. For a finite interval, the cycle becomes
    a boundary at its death; an essential feature's cycle persists beyond the
    supplied filtration. Indices refer to columns, not filtration values.

The input is reduced directly, without anti-transposition.
:py:attr:`~lophat.LoPhatOptions.clearing` defaults to ``True`` and requires a
square boundary matrix with ``D * D = 0`` and correct chain degrees. For
rectangular input, disable clearing and set
:py:attr:`~lophat.LoPhatOptions.column_height` to the row count. These mathematical
requirements are not validated; violating them can silently produce incorrect
results even when computation completes successfully.

The iterable is consumed once. If a call fails while reading the input, the
iterator may already be partly consumed. Retry with a fresh iterator, and do not
consume the same iterator from another thread. Keep its columns unchanged while
they are being read. Exceptions raised by the iterable propagate to the caller.
An empty input returns ``({}, {})``. Use ``diagram[birth]`` to look up a death
and ``representatives[birth]`` to look up its cycle. Both dictionaries and their
returned lists can be edited independently of other results. Dictionary order
is unspecified; match each diagram entry to its representative by birth key.

.. rubric:: Raises

:py:exc:`TypeError`
    If the matrix, a column, or an option has an incompatible type.
:py:exc:`ValueError`
    If a column tuple does not contain exactly two elements.
:py:exc:`OverflowError`
    If a degree or row index is negative or too large for the supported index range.
:py:exc:`RuntimeError`
    If accessing the supplied options conflicts with a concurrent update.
``pyo3_runtime.PanicException``
    If a pivot row index is greater than or equal to the configured column height,
    or clearing encounters a pivot row index that does not identify an existing column.

Example::

    matrix = [(0, []), (0, []), (1, [0, 1])]
    diagram, representatives = compute_pairings_with_reps(matrix)
    assert diagram == {0: None, 1: 2}
    assert representatives == {0: [0], 1: [0, 1]}"#]
	#[pyfunction]
	#[pyo3(signature = (matrix, options=None))]
	fn compute_pairings_with_reps(
		py: Python<'_>,
		matrix: MatrixInput<'_>,
		options: Option<Bound<'_, LoPhatOptions>>,
	) -> PyResult<(PersistenceDiagram, HashMap<usize, Vec<usize>>)> {
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
		let matrix_as_vec = try_matrix_as_vec(&matrix.0)?;

		// Run R=DV decomposition
		py.detach(|| {
			let decomposition = LockFreeAlgorithm::init(options)
				.add_cols(matrix_as_vec.into_iter())
				.decompose();
			let (diagram, representatives) = decomposition.diagram_with_reps();
			let representatives = match representatives {
				Ok(representatives) => representatives,
				// Preserve the Python API's empty-input result.
				Err(NoVMatrixError::EmptyDecompositionError) => HashMap::new(),
				Err(NoVMatrixError::VMatrixDiscardedError) => {
					unreachable!("maintain_v is always enabled for representative computation")
				},
			};
			Ok((diagram, representatives))
		})
	}

	#[doc = r#"Compute persistence pairings.

:param matrix: Iterable of ``(degree, boundary)`` tuples in filtration order.
    Each degree and row index must be a nonnegative integer. Each boundary must
    be a sequence of strictly increasing, distinct row indices with coefficient
    one. For a filtered boundary matrix, each row index precedes its column.
:param anti_transpose: Whether to anti-transpose the square input matrix before
    reduction. Defaults to ``True``. Results refer to the original column
    indices. Set to ``False`` for rectangular input.
:param options: A :py:class:`~lophat.LoPhatOptions`, or ``None`` to use defaults.
    Settings are selected before consuming the matrix. The supplied options are
    not modified, and later changes to them do not affect this computation.
:returns: A :py:class:`dict` mapping each birth column index to its death
    column index, or ``None`` for a feature that persists beyond the supplied
    filtration. These are indices, not filtration values. Use
    :py:func:`~lophat.compute_pairings_with_reps` to also obtain representative cycles.

:py:attr:`~lophat.LoPhatOptions.clearing` defaults to ``True`` and requires a
square boundary matrix with ``D * D = 0`` and correct chain degrees. For
rectangular input, set ``anti_transpose=False``, disable clearing, and set
:py:attr:`~lophat.LoPhatOptions.column_height` to the row count. These mathematical
requirements are not validated; violating them can silently produce incorrect
results even when computation completes successfully.

The iterable is consumed once. If a call fails while reading the input, the
iterator may already be partly consumed. Retry with a fresh iterator, and do not
consume the same iterator from another thread. Keep its columns unchanged while
they are being read. Exceptions raised by the iterable propagate to the caller.
An empty input returns an empty dictionary. Use ``diagram[birth]`` for lookup
and ``diagram.items()`` to iterate over intervals. The returned dictionary can
be edited independently of other results.

.. rubric:: Raises

:py:exc:`TypeError`
    If the matrix, a column, or an option has an incompatible type.
:py:exc:`ValueError`
    If a column tuple does not contain exactly two elements.
:py:exc:`OverflowError`
    If a degree or row index is negative or too large for the supported index range.
:py:exc:`RuntimeError`
    If accessing the supplied options conflicts with a concurrent update.
``pyo3_runtime.PanicException``
    If anti-transposition encounters a row index outside the column range, a pivot
    row index is greater than or equal to the configured column height, or clearing
    encounters a pivot row index that does not identify an existing column.

Example::

    matrix = [(0, []), (0, []), (1, [0, 1])]
    diagram = compute_pairings(matrix)
    assert diagram == {0: None, 1: 2}"#]
	#[pyfunction]
	#[pyo3(signature = (matrix,anti_transpose= true, options=None))]
	fn compute_pairings(
		py: Python<'_>,
		matrix: MatrixInput<'_>,
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
		let matrix_as_vec = try_matrix_as_vec(&matrix.0)?;
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
}
