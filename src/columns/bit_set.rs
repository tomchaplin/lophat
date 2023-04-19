use bit_set::BitSet;

use super::{Column, ColumnMode};
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
