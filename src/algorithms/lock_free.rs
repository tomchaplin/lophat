use std::{
	ops::Deref,
	sync::atomic::{
		AtomicUsize,
		Ordering::{Relaxed, Release},
	},
};

use pinboard::{GuardedRef, NonEmptyPinboard};
#[cfg(feature = "local_thread_pool")]
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;

use super::{Decomposition, DecompositionAlgo, NoVMatrixError};
#[cfg(feature = "serde")]
use crate::impl_rvd_serialize;
use crate::{
	columns::{
		Column,
		ColumnMode::{Storage, Working},
	},
	options::LoPhatOptions,
	utils::set_mode_of_pair,
};

enum LoPhatThreadPool {
	#[cfg(not(feature = "local_thread_pool"))]
	Global(),
	#[cfg(feature = "local_thread_pool")]
	Local(rayon::ThreadPool),
}

impl LoPhatThreadPool {
	fn install<OP, R>(&self, op: OP) -> R
	where
		OP: FnOnce() -> R + Send,
		R: Send,
	{
		match self {
			#[cfg(not(feature = "local_thread_pool"))]
			LoPhatThreadPool::Global() => op(),
			#[cfg(feature = "local_thread_pool")]
			LoPhatThreadPool::Local(pool) => pool.install(op),
		}
	}
}

/// Parallel matrix reduction using atomic pointers and immutable guarded
/// snapshots.
///
/// Implements the algorithm of [Morozov and Nigmetov](https://doi.org/10.1145/3350755.3400244)
/// with atomic column publication and the clearing optimization of
/// [Bauer et al.](https://doi.org/10.1007/978-3-319-04099-8_7).
///
/// Use [`DecompositionAlgo::init`], [`DecompositionAlgo::add_cols`], and
/// [`DecompositionAlgo::decompose`] as the public construction workflow. The
/// algorithm uses [`LoPhatOptions`] and is generic over the [`Column`] storage.
/// Clearing requires a square boundary matrix with `D * D = 0` and correct
/// chain degrees. For general or rectangular matrices, disable clearing and
/// supply an appropriate `column_height`. Mathematical requirements are not
/// validated.
///
/// The inherent reduction methods are low-level operations used during the
/// consuming decomposition workflow. Their pivot table is initialized by
/// `decompose`, not by `init` or `add_cols`.
pub struct LockFreeAlgorithm<C: Column + 'static> {
	matrix: Vec<NonEmptyPinboard<(C, Option<C>)>>,
	// NOTE: We use `usize::MAX` as a sentinel value, meaning no pivot.
	pivots: Vec<AtomicUsize>,
	options: LoPhatOptions,
	thread_pool: LoPhatThreadPool,
	max_dim: usize,
}
type RVColumnPair<C> = (C, Option<C>);

