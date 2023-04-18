use std::cmp::Ordering;

use bit_set::BitSet;

pub enum ColumnMode {
    Working,
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

    fn set_mode(&mut self, _mode: ColumnMode);

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

/// A [`Column`]-implementing struct, representing the column by an increasing vector of the non-zero indices.
///
/// To construct call [`VecColumn::from`] or use [`VecColumn::new_with_dimension`] and [`VecColumn::add_entries`]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VecColumn {
    boundary: Vec<usize>,
    dimension: usize,
}

impl VecColumn {
    // Returns the index where we should try to insert next entry
    fn add_entry_starting_at(&mut self, entry: usize, starting_idx: usize) -> usize {
        let mut working_idx = starting_idx;
        while let Some(value_at_idx) = self.boundary.iter().nth(working_idx) {
            match value_at_idx.cmp(&entry) {
                Ordering::Less => {
                    working_idx += 1;
                    continue;
                }
                Ordering::Equal => {
                    self.boundary.remove(working_idx);
                    return working_idx;
                }
                Ordering::Greater => {
                    self.boundary.insert(working_idx, entry);
                    return working_idx + 1;
                }
            }
        }
        // Bigger than all idxs in col - add to end
        self.boundary.push(entry);
        return self.boundary.len() - 1;
    }
}

impl Column for VecColumn {
    fn pivot(&self) -> Option<usize> {
        self.boundary.iter().last().copied()
    }

    fn add_col(&mut self, other: &Self) {
        let mut working_idx = 0;
        for entry in other.boundary.iter() {
            working_idx = self.add_entry_starting_at(*entry, working_idx);
        }
    }

    fn add_entry(&mut self, entry: usize) {
        self.add_entry_starting_at(entry, 0);
    }

    fn has_entry(&self, entry: &usize) -> bool {
        self.boundary.contains(entry)
    }

    type EntriesIter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn entries<'a>(&'a self) -> Self::EntriesIter<'a> {
        self.boundary.iter().copied()
    }

    type EntriesRepr = Vec<usize>;

    fn set_entries(&mut self, entries: Self::EntriesRepr) {
        self.boundary = entries;
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: usize) {
        self.dimension = dimension;
    }

    fn is_cycle(&self) -> bool {
        self.boundary.is_empty()
    }

    fn new_with_dimension(dimension: usize) -> Self {
        Self {
            boundary: vec![],
            dimension,
        }
    }

    // No difference for this representation
    fn set_mode(&mut self, _mode: ColumnMode) {}
}

impl From<(usize, Vec<usize>)> for VecColumn {
    /// Constructs a `VecColumn`, from a tuple where
    /// `boundary` is the vector of non-zero indices, sorted in increasing order.
    fn from((dimension, boundary): (usize, Vec<usize>)) -> Self {
        Self {
            boundary,
            dimension,
        }
    }
}

/// A [`Column`]-implementing struct, representing the column by a bit set of the non-zero indices.
///
/// To construct call [`BitSetColumn::from`] or use [`BitSetColumn::new_with_dimension`] and [`BitSetColumn::add_entries`]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BitSetColumn {
    boundary: BitSet,
    dimension: usize,
}

impl Column for BitSetColumn {
    fn pivot(&self) -> Option<usize> {
        self.boundary.iter().max()
    }

    fn add_col(&mut self, other: &Self) {
        self.boundary.symmetric_difference_with(&other.boundary);
    }

    fn add_entry(&mut self, entry: usize) {
        if self.has_entry(&entry) {
            self.boundary.remove(entry);
        } else {
            self.boundary.insert(entry);
        }
    }

    fn has_entry(&self, entry: &usize) -> bool {
        self.boundary.contains(*entry)
    }

    type EntriesIter<'a> = bit_set::Iter<'a, u32>;

    fn entries<'a>(&'a self) -> Self::EntriesIter<'a> {
        self.boundary.iter()
    }

    type EntriesRepr = BitSet;

    fn set_entries(&mut self, entries: Self::EntriesRepr) {
        self.boundary = entries;
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn set_dimension(&mut self, dimension: usize) {
        self.dimension = dimension;
    }

    fn is_cycle(&self) -> bool {
        self.boundary.is_empty()
    }

    fn new_with_dimension(dimension: usize) -> Self {
        Self {
            boundary: BitSet::new(),
            dimension,
        }
    }

    // No difference for this representation
    fn set_mode(&mut self, _mode: ColumnMode) {}
}

