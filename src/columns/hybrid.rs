use super::{BitSetColumn, Column, ColumnMode, VecColumn};

#[derive(Debug, Clone, PartialEq)]
enum HybridColumnInternal {
	BitSet(BitSetColumn),
	Vec(VecColumn),
}

/// Borrowed iterator over the nonzero row indices of a hybrid column.
pub enum BitSetVecHybridIter<'a> {
	/// Iterate over the bit set used while the column is in working mode.
	BitSet(<BitSetColumn as Column>::EntriesIter<'a>),
	/// Iterate over the sorted vector used while the column is in storage mode.
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
	fn default() -> Self { Self::Vec(VecColumn::default()) }
}

/// A sparse column that switches representation in response to [`ColumnMode`].
///
/// * During [`ColumnMode::Working`], the representation is as a
///   [`BitSetColumn`].
/// * During [`ColumnMode::Storage`], the representation is as a [`VecColumn`].
///
/// Construct from `(degree, entries)` using a strictly increasing vector of
/// distinct row indices. Construction and [`Column::set_entries`] select vector
/// storage without normalizing the input. Switching modes preserves the chain
/// degree and coefficients. Both representations iterate in increasing order.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BitSetVecHybridColumn {
	internal: HybridColumnInternal,
}

impl Column for BitSetVecHybridColumn {
	type EntriesIter<'a> = BitSetVecHybridIter<'a>;
	// Since we use this during setup, we use stored version
	type EntriesRepr = Vec<usize>;

	fn pivot(&self) -> Option<usize> {
		match &self.internal {
			HybridColumnInternal::BitSet(x) => x.pivot(),
			HybridColumnInternal::Vec(x) => x.pivot(),
		}
	}

	fn add_col(&mut self, other: &Self) {
		// We do this because it is assumes you are adding a Vec into a BitSet
		// therefore no way to optimise over consuming the iterator
		self.add_entries(other.entries())
	}

	fn add_entry(&mut self, entry: usize) {
		match &mut self.internal {
			HybridColumnInternal::BitSet(x) => x.add_entry(entry),
			HybridColumnInternal::Vec(x) => x.add_entry(entry),
		}
	}

	fn has_entry(&self, entry: &usize) -> bool {
		match &self.internal {
			HybridColumnInternal::BitSet(x) => x.has_entry(entry),
			HybridColumnInternal::Vec(x) => x.has_entry(entry),
		}
	}

	fn entries<'a>(&'a self) -> Self::EntriesIter<'a> {
		match &self.internal {
			HybridColumnInternal::BitSet(x) => BitSetVecHybridIter::BitSet(x.entries()),
			HybridColumnInternal::Vec(x) => BitSetVecHybridIter::Vec(x.entries()),
		}
	}

	fn set_entries(&mut self, entries: Self::EntriesRepr) {
		self.internal = HybridColumnInternal::Vec(VecColumn::from((self.degree(), entries)))
	}

	fn degree(&self) -> usize {
		match &self.internal {
			HybridColumnInternal::BitSet(x) => x.degree(),
			HybridColumnInternal::Vec(x) => x.degree(),
		}
	}

	fn set_degree(&mut self, degree: usize) {
		match &mut self.internal {
			HybridColumnInternal::BitSet(x) => x.set_degree(degree),
			HybridColumnInternal::Vec(x) => x.set_degree(degree),
		}
	}

	fn set_mode(&mut self, mode: ColumnMode) {
		match (mode, &self.internal) {
			(ColumnMode::Working, HybridColumnInternal::Vec(_)) => {
				let mut set_column = BitSetColumn::new_with_degree(self.degree());
				set_column.add_entries(self.entries());
				self.internal = HybridColumnInternal::BitSet(set_column);
			},
			(ColumnMode::Storage, HybridColumnInternal::BitSet(_)) => {
				let mut vec_column = VecColumn::new_with_degree(self.degree());
				vec_column.add_entries(self.entries());
				self.internal = HybridColumnInternal::Vec(vec_column);
			},
			_ => (),
		}
	}
}

impl From<(usize, Vec<usize>)> for BitSetVecHybridColumn {
	/// Take ownership of a degree and sorted, distinct indices in vector
	/// storage.
	fn from(value: (usize, Vec<usize>)) -> Self {
		Self {
			internal: HybridColumnInternal::Vec(VecColumn::from(value)),
		}
	}
}