impl<C: Column + 'static> LockFreeAlgorithm<C> {
	// Returns the value in position [idx] of the pivots array
	// Maps to Option<usize> to cover the case that no column yet has that pivot
	fn get_pivot(&self, idx: usize) -> Option<usize> {
		let piv = self
			.pivots
			.get(idx)
			.expect("Should ask for column index within range")
			.load(Relaxed);
		usize_to_option_usize(piv)
	}

	// Attempts to compare_exchange_week position [idx] of the pivots array
	// Returns whether or not the operation succeeded
	fn cew_pivot_succeeds(&self, idx: usize, current: Option<usize>, new: Option<usize>) -> bool {
		let current = option_usize_to_usize(current);
		let new = option_usize_to_usize(new);
		self.pivots[idx]
			.compare_exchange_weak(current, new, Release, Relaxed)
			.is_ok()
	}

	/// Borrow a published column whose greatest nonzero row is `l`.
	///
	/// Returns the column index and a guard over `(r_column,
	/// optional_v_column)`. The second column is absent when `maintain_v`
	/// is disabled. Returns `None` if no pivot has been published for row
	/// `l`; publication may change while other threads reduce the matrix.
	/// The guard keeps its column data accessible.
	///
	/// # Panics
	///
	/// Panics if `l` is outside the initialized pivot table. This table is set
	/// up inside [`DecompositionAlgo::decompose`], so this is a low-level
	/// operation rather than a query on a newly created builder.
	pub fn get_col_with_pivot(&self, l: usize) -> Option<(usize, GuardedRef<RVColumnPair<C>>)> {
		loop {
			// If there is not yet a column with pivot l, inform caller.
			let piv = self.get_pivot(l)?;
			let cols = self.matrix[piv].get_ref();
			if cols.0.pivot() != Some(l) {
				// Got a column but it now has the wrong pivot; loop again.
				continue;
			};
			// Get column with correct pivot, return to caller.
			return Some((piv, cols));
		}
	}

	/// Reduce column `j`, publishing changes to the matrix and pivot table.
	///
	/// If a competing pivot owner lies to the right of the working column, the
	/// operation can continue by reducing that displaced column. The algorithm
	/// calls this operation concurrently for distinct columns during reduction.
	///
	/// # Panics
	///
	/// Panics for an invalid column index or a pivot outside the initialized
	/// table. Calling it with an already published pivot owned by the same
	/// column can also panic. The pivot table must have been initialized by
	/// the decomposition workflow; this is not a standalone reduction entry
	/// point.
	pub fn reduce_column(&self, j: usize) {
		let mut working_j = j;
		'outer: loop {
			// We make a copy of the column because we want to mutate our local
			// copy
			let mut curr_column = self.matrix[working_j].read();
			set_mode_of_pair(&mut curr_column, Working);
			while let Some(l) = curr_column.0.pivot() {
				let piv_with_column_opt = self.get_col_with_pivot(l);
				if let Some((piv, piv_column)) = piv_with_column_opt {
					// Lines 17-24
					if piv < working_j {
						curr_column.0.add_col(&piv_column.0);
						// Only add V columns if we need to
						if self.options.maintain_v {
							let curr_v_col = curr_column.1.as_mut().unwrap();
							curr_v_col.add_col(piv_column.1.as_ref().unwrap());
						}
					} else if piv > working_j {
						self.write_to_matrix(working_j, curr_column);
						if self.cew_pivot_succeeds(l, Some(piv), Some(working_j)) {
							working_j = piv;
						}
						continue 'outer;
					} else {
						panic!()
					}
				} else {
					// piv = -1 case
					self.write_to_matrix(working_j, curr_column);
					if self.cew_pivot_succeeds(l, None, Some(working_j)) {
						return;
					} else {
						continue 'outer;
					}
				}
			}
			// Lines 25-27 (curr_column = 0 clause)
			if curr_column.0.is_cycle() {
				self.write_to_matrix(working_j, curr_column);
				return;
			}
		}
	}

	fn write_to_matrix(&self, index: usize, mut to_write: (C, Option<C>)) {
		set_mode_of_pair(&mut to_write, Storage);
		self.matrix[index].set(to_write);
	}

	/// Clear the pivot column of a nonzero reduced boundary column.
	///
	/// `boudary_idx` identifies the boundary column in R. Its pivot determines
	/// the column to clear. If V is retained, that V column is replaced
	/// with the boundary's R column, relabelled with the cleared column's
	/// degree. Requires the clearing invariants of a square chain-complex
	/// boundary matrix.
	///
	/// # Panics
	///
	/// Panics if the boundary index or pivot column index is out of range, or
	/// if the boundary column is zero. Use only after reducing the boundary
	/// column.
	pub fn clear_with_column(&self, boudary_idx: usize) {
		let boundary = self.matrix[boudary_idx].get_ref();
		let boundary_r = &boundary.0;
		let clearing_idx = boundary_r
			.pivot()
			.expect("Attempted to clear using cycle column");
		let clearing_degree = self.matrix[clearing_idx].get_ref().0.degree();
		// The cleared R column is empty
		let r_col = C::new_with_degree(clearing_degree);
		// The corresponding V column should be the R column of the boundary
		let v_col = self.options.maintain_v.then(|| {
			let mut br = boundary_r.clone();
			br.set_degree(clearing_degree);
			br
		});
		self.write_to_matrix(clearing_idx, (r_col, v_col));
	}

	/// Reduce all columns carrying the given chain degree in parallel.
	///
	/// Uses the configured Rayon pool and `min_chunk_len`. Called by the
	/// consuming decomposition workflow after initializing its pivot table.
	/// Inherits the preconditions and panic behavior of
	/// [`Self::reduce_column`].
	pub fn reduce_in_degree(&self, degree: usize) {
		// Reduce matrix for columns of that degree
		self.thread_pool.install(|| {
			(0..self.matrix.len())
				.into_par_iter()
				.with_min_len(self.options.min_chunk_len)
				.filter(|&j| self.matrix[j].get_ref().0.degree() == degree)
				.for_each(|j| self.reduce_column(j));
		});
	}

	/// Use nonzero R columns of the given degree to clear their pivot columns.
	///
	/// Runs in parallel with the configured pool and `min_chunk_len`. Those
	/// boundary columns must already have been reduced, and clearing requires a
	/// valid square chain-complex boundary matrix. Inherits the panic behavior
	/// of [`Self::clear_with_column`].
	pub fn clear_in_degree(&self, degree: usize) {
		// Reduce matrix for columns of that degree
		self.thread_pool.install(|| {
			(0..self.matrix.len())
				.into_par_iter()
				.with_min_len(self.options.min_chunk_len)
				.filter(|&j| self.matrix[j].get_ref().0.degree() == degree)
				.filter(|&j| self.matrix[j].get_ref().0.is_boundary())
				.for_each(|j| self.clear_with_column(j));
		});
	}
}