impl From<(usize, BitSet)> for BitSetColumn {
    /// Constructs a `BitSetColumn`, from a tuple where
    /// `boundary_vec` is the vector of non-zero indices, sorted in increasing order.
    fn from((dimension, boundary): (usize, BitSet)) -> Self {
        Self {
            boundary,
            dimension,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum HybridColumnInternal {
    BitSet(BitSetColumn),
    Vec(VecColumn),
}

enum BitSetVecHybridIter<'a> {
    BitSet(<BitSetColumn as Column>::EntriesIter<'a>),
    Vec(<VecColumn as Column>::EntriesIter<'a>),
}

impl<'a> Iterator for BitSetVecHybridIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BitSetVecHybridIter::BitSet(x) => x.next(),
            BitSetVecHybridIter::Vec(x) => x.next(),
        }
    }
}

impl Default for HybridColumnInternal {
    fn default() -> Self {
        Self::Vec(VecColumn::default())
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct BitSetVecHybridColumn {
    internal: HybridColumnInternal,
}

impl Column for BitSetVecHybridColumn {
    fn pivot(&self) -> Option<usize> {
        match &self.internal {
            HybridColumnInternal::BitSet(x) => x.pivot(),
            HybridColumnInternal::Vec(x) => x.pivot(),
        }
    }

    fn add_col(&mut self, other: &Self) {
        self.add_entries(other.entries())
    }

    fn add_entry(&mut self, entry: usize) {
        match &mut self.internal {
            HybridColumnInternal::BitSet(ref mut x) => x.add_entry(entry),
            HybridColumnInternal::Vec(ref mut x) => x.add_entry(entry),
        }
    }

    fn has_entry(&self, entry: &usize) -> bool {
        match &self.internal {
            HybridColumnInternal::BitSet(x) => x.has_entry(entry),
            HybridColumnInternal::Vec(x) => x.has_entry(entry),
        }
    }

    type EntriesIter<'a> = BitSetVecHybridIter<'a>;

    // No idea what's going on here
    #[allow(implied_bounds_entailment)]
    fn entries<'a>(&'a self) -> Self::EntriesIter<'a> {
        match &self.internal {
            HybridColumnInternal::BitSet(x) => BitSetVecHybridIter::BitSet(x.entries()),
            HybridColumnInternal::Vec(x) => BitSetVecHybridIter::Vec(x.entries()),
        }
    }

    // Since we use this during setup, we use stored version
    type EntriesRepr = Vec<usize>;

    fn set_entries(&mut self, entries: Self::EntriesRepr) {
        self.internal = HybridColumnInternal::Vec(VecColumn::from((self.dimension(), entries)))
    }

    fn dimension(&self) -> usize {
        match &self.internal {
            HybridColumnInternal::BitSet(x) => x.dimension(),
            HybridColumnInternal::Vec(x) => x.dimension(),
        }
    }

    fn set_dimension(&mut self, dimension: usize) {
        match &mut self.internal {
            HybridColumnInternal::BitSet(ref mut x) => x.set_dimension(dimension),
            HybridColumnInternal::Vec(ref mut x) => x.set_dimension(dimension),
        }
    }

    fn set_mode(&mut self, mode: ColumnMode) {
        match (mode, &self.internal) {
            (ColumnMode::Working, HybridColumnInternal::Vec(_)) => {
                let mut set_column = BitSetColumn::new_with_dimension(self.dimension());
                set_column.add_entries(self.entries());
                self.internal = HybridColumnInternal::BitSet(set_column);
            }
            (ColumnMode::Storage, HybridColumnInternal::BitSet(_)) => {
                let mut vec_column = VecColumn::new_with_dimension(self.dimension());
                vec_column.add_entries(self.entries());
                self.internal = HybridColumnInternal::Vec(vec_column);
            }
            _ => return,
        }
    }
}

impl From<(usize, Vec<usize>)> for BitSetVecHybridColumn {
    fn from(value: (usize, Vec<usize>)) -> Self {
        Self {
            internal: HybridColumnInternal::Vec(VecColumn::from(value)),
        }
    }
}
