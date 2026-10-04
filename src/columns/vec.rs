use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Column, ColumnMode};

/// A sparse column stored as a strictly increasing vector of nonzero row
/// indices.
///
/// Construction from `(degree, entries)` and [`Column::set_entries`] take the
/// vector unchanged: callers must sort it and remove duplicates first. To
/// toggle arbitrary indices modulo two, construct an empty column with
/// [`Column::new_with_degree`] and use [`Column::add_entries`]. Iteration is in
/// increasing order, and [`ColumnMode`] hints do not change the representation.
///
/// # Example
///
/// ```
/// use lophat::columns::{Column, VecColumn};
///
/// let mut column = VecColumn::from((1, vec![0, 2]));
/// column.add_entry(2);
/// assert_eq!(column.entries().collect::<Vec<_>>(), vec![0]);
/// assert_eq!(column.degree(), 1);
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct VecColumn {
	boundary: Vec<usize>,
	degree: usize,
}

impl VecColumn {
	// Returns the index where we should try to insert next entry
	fn add_entry_starting_at(&mut self, entry: usize, starting_idx: usize) -> usize {
		let mut working_idx = starting_idx;
		while let Some(value_at_idx) = self.boundary.get(working_idx) {
			match value_at_idx.cmp(&entry) {
				Ordering::Less => {
					working_idx += 1;
					continue;
				},
				Ordering::Equal => {
					self.boundary.remove(working_idx);
					return working_idx;
				},
				Ordering::Greater => {
					self.boundary.insert(working_idx, entry);
					return working_idx + 1;
				},
			}
		}
		// Bigger than all idxs in col - add to end
		self.boundary.push(entry);
		self.boundary.len() - 1
	}
}

impl Column for VecColumn {
	type EntriesIter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;
	type EntriesRepr = Vec<usize>;

	fn pivot(&self) -> Option<usize> { self.boundary.iter().last().copied() }

	fn add_col(&mut self, other: &Self) {
		let mut working_idx = 0;
		for entry in other.boundary.iter() {
			working_idx = self.add_entry_starting_at(*entry, working_idx);
		}
	}

	fn add_entry(&mut self, entry: usize) { self.add_entry_starting_at(entry, 0); }

	fn has_entry(&self, entry: &usize) -> bool { self.boundary.contains(entry) }

	fn entries<'a>(&'a self) -> Self::EntriesIter<'a> { self.boundary.iter().copied() }

	fn set_entries(&mut self, entries: Self::EntriesRepr) { self.boundary = entries; }

	fn degree(&self) -> usize { self.degree }

	fn set_degree(&mut self, degree: usize) { self.degree = degree; }

	fn is_cycle(&self) -> bool { self.boundary.is_empty() }

	fn new_with_degree(degree: usize) -> Self {
		Self {
			boundary: vec![],
			degree,
		}
	}

	// No difference for this representation
	fn set_mode(&mut self, _mode: ColumnMode) {}
}

impl From<(usize, Vec<usize>)> for VecColumn {
	/// Take ownership of `(degree, boundary)` without sorting or deduplicating.
	///
	/// `boundary` must be strictly increasing and contain no duplicates.
	fn from((degree, boundary): (usize, Vec<usize>)) -> Self { Self { boundary, degree } }
}
