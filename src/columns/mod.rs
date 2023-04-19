//! Representations of columns of a Z_2 matrix, complying to a common interface.

mod bit_set;
mod hybrid;
mod vec;

pub use self::bit_set::BitSetColumn;
pub use hybrid::BitSetVecHybridColumn;
pub use vec::VecColumn;

/// Enum representing the differnt modes that the column is in, which may impact the optimal representation.
pub enum ColumnMode {
    /// A column in this mode is about to be mutated regularly, e.g. through [`add_col`](Column::add_col).
    Working,
    /// A column in this mode will not be mutated much but may be regularly read from.
    Storage,
}

/// Structs implementing `Column` represent columns of a `usize`-indexed matrix,
/// over the finite field F_2.
///
/// Note the requirement to implement `From<(usize, Self::EntriesRepr)>`.
/// The `usize` is the dimension and `Self::EntriesRepr` is the entries in the column.
pub trait Column: Sync + Clone + Send + From<(usize, Self::EntriesRepr)> {
    /// Returns the index of the lowest non-zero column, or `None` if the column is empty.
    fn pivot(&self) -> Option<usize>;
    /// Adds one copy of `other` into `self`
    fn add_col(&mut self, other: &Self);
    /// Should be equivalent to `self.add_col(e_entry)` where `e_entry` is the column
    /// with all zeros except a 1 in index `entry`.
    fn add_entry(&mut self, entry: usize);
    /// Return whether or not entry appears with value 1 in the column
    fn has_entry(&self, entry: &usize) -> bool;
    /// The output type of [`Self::entries`]
    type EntriesIter<'a>: Iterator<Item = usize>
    where
        Self: 'a;
    /// Returns the entries of the columns as an iterator over the non-zero indices (not necessarily sorted)
    fn entries<'a>(&'a self) -> Self::EntriesIter<'a>;
    /// A format that the user can provide the entries of the column in, in order to efficiently construct the column.
    /// The `Default` should correspond to the empty column
    type EntriesRepr: Default;
    /// Efficiently override the column, by providing entries in the internal format.
    fn set_entries(&mut self, entries: Self::EntriesRepr);
    /// Return the dimension of this column (assuming the matrix represents a chain complex boundary matrix)
    fn dimension(&self) -> usize;
    /// Change column to provided dimension
    fn set_dimension(&mut self, dimension: usize);

    /// Change the underlying representation of the column to optimise it for the corresponding `mode`.
    /// Only relevant for certain representations.
    fn set_mode(&mut self, mode: ColumnMode);

    /// Returns whether or not the column is a cycle, i.e. has no entries.
    /// Provided implementation makes call to [`Self::pivot`].
    /// You may wish to provide a more efficient implementation
    fn is_cycle(&self) -> bool {
        self.pivot().is_none()
    }

    /// Returns whether or not the column is a boundary, i.e. is non-empty.
    /// Provided implementation negates [`Self::is_cycle`]
    fn is_boundary(&self) -> bool {
        !self.is_cycle()
    }

    /// Uses [`Self::add_entry`] to add elements from the iterator to the column
    fn add_entries<B: Iterator<Item = usize>>(&mut self, entries: B) {
        for entry in entries {
            self.add_entry(entry);
        }
    }

    /// Init an empty column with the given dimension
    fn new_with_dimension(dimension: usize) -> Self {
        Self::from((dimension, Self::EntriesRepr::default()))
    }

    /// Removes all entries from the column
    fn clear_entries(&mut self) {
        self.set_entries(Self::EntriesRepr::default())
    }
}
