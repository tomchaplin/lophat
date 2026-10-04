//! Sparse column representations for matrices over the field with two elements.
//!
//! Each column stores its chain degree and nonzero row indices. Adding columns
//! or entries toggles coefficients modulo two. Implementations differ in their
//! storage and iteration costs but share the [`Column`] interface.

mod bit_set;
mod hybrid;
mod vec;

pub use hybrid::BitSetVecHybridColumn;
pub use vec::VecColumn;

pub use self::bit_set::BitSetColumn;

/// Hint describing how a column will be used, so its storage can be optimized.
///
/// [`BitSetVecHybridColumn`] changes representation in response to this hint.
/// [`VecColumn`] and [`BitSetColumn`] keep their representations unchanged.
#[derive(Debug, Clone, Copy)]
pub enum ColumnMode {
	/// Prefer storage for repeated mutation, such as [`Column::add_col`].
	Working,
	/// Prefer storage for repeated reads and infrequent mutation.
	Storage,
}

/// A sparse matrix column over the field with two elements, labelled by chain
/// degree.
///
/// Nonzero coefficients are identified by `usize` row indices. Column addition
/// is symmetric difference: an index appearing in both operands disappears.
/// The pivot is the greatest nonzero row index. Algebraic operations preserve
/// the column's degree unless [`Self::set_degree`] is called explicitly.
///
/// Implementations must support construction from `(degree, entries)` through
/// [`From`]. The native [`Self::EntriesRepr`] may have representation-specific
/// requirements; vectors must be strictly increasing and contain no duplicates.
pub trait Column: Sync + Clone + Send + From<(usize, Self::EntriesRepr)> {
	/// Return the greatest nonzero row index, or `None` for an empty column.
	fn pivot(&self) -> Option<usize>;
	/// Add `other` modulo two, toggling its nonzero entries in this column.
	///
	/// The degree of `self` is unchanged.
	fn add_col(&mut self, other: &Self);
	/// Toggle the coefficient at row `entry`, leaving the degree unchanged.
	///
	/// Equivalent to adding a column whose only nonzero row is `entry`.
	fn add_entry(&mut self, entry: usize);
	/// Return whether the coefficient at row `entry` is one.
	fn has_entry(&self, entry: &usize) -> bool;
	/// Iterator over the nonzero row indices, borrowing this column.
	type EntriesIter<'a>: Iterator<Item = usize>
	where
		Self: 'a;
	/// Iterate over nonzero row indices without modifying the column.
	///
	/// The trait does not require a particular iteration order.
	fn entries<'a>(&'a self) -> Self::EntriesIter<'a>;
	/// Native input format for construction and [`Self::set_entries`].
	///
	/// Its default value must represent an empty column. Callers must satisfy
	/// the representation's ordering and uniqueness requirements.
	type EntriesRepr: Default;
	/// Replace all entries with the supplied native representation.
	///
	/// Preserves the degree. This replaces coefficients rather than toggling
	/// them; it does not promise to validate or normalize the input.
	fn set_entries(&mut self, entries: Self::EntriesRepr);
	/// Return the chain degree attached to this column.
	fn degree(&self) -> usize;
	/// Set the chain degree without changing the nonzero entries.
	fn set_degree(&mut self, degree: usize);

	/// Optimize storage for `mode`, preserving the degree and coefficients.
	///
	/// Implementations with a fixed representation may ignore the hint.
	fn set_mode(&mut self, mode: ColumnMode);

	/// Return whether the column is empty, as for a cycle in a reduced boundary
	/// matrix.
	///
	/// The default implementation checks [`Self::pivot`]. This is a statement
	/// about the stored entries, not an independent homological calculation.
	fn is_cycle(&self) -> bool { self.pivot().is_none() }

	/// Return whether the column is nonempty, by negating [`Self::is_cycle`].
	///
	/// In a reduced boundary matrix, such a column records a finite pairing.
	/// The method does not check whether an arbitrary chain is a boundary.
	fn is_boundary(&self) -> bool { !self.is_cycle() }

	/// Toggle each row index yielded by `entries` using [`Self::add_entry`].
	///
	/// Repeated indices cancel modulo two. The degree is unchanged.
	fn add_entries<B: Iterator<Item = usize>>(&mut self, entries: B) {
		for entry in entries {
			self.add_entry(entry);
		}
	}

	/// Construct an empty column with the supplied chain degree.
	fn new_with_degree(degree: usize) -> Self { Self::from((degree, Self::EntriesRepr::default())) }

	/// Remove every nonzero entry, preserving the chain degree.
	fn clear_entries(&mut self) { self.set_entries(Self::EntriesRepr::default()) }
}
