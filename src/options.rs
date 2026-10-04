//! Configuration shared by the serial and parallel decomposition algorithms.

#[cfg(feature = "python-module")]
use pyo3::prelude::*;

use crate::utils::python_doc;
#[cfg(all(feature = "python-module", not(any(doc, rust_analyzer))))]
use crate::utils::{macro_rules_apply, strip_rust_docs};

/// Configure an `R = D V` matrix decomposition.
///
/// Construct this value with [`Default::default`] and override individual
/// fields. Defaults are `maintain_v = false`, `num_threads = 0`, `column_height
/// = None`, `min_chunk_len = 1`, and `clearing = true`.
///
/// [`SerialAlgorithm`](crate::algorithms::SerialAlgorithm) uses only
/// `maintain_v`, while
/// [`LockFreeAlgorithm`](crate::algorithms::LockFreeAlgorithm)
/// and [`LockingAlgorithm`](crate::algorithms::LockingAlgorithm) use the
/// remaining options. The clearing optimization requires a square boundary
/// matrix with `D * D = 0`, with each column labelled by its chain degree.
/// These requirements are not checked; violating them can produce incorrect
/// results or cause a panic. See [`Self::clearing`] for details.
///
/// # Example
///
/// ```
/// use lophat::options::LoPhatOptions;
///
/// let options = LoPhatOptions {
/// 	maintain_v: true,
/// 	num_threads: 2,
/// 	..Default::default()
/// };
/// assert!(options.maintain_v);
/// ```
#[doc = python_doc!(
r#"Configure the computation of persistence pairings over the field with two elements.

Use with :py:func:`~lophat.compute_pairings` or
:py:func:`~lophat.compute_pairings_with_reps`. Attributes can be updated for
subsequent calls; changing them does not affect a computation already in progress.

:param maintain_v: Whether to retain the change-of-basis matrix V. Default: ``False``.
    Ignored by :py:func:`~lophat.compute_pairings_with_reps`, which always retains V.
:param num_threads: Number of worker threads. Zero chooses the number of threads
    automatically. Default: ``0``.
:param column_height: Number of possible row indices. ``None`` uses the number of
    columns. For a rectangular matrix, supply its row count. Every boundary index
    must be strictly less than this height. See
    :py:attr:`~lophat.LoPhatOptions.column_height` for computation errors.
    Default: ``None``.
:param min_chunk_len: Minimum chunk size for parallel work. Default: ``1``.
:param clearing: Whether to use clearing. Requires a square boundary matrix
    with ``D * D = 0`` and correct column degrees. See
    :py:attr:`~lophat.LoPhatOptions.clearing` for requirements and computation
    errors. Default: ``True``.

.. rubric:: Raises

:py:exc:`TypeError`
    If a constructor argument or assigned attribute value has an incompatible type.
:py:exc:`OverflowError`
    If ``num_threads``, ``column_height``, or ``min_chunk_len`` is negative or too
    large for the supported integer range.
:py:exc:`RuntimeError`
    If reading or updating these options conflicts with a concurrent update.

Example::

    options = LoPhatOptions(num_threads=2, clearing=False)
    options.maintain_v = True"#
)]
#[cfg_attr(
	all(feature = "python-module", not(any(doc, rust_analyzer))),
	macro_rules_apply(strip_rust_docs!)
)]
#[cfg_attr(
	feature = "python-module",
	pyclass(module = "lophat", skip_from_py_object, get_all, set_all)
)]
#[derive(Copy, Clone)]
pub struct LoPhatOptions {
	/// Whether to retain V in the decomposition; defaults to `false`.
	///
	/// When disabled, querying a nonempty decomposition for a V column returns
	/// [`VMatrixDiscardedError`](crate::algorithms::NoVMatrixError::VMatrixDiscardedError).
	/// Querying an empty decomposition for a V column returns
	/// [`EmptyDecompositionError`](crate::algorithms::NoVMatrixError::EmptyDecompositionError)
	/// regardless of this setting.
	#[doc = python_doc!(
r#"Whether to retain the change-of-basis matrix V. Defaults to ``False``.

:py:func:`~lophat.compute_pairings_with_reps` always retains V regardless of
this setting."#
	)]
	pub maintain_v: bool,
	/// Number of threads in a parallel algorithm's local Rayon pool; defaults
	/// to zero.
	///
	/// Zero delegates the count to [`rayon::ThreadPoolBuilder::num_threads`].
	/// Ignored by the serial algorithm. Without the `local_thread_pool`
	/// feature, this must be zero or the parallel algorithm initialization
	/// panics.
	#[doc = python_doc!(
r#"Number of worker threads. Defaults to ``0``.

