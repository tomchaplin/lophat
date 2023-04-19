use super::{BitSetColumn, Column, ColumnMode, VecColumn};

#[derive(Debug, Clone, PartialEq)]
enum HybridColumnInternal {
    BitSet(BitSetColumn),
    Vec(VecColumn),
}

pub enum BitSetVecHybridIter<'a> {
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

/// A hybrid column which changes representation depending on the current [`ColumnMode`].
///
/// * During [`ColumnMode::Working`], the representation is as a [`BitSetColumn`].
/// * During [`ColumnMode::Storage`], the representation is as a [`VecColumn`].
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BitSetVecHybridColumn {
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
        // We do this because it is assumes you are adding a Vec into a BitSet
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
