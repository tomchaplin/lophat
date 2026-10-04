use std::collections::HashSet;

#[cfg(feature = "python-module")]
use pyo3::prelude::*;

/// Persistence pairings and unpaired column indices read from a reduced matrix.
///
/// Each pair `(birth, death)` records a pivot row and the column that kills the
/// feature born at that row. Unpaired indices represent features that do not
/// die within the supplied filtration. For a valid filtered boundary matrix,
/// each input column index occurs exactly once across these sets.
///
/// The sets have no specified iteration order. Indices refer to input columns,
/// not filtration values. An empty matrix produces two empty sets.
#[doc = python_doc!(
r#"Persistence pairings and unpaired indices in the input boundary matrix.

Each ``(birth, death)`` pair contains column indices, not filtration values.
Unpaired indices identify features that persist beyond the supplied filtration.
For a valid filtered boundary matrix, each input index occurs exactly once
across :py:attr:`~lophat.PersistenceDiagram.paired` and
:py:attr:`~lophat.PersistenceDiagram.unpaired`. Both attributes are sets with
unspecified order.

Returned by :py:func:`~lophat.compute_pairings`; direct construction is not
supported. Both attributes are read-only. Changing a set obtained from an
attribute does not update the diagram.

Equality with ``==`` compares both sets. Other comparisons are unsupported.
An empty input produces empty sets.

:ivar paired: Set of (birth, death) index pairs.
:ivar unpaired: Set of indices of features that remain unpaired.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to a read-only attribute.
:py:exc:`TypeError`
    If constructing this class directly.
:py:exc:`NotImplementedError`
    If a comparison other than ``==`` is requested between diagrams."#
)]
#[cfg_attr(
	all(feature = "python-module", not(any(doc, rust_analyzer))),
	crate::utils::macro_rules_apply(crate::utils::strip_rust_docs!)
)]
#[cfg_attr(
	feature = "python-module",
	pyclass(module = "lophat", skip_from_py_object, get_all)
)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct PersistenceDiagram {
	/// Indices of columns that occur in no persistence pairing.
	#[doc = python_doc!(
r#"Set of indices of features that remain unpaired.

This attribute is read-only. Changing the returned set does not update the diagram.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to this attribute."#
	)]
	pub unpaired: HashSet<usize>,
	/// Persistence pairs `(birth, death)` in input column indices.
	#[doc = python_doc!(
r#"Set of ``(birth, death)`` pairs in input column indices.

This attribute is read-only. Changing the returned set does not update the diagram.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to this attribute."#
	)]
	pub paired: HashSet<(usize, usize)>,
}

impl PersistenceDiagram {
	/// Convert a diagram of an anti-transposed square matrix to original
	/// indices.
	///
	/// Consumes the diagram and maps each pair `(b, d)` to
	/// `(matrix_size - 1 - d, matrix_size - 1 - b)`, and each unpaired index
	/// `i` to `matrix_size - 1 - i`. `matrix_size` is the original matrix's
	/// column count. Empty diagrams remain empty, including when
	/// `matrix_size` is zero.
	///
	/// # Panics
	///
	/// May panic on subtraction overflow if any index is at least
	/// `matrix_size`.
	pub fn anti_transpose(mut self, matrix_size: usize) -> Self {
		let new_paired = self
			.paired
			.into_iter()
			.map(|(b, d)| (matrix_size - 1 - d, matrix_size - 1 - b))
			.collect();
		let new_unpaired = self
			.unpaired
			.into_iter()
			.map(|idx| matrix_size - 1 - idx)
			.collect();
		self.paired = new_paired;
		self.unpaired = new_unpaired;
		self
	}
}

impl std::fmt::Display for PersistenceDiagram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Paired: {:?}\nUnpaired: {:?}",
			self.paired, self.unpaired
		)
	}
}

#[cfg(feature = "python-module")]
#[cfg_attr(
	all(feature = "python-module", not(any(doc, rust_analyzer))),
	crate::utils::macro_rules_apply(crate::utils::strip_rust_docs!)
)]
#[pymethods]
impl PersistenceDiagram {
	#[doc = python_doc!(
r#"Compare both paired and unpaired sets with another :py:class:`~lophat.PersistenceDiagram`.

Only ``==`` is supported.

:returns: ``True`` if both diagrams have the same pairings and unpaired indices,
    otherwise ``False``.

.. rubric:: Raises

:py:exc:`NotImplementedError`
    If a comparison other than ``==`` is requested."#
	)]
	fn __richcmp__(
		&self,
		other: &PersistenceDiagram,
		cmp_op: pyo3::pyclass::CompareOp,
	) -> PyResult<bool> {
		match cmp_op {
			pyo3::pyclass::CompareOp::Eq => Ok(self == other),
			_ => {
				Err(pyo3::exceptions::PyNotImplementedError::new_err(
					"Only equality comparisons are supported for PersistenceDiagram",
				))
			},
		}
	}

	#[doc = python_doc!(
