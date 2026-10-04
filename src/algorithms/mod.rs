//! Implementations of various algorithms for computing persistent homology.
//!
//! Use [`DecompositionAlgo`] to configure an algorithm, append input columns,
//! and consume it to produce an `R = D V` decomposition. [`Decomposition`]
//! provides access to R, optionally V, and the resulting persistence diagram.
//! All arithmetic is over the field with two elements.

use std::{collections::HashSet, ops::Deref};

use crate::{columns::Column, utils::PersistenceDiagram};

mod lock_free;
mod locking;
mod serial;

pub use lock_free::{LockFreeAlgorithm, LockFreeDecomposition};
pub use locking::{LockingAlgorithm, LockingDecomposition};
pub use serial::{SerialAlgorithm, SerialDecomposition};

/// Reason a decomposition cannot supply a V column.
///
/// Returned by [`Decomposition::get_v_col`] and [`Decomposition::has_v`]. An
/// empty decomposition cannot reveal its original `maintain_v` setting, even
/// if V was requested. A stored zero R column still makes a decomposition
/// nonempty.
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
	/// A nonzero R column at index `death` contributes `(pivot, death)`.
	/// All input indices that occur in no pair are returned as unpaired.
	/// Indices refer to the input basis, not filtration values. Requires no V
	/// columns; an empty decomposition produces an empty diagram. The intended
	/// persistence interpretation assumes a valid filtered boundary matrix.
	fn diagram(&self) -> PersistenceDiagram {
		let r_col_iter = (0..self.n_cols()).map(|idx| self.get_r_col(idx));
		let paired: HashSet<(usize, usize)> = r_col_iter
			.enumerate()
			.filter_map(|(idx, col)| {
				let lowest_idx = col.pivot()?;
				Some((lowest_idx, idx))
			})
			.collect();
		let mut unpaired: HashSet<usize> = (0..self.n_cols()).collect();
		for (birth, death) in paired.iter() {
			unpaired.remove(birth);
			unpaired.remove(death);
		}
		PersistenceDiagram { unpaired, paired }
	}

	/// Report whether a nonempty decomposition retained V.
	///
	/// Returns `Ok(())` when V is available. This is a presence query, not a
	/// Boolean method, and does not return the original options.
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
