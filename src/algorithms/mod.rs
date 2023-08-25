//! Implementations of various algorithms for computing persistent homology.
//!
//! Each algorithm is encapsulated in a struct and the main interface to these structs is the [`DecompositionAlgo`] trait.
//! By providing appropriate options during construction, each algorithm can also maintain V in the R=DV decomposition.

use crate::{columns::Column, utils::PersistenceDiagram};
use hashbrown::HashSet;
use std::ops::Deref;

mod lock_free;
mod locking;
mod serial;

pub use lock_free::{LockFreeAlgorithm, LockFreeDecomposition};
pub use locking::{LockingAlgorithm, LockingDecomposition};
pub use serial::{SerialAlgorithm, SerialDecomposition};

/// Error type returned when attempting to query a column of V from a decomposition in which V was not maintained.
#[derive(Debug)]
pub struct NoVMatrixError;

/// A struct implementing this trait represents the output of an R=DV decomposition of a matrix D and is typically constructed by [`DecompositionAlgo::decompose`].
///
/// The main required methods are [`get_r_col`](Decomposition::get_r_col) and [`get_v_col`](Decomposition::get_v_col), which return immutable references to columns of the R and V matrix respectively.
/// Given these methods, the persistence diagram can be computed via the provided [`diagram`](Decomposition::diagram) method.
pub trait Decomposition<C>
where
    C: Column,
{
    /// Return type of [`get_r_col`](Self::get_r_col), typically `&'a C`.
    type RColRef<'a>: Deref<Target = C> + 'a
    where
        Self: 'a;
    /// Returns a reference to the column in position `index` of R, in the decomposition
    fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a>;

    /// Return type of [`get_v_col`](Self::get_v_col), typically `&'a C`.
    type VColRef<'a>: Deref<Target = C> + 'a
    where
        Self: 'a;
    /// Returns a reference to the column in position `index` of V, in the decomposition.
    /// Returns `NoVMatrixError` if V was not maintained by the algorithm.
    fn get_v_col<'a>(&'a self, index: usize) -> Result<Self::VColRef<'a>, NoVMatrixError>;

    /// Returns the number of column in R (equal to the number of columns in D).
    fn n_cols(&self) -> usize;

    /// Uses the methods implemented by this trait to read-off the column pairings which constiute the persistence diagram.
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

    /// By checking whether `self.get_v_col(0)` returns an error, determines whether the V matrix was maintained for this decomposition.
    fn has_v(&self) -> bool {
        // If n_cols is zero then it may as well have v
        // Otherwise we just check whether we can get the first v column
        self.n_cols() == 0 || self.get_v_col(0).is_ok()
    }
}

/// A struct implementing this trait implements an algorithm for computing the R=DV decomposition of a matrix D.
///
/// The struct is initialised via the [`init`](DecompositionAlgo::init) method, in which options for the algorithm are provided.
/// The D matrix is build up by using the [`add_cols`](DecompositionAlgo::add_cols) and [`add_entries`](DecompositionAlgo::add_entries) methods.
/// Once constructed, the decomposition is computed via the [`decompose`](DecompositionAlgo::decompose) methods
pub trait DecompositionAlgo<C>
where
    C: Column,
{
    /// A struct of options that you wish to provide to the algorithm.
    type Options: Default + Copy;
    /// Initialise the algorithm with the options provided and an empty input matrix
    fn init(options: Option<Self::Options>) -> Self;

    /// Push the provided columns onto the end of the matrix
    fn add_cols(self, cols: impl Iterator<Item = C>) -> Self;

    /// Add the provided (row, column) entries to the matrix.
    /// If the column has not already been pushed via [`add_cols`](DecompositionAlgo::add_cols) then `panic!()`
    fn add_entries(self, entries: impl Iterator<Item = (usize, usize)>) -> Self;

    /// Return tupe of [`decompose`](DecompositionAlgo::decompose) -- should carry sufficient information to query columns of the resulting decomposition.
    type Decomposition: Decomposition<C>;
    /// Decomposes the built-up matrix (D) into an R=DV decomposition, following the relevant algorithm and provided options.
    fn decompose(self) -> Self::Decomposition;
}
