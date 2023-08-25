use std::ops::Deref;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, Release};

#[cfg(feature = "serde")]
use crate::impl_rvd_serialize;

use crate::columns::Column;
use crate::columns::ColumnMode::{Storage, Working};
use crate::options::LoPhatOptions;
use crate::utils::set_mode_of_pair;

use pinboard::GuardedRef;
use pinboard::NonEmptyPinboard;
use rayon::prelude::*;
#[cfg(feature = "local_thread_pool")]
use rayon::ThreadPoolBuilder;

use super::{Decomposition, DecompositionAlgo, NoVMatrixError};

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

/// Implements the parallel, lockfree algorithm introduced by [Morozov and Nigmetov](https://doi.org/10.1145/3350755.3400244).
/// Also able to employ the clearing optimisation of [Bauer et al.](https://doi.org/10.1007/978-3-319-04099-8_7).
pub struct LockFreeAlgorithm<C: Column + 'static> {
    matrix: Vec<NonEmptyPinboard<(C, Option<C>)>>,
    // NOTE: We use `usize::MAX` as a sentinel value, meaning no pivot.
    pivots: Vec<AtomicUsize>,
    options: LoPhatOptions,
    thread_pool: LoPhatThreadPool,
    max_dim: usize,
}

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

    /// Return a column with index `l`, if one exists.
    /// If found, returns `(col_idx, col)`, where col is a tuple consisting of the corresponding column in R and V.
    /// If not maintaining V, second entry of tuple is `None`.
    pub fn get_col_with_pivot(&self, l: usize) -> Option<(usize, GuardedRef<(C, Option<C>)>)> {
        loop {
            let piv = self.get_pivot(l);
            if let Some(piv) = piv {
                let cols = self.matrix[piv].get_ref();
                if cols.0.pivot() != Some(l) {
                    // Got a column but it now has the wrong pivot; loop again.
                    continue;
                };
                // Get column with correct pivot, return to caller.
                return Some((piv, cols));
            } else {
                // There is not yet a column with this pivot, inform caller.
                return None;
            }
        }
    }

    /// Reduces the `j`th column of the matrix as far as possible.
    /// If a pivot is found to the right of `j` (e.g. redued by another thread)
    /// then will switch to reducing that column.
    /// It is safe to reduce all columns in parallel.
    pub fn reduce_column(&self, j: usize) {
        let mut working_j = j;
        'outer: loop {
            // We make a copy of the column because we want to mutate our local copy
            let mut curr_column = self.matrix[working_j].read();
            set_mode_of_pair(&mut curr_column, Working);
            while let Some(l) = (&curr_column).0.pivot() {
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
            if (&curr_column.0).is_cycle() {
                self.write_to_matrix(working_j, curr_column);
                return;
            }
        }
    }

    fn write_to_matrix(&self, index: usize, mut to_write: (C, Option<C>)) {
        set_mode_of_pair(&mut to_write, Storage);
        self.matrix[index].set(to_write);
    }

    /// Uses the boundary built up in column `boudary_idx` to clear the column corresponding to its pivot
    pub fn clear_with_column(&self, boudary_idx: usize) {
        let boundary = self.matrix[boudary_idx].get_ref();
        let boundary_r = &boundary.0;
        let clearing_idx = boundary_r
            .pivot()
            .expect("Attempted to clear using cycle column");
        let clearing_dimension = self.matrix[clearing_idx].get_ref().0.dimension();
        // The cleared R column is empty
        let r_col = C::new_with_dimension(clearing_dimension);
        // The corresponding V column should be the R column of the boundary
        let v_col = self.options.maintain_v.then(|| {
            let mut br = boundary_r.clone();
            br.set_dimension(clearing_dimension);
            br
        });
        self.write_to_matrix(clearing_idx, (r_col, v_col));
    }

    /// Reduce all columns of given dimension in parallel, according to `options`.
    pub fn reduce_dimension(&self, dimension: usize) {
        // Reduce matrix for columns of that dimension
        self.thread_pool.install(|| {
            (0..self.matrix.len())
                .into_par_iter()
                .with_min_len(self.options.min_chunk_len)
                .filter(|&j| self.matrix[j].get_ref().0.dimension() == dimension)
                .for_each(|j| self.reduce_column(j));
        });
    }

    /// Clear all columns of given dimension in parallel
    pub fn clear_dimension(&self, dimension: usize) {
        // Reduce matrix for columns of that dimension
        self.thread_pool.install(|| {
            (0..self.matrix.len())
                .into_par_iter()
                .with_min_len(self.options.min_chunk_len)
                .filter(|&j| self.matrix[j].get_ref().0.dimension() == dimension)
                .filter(|&j| self.matrix[j].get_ref().0.is_boundary())
                .for_each(|j| self.clear_with_column(j));
        });
    }
}

