use std::cmp::Ordering;

use super::{Column, ColumnMode};

/// A column represented by an increasing vector of the non-zero indices.
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
