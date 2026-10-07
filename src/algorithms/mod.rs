//! Implementations of various algorithms for computing persistent homology.
//!
//! Use [`DecompositionAlgo`] to configure an algorithm, append input columns,
//! and consume it to produce an `R = D V` decomposition. [`Decomposition`]
//! provides access to R, optionally V, the persistence diagram, and
//! representative cycles.
//! All arithmetic is over the field with two elements.

use std::{collections::HashMap, ops::Deref};

use crate::{
	columns::Column,
	utils::{ExtendedUsize, PersistenceDiagram},
};

mod lock_free;
mod locking;
mod serial;

pub use lock_free::{LockFreeAlgorithm, LockFreeDecomposition};
pub use locking::{LockingAlgorithm, LockingDecomposition};
pub use serial::{SerialAlgorithm, SerialDecomposition};

/// Reason a decomposition cannot supply a V column.
///
/// Returned by [`Decomposition::get_v_col`], [`Decomposition::has_v`], and
/// the representative result of [`Decomposition::diagram_with_reps`]. An empty
/// decomposition cannot reveal its original `maintain_v` setting, even if V
/// was requested. A stored zero R column still makes a decomposition nonempty.
#[derive(Debug, PartialEq, Eq)]
pub enum NoVMatrixError {
	/// The decomposition has no columns, so V cannot be queried.
	EmptyDecompositionError,
	/// The decomposition is nonempty, but V was not maintained.
	VMatrixDiscardedError,
}