r#"Return a string displaying :py:attr:`~lophat.PersistenceDiagram.paired`
and :py:attr:`~lophat.PersistenceDiagram.unpaired`."#
	)]
	fn __repr__(&self) -> String { self.to_string() }
}

#[cfg(feature = "python-module")]
/// Persistence pairings together with representative cycles over the field with
/// two elements.
///
/// `paired_reps[i]` corresponds to `paired[i]`, and `unpaired_reps[i]`
/// corresponds to `unpaired[i]`. For a finite pair `(birth, death)`, the
/// representative is column `death` of R; for an unpaired birth, it is that
/// column of V. Each cycle is represented by the indices of its nonzero
/// coefficients in the original chain basis. List order is unspecified, but
/// corresponding lists are aligned.
#[doc = python_doc!(
r#"Persistence pairings with representative cycles over the field with two elements.

Returned by :py:func:`~lophat.compute_pairings_with_reps`; direct construction is
not supported. ``paired_reps[i]`` corresponds to ``paired[i]``, and
``unpaired_reps[i]`` corresponds to ``unpaired[i]``. The order of features is
unspecified, but these lists are aligned.

A representative lists the input column indices that form a cycle, each with
coefficient one. For a finite ``(birth, death)`` pair, that cycle becomes a
boundary when the death column enters the filtration. An unpaired feature's
cycle persists beyond the supplied filtration.

All attributes are read-only. Changing a returned list, including its nested
lists, does not update the diagram or the alignment of its representative lists.
An empty input produces four empty lists.

:ivar paired: List of (birth, death) index pairs.
:ivar unpaired: List of unpaired birth indices.
:ivar paired_reps: Representative cycles aligned with
    :py:attr:`~lophat.PersistenceDiagramWithReps.paired`.
:ivar unpaired_reps: Representative cycles aligned with
    :py:attr:`~lophat.PersistenceDiagramWithReps.unpaired`.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to a read-only attribute.
:py:exc:`TypeError`
    If constructing this class directly."#
)]
#[cfg_attr(
	all(feature = "python-module", not(any(doc, rust_analyzer))),
	crate::utils::macro_rules_apply(crate::utils::strip_rust_docs!)
)]
#[pyclass(skip_from_py_object, get_all, module = "lophat")]
pub struct PersistenceDiagramWithReps {
	/// Finite `(birth, death)` pairs, aligned with [`Self::paired_reps`].
	#[doc = python_doc!(
r#"List of ``(birth, death)`` pairs, aligned with
:py:attr:`~lophat.PersistenceDiagramWithReps.paired_reps`.

This attribute is read-only. Changing the returned list does not update the diagram.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to this attribute."#
	)]
	pub paired: Vec<(usize, usize)>,
	/// Unpaired birth indices, aligned with [`Self::unpaired_reps`].
	#[doc = python_doc!(
r#"List of unpaired birth indices, aligned with
:py:attr:`~lophat.PersistenceDiagramWithReps.unpaired_reps`.

This attribute is read-only. Changing the returned list does not update the diagram.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to this attribute."#
	)]
	pub unpaired: Vec<usize>,
	/// Nonzero basis indices of representative R columns, aligned with
	/// [`Self::paired`].
	#[doc = python_doc!(
r#"Representative cycles aligned with :py:attr:`~lophat.PersistenceDiagramWithReps.paired`.

This attribute is read-only. Changing the returned lists, including their nested
lists, does not update the diagram.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to this attribute."#
	)]
	pub paired_reps: Vec<Vec<usize>>,
	/// Nonzero basis indices of representative V columns, aligned with
	/// [`Self::unpaired`].
	#[doc = python_doc!(
r#"Representative cycles aligned with :py:attr:`~lophat.PersistenceDiagramWithReps.unpaired`.

This attribute is read-only. Changing the returned lists, including their nested
lists, does not update the diagram.

.. rubric:: Raises

:py:exc:`AttributeError`
    If assigning to this attribute."#
	)]
	pub unpaired_reps: Vec<Vec<usize>>,
}
