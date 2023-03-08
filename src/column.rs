use std::cmp::Ordering;

/// Structs implementing `Column` represent columns of a `usize`-indexed matrix,
/// over the field F_2.
pub trait Column: Sync + Clone + Send + Default {
    /// Returns the index of the lowest non-zero column, or `None` if the column is empty.
    fn pivot(&self) -> Option<usize>;
    /// Adds one copy of `other` into `self`
    fn add_col(&mut self, other: &Self);
    /// Should be equivalent to `self.add_col(e_entry)` where `e_entry` is the column
    /// with all zeros except a 1 in index `entry`.
    fn add_entry(&mut self, entry: usize);
}

/// A [`Column`]-implementing struct, representing the column by an increasing vector of the non-zero indices.
///
/// To construct call [`VecColumn::from`].
#[derive(Debug, Default, Clone)]
pub struct VecColumn {
    pub internal: Vec<usize>,
}

impl VecColumn {
    // Returns the index where we should try to insert next entry
    fn add_entry_starting_at(&mut self, entry: usize, starting_idx: usize) -> usize {
        let mut working_idx = starting_idx;
        while let Some(value_at_idx) = self.internal.iter().nth(working_idx) {
            match value_at_idx.cmp(&entry) {
                Ordering::Less => {
                    working_idx += 1;
                    continue;
                }
                Ordering::Equal => {
                    self.internal.remove(working_idx);
                    return working_idx;
                }
                Ordering::Greater => {
                    self.internal.insert(working_idx, entry);
                    return working_idx + 1;
                }
            }
        }
        // Bigger than all idxs in col - add to end
        self.internal.push(entry);
        return self.internal.len() - 1;
    }
}

impl Column for VecColumn {
    fn pivot(&self) -> Option<usize> {
        self.internal.iter().last().copied()
    }

    fn add_entry(&mut self, entry: usize) {
        self.add_entry_starting_at(entry, 0);
    }

    fn add_col(&mut self, other: &Self) {
        let mut working_idx = 0;
        for entry in other.internal.iter() {
            working_idx = self.add_entry_starting_at(*entry, working_idx);
        }
    }
}

impl From<Vec<usize>> for VecColumn {
    /// Constructs a `VecColumn`, consuming `internal`, where
    /// `internal` is the vector of non-zero indices, sorted in increasing order.
    fn from(internal: Vec<usize>) -> Self {
        Self { internal }
    }
}
