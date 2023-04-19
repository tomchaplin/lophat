//! Implementations of various algorithms for computing persistent homology.
//!
//! Each algorithm is encapsulated in a struct and the main interface to these structs is the [`RVDecomposition`] trait.
//! By providing appropriate options during construction, each algorithm can also maintain V in the R=DV decomposition.

use crate::{columns::Column, utils::PersistenceDiagram};
use hashbrown::HashSet;
use std::ops::Deref;

mod lock_free;
mod locking;
mod serial;

pub use lock_free::LockFreeAlgorithm;
pub use locking::LockingAlgorithm;
pub use serial::SerialAlgorithm;

/// A struct implementing this trait implements an algorithm for the R=DV decomposition of a matrix.
/// The struct is produced by calling [`decompose`](RVDecomposition::decompose).
///
/// You must implement [`decompose`](RVDecomposition::decompose) so that it runs the algorithm on the provided matrix.
/// You must also implement a number of methods to query columns of both R and V in the resulting decomposition.
/// Once these query methods are implemented, the trait provides a method for computing the persistence diagram from the decomposition.
pub trait RVDecomposition<C>
where
    C: Column,
{
    /// A struct of options that you wish to provide to the algorithm.
    type Options: Default + Copy;

    /// Decomposes the input `matrix` (D) into an R=DV decomposition, following the relevant algorithm and according to `options`.
    /// The input `matrix` is provided as an iterator over its columns.
    /// If no options are provided you should fall back to `Self::Options::default()`.
    fn decompose(matrix: impl Iterator<Item = C>, options: Option<Self::Options>) -> Self;

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
    /// Returns `None` if V was not maintained by the algorithm.
    fn get_v_col<'a>(&'a self, index: usize) -> Option<Self::VColRef<'a>>;

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
}
