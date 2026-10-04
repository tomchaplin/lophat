use bit_set::BitSet;

use super::{Column, ColumnMode};
/// A sparse column stored as a bit set of nonzero row indices.
///
/// Construct from `(degree, bit_set)` or use [`Column::new_with_degree`] and
/// [`Column::add_entries`]. The bit set enforces unique indices and iterates in
/// increasing order. [`ColumnMode`] hints do not change this representation.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BitSetColumn {
	boundary: BitSet,
	degree: usize,
}

impl Column for BitSetColumn {
	type EntriesIter<'a> = bit_set::Iter<'a, u32>;
	type EntriesRepr = BitSet;

	fn pivot(&self) -> Option<usize> { self.boundary.iter().max() }

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

	fn has_entry(&self, entry: &usize) -> bool { self.boundary.contains(*entry) }

	fn entries<'a>(&'a self) -> Self::EntriesIter<'a> { self.boundary.iter() }

	fn set_entries(&mut self, entries: Self::EntriesRepr) { self.boundary = entries; }

	fn degree(&self) -> usize { self.degree }

	fn set_degree(&mut self, degree: usize) { self.degree = degree; }

	fn is_cycle(&self) -> bool { self.boundary.is_empty() }

	fn new_with_degree(degree: usize) -> Self {
		Self {
			boundary: BitSet::new(),
			degree,
		}
	}

	// No difference for this representation
	fn set_mode(&mut self, _mode: ColumnMode) {}
}

impl From<(usize, BitSet)> for BitSetColumn {
	/// Take ownership of a chain degree and a bit set of nonzero row indices.
	fn from((degree, boundary): (usize, BitSet)) -> Self { Self { boundary, degree } }
}