Zero chooses the number of threads automatically."#
	)]
	pub num_threads: usize,
	/// Size of the parallel algorithms' pivot table; defaults to `None`.
	///
	/// `None` uses the number of input columns. Every row index with a nonzero
	/// entry must be less than the resulting height. Supply the row count for
	/// rectangular matrices. Ignored by the serial algorithm.
	///
	/// The height is not validated against the input when setting this option.
	/// If reduction encounters a pivot row index greater than or equal to the
	/// height, it panics when accessing the pivot table.
	#[doc = python_doc!(
r#"Number of possible row indices, or ``None`` to use the number of columns.

Every boundary index must be strictly less than this height. For a rectangular
matrix, supply its row count. Defaults to ``None``.

Setting this option does not validate the matrix. The following error occurs
during computation.

.. rubric:: Raises

``pyo3_runtime.PanicException``
    If :py:func:`~lophat.compute_pairings` or
    :py:func:`~lophat.compute_pairings_with_reps` encounters a pivot row index
    greater than or equal to the height."#
	)]
	pub column_height: Option<usize>,
	/// Minimum chunk length used to split parallel reduction and clearing work.
	///
	/// Defaults to one. Ignored by the serial algorithm.
	#[doc = python_doc!(
r#"Minimum chunk size for parallel work. Defaults to ``1``."#
	)]
	pub min_chunk_len: usize,
	/// Whether parallel algorithms use clearing; defaults to `true`.
	///
	/// Requires a square boundary matrix satisfying `D * D = 0` and correct
	/// chain degrees. Disable for general or rectangular matrices. Ignored
	/// by the serial algorithm.
	///
	/// These requirements are not checked. Violating them can silently produce
	/// an incorrect decomposition or persistence pairings: clearing replaces a
	/// pivot's column with zero without reducing that column. Invalid input can
	/// also cause a panic, for example if a pivot row index does not identify
	/// an existing column. A successful return does not establish that the
	/// input satisfies the clearing requirements.
	#[doc = python_doc!(
r#"Whether to use the clearing optimization. Defaults to ``True``.

Enable only for a square boundary matrix with ``D * D = 0`` and correct chain
degrees. Set to ``False`` for general or rectangular matrices.

These requirements are not checked. Violating them can silently produce
incorrect persistence pairings or representatives, even when computation
completes successfully. The following error can also occur during computation.

.. rubric:: Raises

``pyo3_runtime.PanicException``
    If :py:func:`~lophat.compute_pairings` or
    :py:func:`~lophat.compute_pairings_with_reps` encounters a pivot row index
    that does not identify an existing column while clearing."#
	)]
	pub clearing: bool,
}

#[cfg(feature = "python-module")]
#[cfg_attr(
	all(feature = "python-module", not(any(doc, rust_analyzer))),
	macro_rules_apply(strip_rust_docs!)
)]
#[pymethods]
impl LoPhatOptions {
	#[doc = python_doc!(
r#"Create a :py:class:`~lophat.LoPhatOptions` with the supplied settings.

:param maintain_v: Whether to retain the change-of-basis matrix V. Default: ``False``.
    Ignored by :py:func:`~lophat.compute_pairings_with_reps`, which always retains V.
:param num_threads: Number of worker threads, or zero for automatic selection.
    Default: ``0``.
:param column_height: Row count, or ``None`` to use the column count. See
    :py:attr:`~lophat.LoPhatOptions.column_height` for matrix requirements.
    Default: ``None``.
:param min_chunk_len: Minimum chunk size for parallel work. Default: ``1``.
:param clearing: Whether to use clearing. See
    :py:attr:`~lophat.LoPhatOptions.clearing` for matrix requirements.
    Default: ``True``.

.. rubric:: Raises

:py:exc:`TypeError`
    If a supplied setting has an incompatible type.
:py:exc:`OverflowError`
    If ``num_threads``, ``column_height``, or ``min_chunk_len`` is negative or too
    large for the supported integer range."#
	)]
	#[new]
	#[pyo3(signature = (maintain_v=false, num_threads=0, column_height=None, min_chunk_len=1, clearing=true))]
	fn new(
		maintain_v: bool,
		num_threads: usize,
		column_height: Option<usize>,
		min_chunk_len: usize,
		clearing: bool,
	) -> Self {
		LoPhatOptions {
			maintain_v,
			num_threads,
			column_height,
			min_chunk_len,
			clearing,
		}
	}
}

impl Default for LoPhatOptions {
	fn default() -> Self {
		Self {
			maintain_v: false,
			num_threads: 0,
			column_height: None,
			min_chunk_len: 1,
			clearing: true,
		}
	}
}
