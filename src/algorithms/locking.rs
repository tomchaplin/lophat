#[cfg(feature = "serde")]
use crate::impl_rvd_serialize;

use std::ops::Deref;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;

use crate::algorithms::RVDecomposition;
use crate::columns::Column;
use crate::columns::ColumnMode::{Storage, Working};
use crate::options::LoPhatOptions;
use crate::utils::set_mode_of_pair;

use crossbeam::atomic::AtomicCell;
use rayon::prelude::*;
#[cfg(feature = "local_thread_pool")]
use rayon::ThreadPoolBuilder;

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

/// Implements a locking version of the parallel, lockfree algorithm introduced by [Morozov and Nigmetov](https://doi.org/10.1145/3350755.3400244).
/// Rather than using atomic pointers to store columns, each column is stored behind a [`RwLock`](std::sync::RwLock).
/// Also able to employ the clearing optimisation of [Bauer et al.](https://doi.org/10.1007/978-3-319-04099-8_7).
pub struct LockingAlgorithm<C: Column + 'static> {
    matrix: Vec<RwLock<(C, Option<C>)>>,
    pivots: Vec<AtomicCell<Option<usize>>>,
    options: LoPhatOptions,
    thread_pool: LoPhatThreadPool,
    max_dim: usize,
}

impl<'a, C: Column> LockingAlgorithm<C> {
    /// Initialise atomic data structure with provided `matrix`, store algorithm options and init thread pool.
    fn new(matrix: impl Iterator<Item = C>, options: LoPhatOptions) -> Self {
        let mut max_dim = 0;
        let matrix: Vec<_> = matrix
            .enumerate()
            .map(|(idx, r_col)| {
                max_dim = max_dim.max(r_col.dimension());
                if options.maintain_v {
                    let mut v_col = C::new_with_dimension(r_col.dimension());
                    v_col.add_entry(idx);
                    RwLock::new((r_col, Some(v_col)))
                } else {
                    RwLock::new((r_col, None))
                }
            })
            .collect();
        let column_height = options.column_height.unwrap_or(matrix.len());
        let pivots: Vec<_> = (0..column_height).map(|_| AtomicCell::new(None)).collect();
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
        // Return options
        Self {
            matrix,
            pivots,
            options,
            thread_pool,
            max_dim,
        }
    }

    /// Return a column with index `l`, if one exists.
    /// If found, returns `(col_idx, col)`, where col is a tuple consisting of the corresponding column in R and V.
    /// If not maintaining V, second entry of tuple is `None`.
    pub fn get_col_with_pivot(
        &'a self,
        l: usize,
    ) -> Option<(usize, RwLockReadGuard<'a, (C, Option<C>)>)> {
        loop {
            let piv = self.pivots[l].load();
            if let Some(piv) = piv {
                let cols = self.matrix[piv].read().unwrap();
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
            // without locking other threads from reading
            let mut curr_column = self.matrix[working_j].read().unwrap().clone();
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
                            curr_v_col.add_col(&piv_column.1.as_ref().unwrap());
                        }
                    } else if piv > working_j {
                        self.write_to_matrix(working_j, curr_column);
                        if self.pivots[l]
                            .compare_exchange(Some(piv), Some(working_j))
                            .is_ok()
                        {
                            working_j = piv;
                        }
                        continue 'outer;
                    } else {
                        panic!()
                    }
                } else {
                    // piv = -1 case
                    self.write_to_matrix(working_j, curr_column);
                    if self.pivots[l]
                        .compare_exchange(None, Some(working_j))
                        .is_ok()
                    {
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

    // Write to matrix; might lock until no read locks present
    // Make sure write lock is dropped quickly
    fn write_to_matrix(&self, index: usize, mut to_write: (C, Option<C>)) {
        set_mode_of_pair(&mut to_write, Storage);
        let mut in_matrix = self.matrix[index].write().unwrap();
        *in_matrix = to_write;
    }

    /// Uses the boundary built up in column `boudary_idx` to clear the column corresponding to its pivot
    pub fn clear_with_column(&self, boudary_idx: usize) {
        let boundary = self.matrix[boudary_idx].read().unwrap();
        let boundary_r = &boundary.0;
        let clearing_idx = boundary_r
            .pivot()
            .expect("Attempted to clear using cycle column");
        let clearing_dimension = self.matrix[clearing_idx].read().unwrap().0.dimension();
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
                .filter(|&j| self.matrix[j].read().unwrap().0.dimension() == dimension)
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
                .filter(|&j| self.matrix[j].read().unwrap().0.dimension() == dimension)
                .filter(|&j| self.matrix[j].read().unwrap().0.is_boundary())
                .for_each(|j| self.clear_with_column(j));
        });
    }

    /// Reduce all columns in parallel, according to `options`.
    pub fn reduce(&self) {
        for dimension in (0..=self.max_dim).rev() {
            self.reduce_dimension(dimension);
            if self.options.clearing && dimension > 0 {
                self.clear_dimension(dimension)
            }
        }
    }
}

pub struct LockingRRef<'a, C>(RwLockReadGuard<'a, (C, Option<C>)>);

impl<'a, C> Deref for LockingRRef<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.deref().0
    }
}

pub struct LockingVRef<'a, C>(RwLockReadGuard<'a, (C, Option<C>)>);

impl<'a, C> Deref for LockingVRef<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.deref().1.as_ref().unwrap()
    }
}

impl<C: Column + 'static> RVDecomposition<C> for LockingAlgorithm<C> {
    type RColRef<'a> = LockingRRef<'a, C> where Self : 'a;
    fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a> {
        LockingRRef(self.matrix[index].read().unwrap())
    }

    type VColRef<'a> = LockingVRef<'a, C> where Self : 'a;
    fn get_v_col<'a>(&'a self, index: usize) -> Option<Self::VColRef<'a>> {
        self.options
            .maintain_v
            .then_some(LockingVRef(self.matrix[index].read().unwrap()))
    }

    fn n_cols(&self) -> usize {
        self.matrix.len()
    }

    type Options = LoPhatOptions;

    fn decompose(matrix: impl Iterator<Item = C>, options: Option<Self::Options>) -> Self {
        let options = options.unwrap_or_default();
        let algo = LockingAlgorithm::new(matrix, options);
        algo.reduce();
        algo
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::algorithms::SerialAlgorithm;
    use crate::columns::VecColumn;
    use proptest::collection::hash_set;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn locking_agrees_with_serial( matrix in sut_matrix(100) ) {
            let options = LoPhatOptions::default();
            let serial_dgm = SerialAlgorithm::decompose(matrix.iter().cloned(), Some(options)).diagram();
            let parallel_dgm = LockingAlgorithm::decompose(matrix.into_iter(), Some(options)).diagram();
            assert_eq!(serial_dgm, parallel_dgm);
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

#[cfg(feature = "serde")]
impl_rvd_serialize!(LockingAlgorithm);