/// Query a reduced matrix R and the optional change-of-basis matrix V in `R = D
/// V`.
///
/// Usually constructed by [`DecompositionAlgo::decompose`]. Column access is
/// immutable; the associated reference types may be ordinary references or
/// guards owning access to parallel storage. [`Self::diagram`] uses R alone,
/// so it works when V was discarded and when the decomposition is empty.
/// [`Self::diagram_with_reps`] additionally extracts representative cycles
/// when V is available.
///
/// V availability must be uniform across a nonempty decomposition: either
/// every input column has a corresponding V column, or V is absent entirely.
/// Once [`Self::has_v`] succeeds, [`Self::get_v_col`] must succeed for every
/// valid column index.
pub trait Decomposition<C>
where
	C: Column,
{
	/// Reference or guard dereferencing to a column of R.
	type RColRef<'a>: Deref<Target = C> + 'a
	where
		Self: 'a;
	/// Borrow column `index` of the reduced matrix R.
	///
	/// A stored zero column is a valid column and is returned normally.
	///
	/// # Panics
	///
	/// Panics if `index >= self.n_cols()`, including every index of an empty
	/// decomposition.
	fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a>;

	/// Reference or guard dereferencing to a retained column of V.
	type VColRef<'a>: Deref<Target = C> + 'a
	where
		Self: 'a;
	/// Borrow column `index` of the change-of-basis matrix V.
	///
	/// # Errors
	///
	/// Returns [`NoVMatrixError::EmptyDecompositionError`] for an empty
	/// decomposition, regardless of `index` and the original `maintain_v`
	/// setting. For a valid index of a nonempty decomposition, returns
	/// [`NoVMatrixError::VMatrixDiscardedError`] if V was not retained.
	///
	/// # Panics
	///
	/// For nonempty decompositions, callers must supply `index <
	/// self.n_cols()`. An out-of-range index may panic even if V was
	/// discarded.
	fn get_v_col<'a>(&'a self, index: usize) -> Result<Self::VColRef<'a>, NoVMatrixError>;

	/// Return the number of input columns, including stored zero columns.
	fn n_cols(&self) -> usize;

	/// Read persistence pairings and unpaired indices from the pivots of R.
	///
	/// A nonzero R column at index `death` contributes `pivot ->
	/// Finite(death)`. All input indices that occur in no pair are mapped
	/// to `ExtendedUsize::Infinity`. Indices refer to the input basis, not
	/// filtration values. Requires no V columns; an empty decomposition
	/// produces an empty diagram. The intended persistence interpretation
	/// assumes a valid filtered boundary matrix.
	fn diagram(&self) -> PersistenceDiagram { PersistenceDiagram::from_decomposition(self) }

	/// Return the persistence diagram and a result containing representative
	/// cycles.
	///
	/// The diagram is returned even when representatives are unavailable. On
	/// success, the representative map has exactly the diagram's birth keys.
	/// Each vector lists the nonzero input basis indices of a cycle, with
	/// coefficient one over the field with two elements. Entry order follows
	/// [`Column::entries`] and is not guaranteed by this method.
	///
	/// For a finite interval `(birth, death)`, the representative is column
	/// `death` of R. For an essential interval `(birth, Infinity)`, it is
	/// column `birth` of V. With a valid filtered boundary matrix D, reduced
	/// R, and `R = D V` for an invertible, upper-triangular V that preserves
	/// chain degrees, these cycles are supported at or before their births.
	/// A finite cycle represents a nonzero homology class until `death`,
	/// when it becomes a boundary; an essential cycle remains nonzero
	/// through the supplied filtration. Representatives need not be minimal
	/// or unique; reduction order can select different valid cycles.
	///
	/// This interpretation requires a square D with `D * D = 0`, boundary rows
	/// preceding their columns, and correct chain degrees. These conditions
	/// and the decomposition invariants are not validated. Results use the
	/// decomposition's input basis; reindexing the returned diagram alone with
	/// [`PersistenceDiagram::map_idxs`] does not reindex the representative
	/// keys or their basis indices.
	///
	/// # Errors
	///
	/// The representative result is [`NoVMatrixError::EmptyDecompositionError`]
	/// for an empty decomposition, alongside an empty diagram, regardless of
	/// the original `maintain_v` setting. For a nonempty decomposition without
	/// V, it is [`NoVMatrixError::VMatrixDiscardedError`]. V must be retained
	/// even if every interval is finite and its representative uses R alone.
	///
	/// # Panics
	///
	/// Invalid decompositions may panic through column access, including
	/// implementations that violate the trait's uniform V availability
	/// invariant.
	///
	/// # Example
	///
	/// ```
	/// use std::collections::HashMap;
	///
	/// use lophat::{
	/// 	algorithms::{Decomposition, DecompositionAlgo, SerialAlgorithm},
	/// 	columns::VecColumn,
	/// 	options::LoPhatOptions,
	/// 	utils::ExtendedUsize::{Finite, Infinity},
	/// };
	///
	/// let options = LoPhatOptions {
	/// 	maintain_v: true,
	/// 	..Default::default()
	/// };
	/// let matrix = [(0, vec![]), (0, vec![]), (1, vec![0, 1])];
	/// let decomposition = SerialAlgorithm::init(Some(options))
	/// 	.add_cols(matrix.into_iter().map(VecColumn::from))
	/// 	.decompose();
	/// let (diagram, representatives) = decomposition.diagram_with_reps();
	/// assert_eq!(diagram[&0], Infinity);
	/// assert_eq!(diagram[&1], Finite(2));
	/// assert_eq!(
	/// 	representatives.unwrap(),
	/// 	HashMap::from([(0, vec![0]), (1, vec![0, 1])])
	/// );
	/// ```
	fn diagram_with_reps(
		&self,
	) -> (
		PersistenceDiagram,
		Result<HashMap<usize, Vec<usize>>, NoVMatrixError>,
	) {
		let dgm = PersistenceDiagram::from_decomposition(self);

		let reps = self.has_v().map(|_| {
			dgm.iter()
				.map(|(&birth, &death)| {
					(
						birth,
						match death {
							ExtendedUsize::Finite(death) => {
								self.get_r_col(death).entries().collect()
							},
							ExtendedUsize::Infinity => {
								self.get_v_col(birth).unwrap().entries().collect()
							},
						},
					)
				})
				.collect()
		});
		(dgm, reps)
	}

	/// Report whether a nonempty decomposition retained V.
	///
	/// Returns `Ok(())` when V is available. This is a presence query, not a
	/// Boolean method, and does not return the original options. Success
	/// guarantees V access for every valid column index by the trait's
	/// uniform V availability invariant.
	///
	/// # Errors
	///
	/// Returns [`NoVMatrixError::EmptyDecompositionError`] for an empty
	/// decomposition, whose original `maintain_v` setting is not recoverable
	/// from column presence. Returns [`NoVMatrixError::VMatrixDiscardedError`]
	/// for a nonempty decomposition that discarded V.
	fn has_v(&self) -> Result<(), NoVMatrixError> {
		if self.n_cols() == 0 {
			return Err(NoVMatrixError::EmptyDecompositionError);
		}
		self.get_v_col(0).map(|_| ())
	}
}