impl<C: Column> DecompositionAlgo<C> for LockFreeAlgorithm<C> {
	type Decomposition = LockFreeDecomposition<C>;
	type Options = LoPhatOptions;

	fn init(options: Option<Self::Options>) -> Self {
		let options = options.unwrap_or_default();
		// Setup thread pool
		#[cfg(feature = "local_thread_pool")]
		let thread_pool = LoPhatThreadPool::Local(
			ThreadPoolBuilder::new()
				.num_threads(options.num_threads)
				.build()
				.expect("Failed to build thread pool"),
		);
		#[cfg(not(feature = "local_thread_pool"))]
		let thread_pool = {
			if options.num_threads != 0 {
				panic!(
					"To specify a number of threads, please enable the local_thread_pool feature"
				);
			}
			LoPhatThreadPool::Global()
		};
		Self {
			matrix: vec![],
			pivots: vec![],
			options,
			thread_pool,
			max_dim: 0,
		}
	}

	fn add_cols(mut self, cols: impl Iterator<Item = C>) -> Self {
		let first_idx = self.matrix.len();
		let new_cols = cols.enumerate().map(|(idx, r_col)| {
			self.max_dim = self.max_dim.max(r_col.degree());
			if self.options.maintain_v {
				let mut v_col = C::new_with_degree(r_col.degree());
				v_col.add_entry(first_idx + idx);
				NonEmptyPinboard::new((r_col, Some(v_col)))
			} else {
				NonEmptyPinboard::new((r_col, None))
			}
		});
		self.matrix.extend(new_cols);
		self
	}

	fn add_entries(self, entries: impl Iterator<Item = (usize, usize)>) -> Self {
		for (row, col) in entries {
			let col = self
				.matrix
				.get(col)
				.expect("Column index should correspond to a pre-existing column");
			let mut col_clone = col.get_ref().clone();
			col_clone.0.add_entry(row);
			col.set(col_clone);
		}
		self
	}

	fn decompose(mut self) -> Self::Decomposition {
		// Setup pivots vector
		let column_height = self.options.column_height.unwrap_or(self.matrix.len());
		self.pivots = (0..column_height)
			.map(|_| AtomicUsize::new(usize::MAX))
			.collect();
		// Decompose
		for degree in (0..=self.max_dim).rev() {
			self.reduce_in_degree(degree);
			if self.options.clearing && degree > 0 {
				self.clear_in_degree(degree)
			}
		}
		LockFreeDecomposition(self.matrix)
	}
}

/// Reduced R and optional V columns produced by [`LockFreeAlgorithm`].
///
/// Use the [`Decomposition`] methods to borrow columns or compute a persistence
/// diagram. The decomposition includes zero columns in its column count. Empty
/// decompositions report [`NoVMatrixError::EmptyDecompositionError`] when V is
/// queried, regardless of the original options.
///
/// With the `serde` feature, serialization uses the common
/// `DecompositionFileFormat` representation in [`crate::utils`].
pub struct LockFreeDecomposition<C: Column + 'static>(Vec<NonEmptyPinboard<(C, Option<C>)>>);

/// Immutable guard dereferencing to a R column in a [`LockFreeDecomposition`].
///
/// Returned by [`Decomposition::get_r_col`]. Keeps the underlying
/// published snapshot alive for the duration of the borrow.
pub struct LockFreeRRef<C>(GuardedRef<(C, Option<C>)>);

impl<C> Deref for LockFreeRRef<C> {
	type Target = C;

	fn deref(&self) -> &Self::Target { &self.0.deref().0 }
}

/// Immutable guard dereferencing to a V column in a [`LockFreeDecomposition`].
///
/// Returned by [`Decomposition::get_v_col`]. Keeps the underlying
/// published snapshot alive for the duration of the borrow.
pub struct LockFreeVRef<C>(GuardedRef<(C, Option<C>)>);