impl<C: Column> DecompositionAlgo<C> for LockFreeAlgorithm<C> {
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
            self.max_dim = self.max_dim.max(r_col.dimension());
            if self.options.maintain_v {
                let mut v_col = C::new_with_dimension(r_col.dimension());
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

    type Decomposition = LockFreeDecomposition<C>;

    fn decompose(mut self) -> Self::Decomposition {
        // Setup pivots vector
        let column_height = self.options.column_height.unwrap_or(self.matrix.len());
        self.pivots = (0..column_height)
            .map(|_| AtomicUsize::new(usize::MAX))
            .collect();
        // Decompose
        for dimension in (0..=self.max_dim).rev() {
            self.reduce_dimension(dimension);
            if self.options.clearing && dimension > 0 {
                self.clear_dimension(dimension)
            }
        }
        LockFreeDecomposition(self.matrix)
    }
}

/// Return type of [`LockFreeAlgorithm`].
pub struct LockFreeDecomposition<C: Column + 'static>(Vec<NonEmptyPinboard<(C, Option<C>)>>);

pub struct LockFreeRRef<C>(GuardedRef<(C, Option<C>)>);

impl<C> Deref for LockFreeRRef<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.deref().0
    }
}

pub struct LockFreeVRef<C>(GuardedRef<(C, Option<C>)>);

impl<C> Deref for LockFreeVRef<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.deref().1.as_ref().unwrap()
    }
}

impl<C: Column + 'static> Decomposition<C> for LockFreeDecomposition<C> {
    type RColRef<'a> = LockFreeRRef<C>;
    fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a> {
        LockFreeRRef(self.0[index].get_ref())
    }

    type VColRef<'a> = LockFreeVRef<C>;
    fn get_v_col<'a>(&'a self, index: usize) -> Result<Self::VColRef<'a>, NoVMatrixError> {
        let col_ref = self.0[index].get_ref();
        let has_v = col_ref.1.is_some();
        if has_v {
            Ok(LockFreeVRef(col_ref))
        } else {
            Err(NoVMatrixError)
        }
    }

    fn n_cols(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::algorithms::Decomposition;
    use crate::algorithms::SerialAlgorithm;
    use crate::columns::{BitSetColumn, BitSetVecHybridColumn, VecColumn};
    use proptest::collection::hash_set;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn lockfree_agrees_with_serial( matrix in sut_matrix(100) ) {
            let options = LoPhatOptions::default();
            let serial_dgm = SerialAlgorithm::init(Some(options)).add_cols(matrix.iter().cloned()).decompose().diagram();
            let parallel_dgm = LockFreeAlgorithm::init(Some(options)).add_cols(matrix.into_iter()).decompose().diagram();
            assert_eq!(serial_dgm, parallel_dgm);
        }
    }

    proptest! {
        #[test]
        fn hybrid_cols_work( matrix in sut_matrix(100) ) {
            let hybrid_matrix = matrix.iter().map(|col| {
                let mut hybrid_col = BitSetVecHybridColumn::new_with_dimension(col.dimension());
                hybrid_col.add_entries(col.entries());
                hybrid_col
            });
            let options = LoPhatOptions::default();
            let hybrid_dgm = LockFreeAlgorithm::init( Some(options)).add_cols(hybrid_matrix).decompose().diagram();
            let vec_dgm = LockFreeAlgorithm::init( Some(options)).add_cols(matrix.into_iter()).decompose().diagram();
            assert_eq!(vec_dgm, hybrid_dgm);
        }
    }

    proptest! {
        #[test]
        fn bit_set_cols_work( matrix in sut_matrix(100) ) {
            let bit_set_matrix = matrix.iter().map(|col| {
                let mut bit_set_col = BitSetColumn::new_with_dimension(col.dimension());
                bit_set_col.add_entries(col.entries());
                bit_set_col
            });
            let options = LoPhatOptions::default();
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

fn option_usize_to_usize(opt: Option<usize>) -> usize {
    opt.unwrap_or(usize::MAX)
}

fn usize_to_option_usize(val: usize) -> Option<usize> {
    if val == usize::MAX {
        None
    } else {
        Some(val)
    }
}

#[cfg(feature = "serde")]
impl_rvd_serialize!(LockFreeDecomposition);