/// Build a matrix D and compute its `R = D V` decomposition over the field with
/// two elements.
///
/// Start with [`Self::init`], append columns using [`Self::add_cols`], and
/// optionally toggle additional coefficients using [`Self::add_entries`].
/// [`Self::decompose`] consumes the builder and returns the reduced matrix,
/// together with V when requested by the algorithm's options.
///
/// For persistence, columns must be in filtration order and boundary rows must
/// precede their columns. Clearing additionally requires a square boundary
/// matrix with `D * D = 0` and correct chain degrees. Algorithms do not
/// validate these mathematical requirements.
pub trait DecompositionAlgo<C>
where
	C: Column,
{
	/// Copyable configuration accepted during initialization.
	type Options: Default + Copy;
	/// Create an empty builder with the supplied options, or defaults for
	/// `None`.
	///
	/// # Panics
	///
	/// Parallel implementations may panic if their thread pool cannot be built.
	/// Without `local_thread_pool`, they require `num_threads = 0`.
	fn init(options: Option<Self::Options>) -> Self;

	/// Consume and append columns in their existing order, returning the
	/// builder.
	///
	/// Ownership of each column is transferred into the builder. Column degrees
	/// and native entry invariants must already be valid. Calling this method
	/// again appends after previously supplied columns.
	fn add_cols(self, cols: impl Iterator<Item = C>) -> Self;

	/// Toggle the supplied `(row, column)` coefficients modulo two.
	///
	/// Columns must already exist, and their degrees are unchanged. Repeated
	/// entries cancel. Row bounds are the caller's responsibility.
	///
	/// # Panics
	///
	/// Panics if a column index does not refer to a previously appended column.
	fn add_entries(self, entries: impl Iterator<Item = (usize, usize)>) -> Self;

	/// Output type providing immutable access to R and optional V columns.
	type Decomposition: Decomposition<C>;
	/// Consume the input builder and reduce D using the configured algorithm.
	///
	/// An empty input produces a decomposition with no R columns; V queries
	/// then report [`NoVMatrixError::EmptyDecompositionError`].
	///
	/// # Panics
	///
	/// Parallel implementations panic if a row index exceeds the configured
	/// pivot-table height. Invalid matrix or column invariants can cause panics
	/// or incorrect results; they are not checked before reduction.
	fn decompose(self) -> Self::Decomposition;
}

#[cfg(test)]
mod tests {
	use super::{
		Decomposition,
		DecompositionAlgo,
		LockFreeAlgorithm,
		LockingAlgorithm,
		NoVMatrixError,
		SerialAlgorithm,
	};
	use crate::{columns::VecColumn, options::LoPhatOptions};

	fn check_v_presence<A: DecompositionAlgo<VecColumn, Options = LoPhatOptions>>() {
		for maintain_v in [false, true] {
			let options = LoPhatOptions {
				maintain_v,
				num_threads: 1,
				..Default::default()
			};
			let empty = A::init(Some(options)).decompose();
			assert_eq!(empty.n_cols(), 0);
			assert_eq!(empty.has_v(), Err(NoVMatrixError::EmptyDecompositionError));
			assert_eq!(
				empty.get_v_col(0).err(),
				Some(NoVMatrixError::EmptyDecompositionError)
			);

			// A zero column still occupies a slot, so this decomposition is
			// nonempty.
			let nonempty = A::init(Some(options))
				.add_cols(std::iter::once(VecColumn::from((0, vec![]))))
				.decompose();
			assert_eq!(nonempty.n_cols(), 1);
			assert_eq!(*nonempty.get_r_col(0), VecColumn::from((0, vec![])));
			if maintain_v {
				assert_eq!(nonempty.has_v(), Ok(()));
				assert_eq!(
					*nonempty.get_v_col(0).unwrap(),
					VecColumn::from((0, vec![0]))
				);
			} else {
				assert_eq!(nonempty.has_v(), Err(NoVMatrixError::VMatrixDiscardedError));
				assert_eq!(
					nonempty.get_v_col(0).err(),
					Some(NoVMatrixError::VMatrixDiscardedError)
				);
			}
		}
	}

	#[test]
	fn serial_reports_v_presence() { check_v_presence::<SerialAlgorithm<VecColumn>>(); }

	#[test]
	fn lockfree_reports_v_presence() { check_v_presence::<LockFreeAlgorithm<VecColumn>>(); }

	#[test]
	fn locking_reports_v_presence() { check_v_presence::<LockingAlgorithm<VecColumn>>(); }
}