impl<C> Deref for LockFreeVRef<C> {
	type Target = C;

	fn deref(&self) -> &Self::Target { self.0.deref().1.as_ref().unwrap() }
}

impl<C: Column + 'static> Decomposition<C> for LockFreeDecomposition<C> {
	type RColRef<'a> = LockFreeRRef<C>;
	type VColRef<'a> = LockFreeVRef<C>;

	fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a> {
		LockFreeRRef(self.0[index].get_ref())
	}

	fn get_v_col<'a>(&'a self, index: usize) -> Result<Self::VColRef<'a>, NoVMatrixError> {
		if self.n_cols() == 0 {
			return Err(NoVMatrixError::EmptyDecompositionError);
		}
		let col_ref = self.0[index].get_ref();
		let has_v = col_ref.1.is_some();
		if has_v {
			Ok(LockFreeVRef(col_ref))
		} else {
			Err(NoVMatrixError::VMatrixDiscardedError)
		}
	}

	fn n_cols(&self) -> usize { self.0.len() }
}

#[cfg(test)]
mod tests {

	use proptest::{collection::hash_set, prelude::*};

	use super::*;
	use crate::{
		algorithms::{Decomposition, SerialAlgorithm},
		columns::{BitSetColumn, BitSetVecHybridColumn, VecColumn},
	};

	proptest! {
		#[test]
		fn lockfree_agrees_with_serial( matrix in sut_matrix(100) ) {
			let options = LoPhatOptions{ clearing: false, ..Default::default() };
			let serial_dgm = SerialAlgorithm::init(Some(options)).add_cols(matrix.iter().cloned()).decompose().diagram();
			let parallel_dgm = LockFreeAlgorithm::init(Some(options)).add_cols(matrix.into_iter()).decompose().diagram();
			assert_eq!(serial_dgm, parallel_dgm);
		}
	}

	proptest! {
		#[test]
		fn hybrid_cols_work( matrix in sut_matrix(100) ) {
			let hybrid_matrix = matrix.iter().map(|col| {
				let mut hybrid_col = BitSetVecHybridColumn::new_with_degree(col.degree());
				hybrid_col.add_entries(col.entries());
				hybrid_col
			});
			let options = LoPhatOptions{ clearing: false, ..Default::default()};
			let hybrid_dgm = LockFreeAlgorithm::init( Some(options)).add_cols(hybrid_matrix).decompose().diagram();
			let vec_dgm = LockFreeAlgorithm::init( Some(options)).add_cols(matrix.into_iter()).decompose().diagram();
			assert_eq!(vec_dgm, hybrid_dgm);
		}
	}

	proptest! {
		#[test]
		fn bit_set_cols_work( matrix in sut_matrix(100) ) {
			let bit_set_matrix = matrix.iter().map(|col| {
				let mut bit_set_col = BitSetColumn::new_with_degree(col.degree());
				bit_set_col.add_entries(col.entries());
				bit_set_col
			});
			let options = LoPhatOptions{ clearing: false, ..Default::default()};
			let bit_set_dgm = LockFreeAlgorithm::init(Some(options)).add_cols(bit_set_matrix).decompose().diagram();
			let vec_dgm = LockFreeAlgorithm::init(Some(options)).add_cols(matrix.into_iter()).decompose().diagram();
			assert_eq!(vec_dgm, bit_set_dgm);
		}
	}

	// Generates a strict upper triangular matrix of VecColumns with given size
	fn sut_matrix(size: usize) -> impl Strategy<Value = Vec<VecColumn>> {
		let mut matrix = vec![];
		for i in 1..size {
			matrix.push(veccolum_with_idxs_below(i));
		}
		matrix
	}

	fn veccolum_with_idxs_below(mut max_idx: usize) -> impl Strategy<Value = VecColumn> {
		// Avoid empty range problem
		// Always returns empty Vec because size is in 0..1 == { 0 }
		if max_idx == 0 {
			max_idx = 1;
		}
		hash_set(0..max_idx, 0..max_idx).prop_map(|set| {
			let mut col: Vec<_> = set.into_iter().collect();
			col.sort();
			VecColumn::from((0, col))
		})
	}
}

fn option_usize_to_usize(opt: Option<usize>) -> usize { opt.unwrap_or(usize::MAX) }

fn usize_to_option_usize(val: usize) -> Option<usize> {
	if val == usize::MAX { None } else { Some(val) }
}

#[cfg(feature = "serde")]
impl_rvd_serialize!(LockFreeDecomposition);
