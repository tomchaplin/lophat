use std::{
	collections::{HashMap, HashSet},
	fmt::{self, Display},
	ops::{Deref, DerefMut},
};

#[cfg(feature = "python-module")]
use pyo3::prelude::*;

use crate::{algorithms::Decomposition, columns::Column};

/// A nonnegative index or infinity.
///
/// [`Self::Infinity`] is greater than every finite value, including
/// `usize::MAX`. In a persistence diagram, finite values identify death columns
/// and infinity identifies features that persist beyond the supplied
/// filtration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtendedUsize {
	/// A finite index.
	Finite(usize),
	/// A value greater than every finite index.
	Infinity,
}

use ExtendedUsize::{Finite, Infinity};

impl Display for ExtendedUsize {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Finite(index) => write!(f, "{index}"),
			Infinity => write!(f, "Inf"),
		}
	}
}

#[cfg(feature = "python-module")]
impl<'py> IntoPyObject<'py> for ExtendedUsize {
	type Error = PyErr;
	type Output = Bound<'py, PyAny>;
	type Target = PyAny;

	const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
		<Option<usize> as IntoPyObject<'py>>::OUTPUT_TYPE;

	fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
		let index = match self {
			Finite(index) => Some(index),
			Infinity => None,
		};
		Ok(index.into_pyobject(py)?)
	}
}

#[cfg(feature = "python-module")]
impl<'py> IntoPyObject<'py> for &ExtendedUsize {
	type Error = PyErr;
	type Output = Bound<'py, PyAny>;
	type Target = PyAny;

	const OUTPUT_TYPE: pyo3::inspect::PyStaticExpr =
		<ExtendedUsize as IntoPyObject<'py>>::OUTPUT_TYPE;

	fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> { (*self).into_pyobject(py) }
}

/// Map each feature's birth index to its death index or infinity.
///
/// Finite deaths use [`ExtendedUsize::Finite`]; essential features use
/// [`ExtendedUsize::Infinity`]. Indices refer to the input basis, not
/// filtration values. Each birth appears once, and death columns are not
/// separate keys. The map has no specified iteration order. Its display is
/// sorted by birth.
///
/// Construct directly from a [`HashMap`], collect `(birth, death)` entries, or
/// extract from a reduced matrix with [`Self::from_decomposition`]. [`Deref`]
/// and [`DerefMut`] expose the map for lookup, iteration, and editing. An empty
/// decomposition produces an empty map. [`Self::map_idxs`] reindexes endpoints;
/// [`Self::anti_transpose`] restores anti-transposed matrix coordinates.
///
/// # Example
///
/// ```
/// use lophat::utils::{
/// 	ExtendedUsize::{Finite, Infinity},
/// 	PersistenceDiagram,
/// };
///
/// let diagram: PersistenceDiagram = [(0, Infinity), (1, Finite(2))].into_iter().collect();
/// assert_eq!(diagram[&1], Finite(2));
/// assert_eq!(diagram.to_string(), "{0: Inf, 1: 2}");
/// ```
#[cfg_attr(feature = "python-module", derive(IntoPyObject, IntoPyObjectRef))]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PersistenceDiagram(
	/// Birth-to-death entries; keys identify features uniquely.
	pub HashMap<usize, ExtendedUsize>,
);

impl Deref for PersistenceDiagram {
	type Target = HashMap<usize, ExtendedUsize>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for PersistenceDiagram {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Display for PersistenceDiagram {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut entries: Vec<_> = self.iter().collect();
		entries.sort_unstable_by_key(|(birth, _)| **birth);
		write!(f, "{{")?;
		for (position, (birth, death)) in entries.into_iter().enumerate() {
			if position > 0 {
				write!(f, ", ")?;
			}
			write!(f, "{birth}: {death}")?;
		}
		write!(f, "}}")
	}
}

impl FromIterator<(usize, ExtendedUsize)> for PersistenceDiagram {
	fn from_iter<T: IntoIterator<Item = (usize, ExtendedUsize)>>(iter: T) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl PersistenceDiagram {
	/// Extract persistence intervals from the pivots of a reduced matrix R.
	///
	/// A nonzero R column at index `death` contributes `pivot ->
	/// Finite(death)`. Input indices that occur in no finite pair
	/// contribute `birth -> Infinity`. V is not needed. A stored zero
	/// column can represent an essential birth; an empty decomposition
	/// produces an empty diagram.
	///
	/// The persistence interpretation requires a valid filtered boundary
	/// matrix. Results use the supplied matrix coordinates; reverse
	/// anti-transposed coordinates separately using
	/// [`Self::anti_transpose`]. Matrix consistency is not validated.
	pub fn from_decomposition<C: Column>(decomposition: &(impl Decomposition<C> + ?Sized)) -> Self {
		let mut diagram = Self::default();
		let mut deaths = HashSet::new();
		for death in 0..decomposition.n_cols() {
			if let Some(birth) = decomposition.get_r_col(death).pivot() {
				diagram.insert(birth, Finite(death));
				deaths.insert(death);
			}
		}
		for birth in 0..decomposition.n_cols() {
			if !deaths.contains(&birth) {
				diagram.entry(birth).or_insert(Infinity);
			}
		}
		diagram
	}

	/// Restore the original indices of an anti-transposed square matrix.
	///
	/// Consumes the diagram. Finite pairs `(b, d)` become
	/// `(matrix_size - 1 - d, matrix_size - 1 - b)`; essential births `b`
	/// become `matrix_size - 1 - b`. Every finite index must be less than
	/// `matrix_size`. An empty diagram accepts size zero.
	///
	/// # Panics
	///
	/// May panic on subtraction overflow if any index is at least
	/// `matrix_size`.
	pub fn anti_transpose(self, matrix_size: usize) -> Self {
		self.0
			.into_iter()
			.map(|(birth, death)| {
				match death {
					Finite(death) => (matrix_size - 1 - death, Finite(matrix_size - 1 - birth)),
					Infinity => (matrix_size - 1 - birth, Infinity),
				}
			})
			.collect()
	}

	/// Map each birth and finite death through `f`, leaving infinity unchanged.
	///
	/// Consumes the diagram. To preserve distinct features, `f` must be
	/// injective on births. Colliding keys overwrite entries in unspecified
	/// order. No filtration-order validation is performed. Use an
	/// inverse-permutation closure to restore original coordinates after a
	/// permuted reduction.
	pub fn map_idxs(mut self, mut f: impl FnMut(usize) -> usize) -> Self {
		self.0 = self
			.drain()
			.map(|(birth, death)| {
				let birth = f(birth);
				let death = match death {
					Finite(death) => Finite(f(death)),
					Infinity => Infinity,
				};
				(birth, death)
			})
			.collect();
		self
	}
}

#[cfg(test)]
mod tests {
	use super::{
		ExtendedUsize::{Finite, Infinity},
		PersistenceDiagram,
	};
	use crate::{
		algorithms::{Decomposition, DecompositionAlgo, SerialAlgorithm},
		columns::VecColumn,
		options::LoPhatOptions,
		utils::anti_transpose,
	};

	#[test]
	fn diagrams_from_boundary_and_anti_transpose_agree() {
		let matrix: Vec<VecColumn> = vec![
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(1, vec![0, 1]),
			(1, vec![1, 2]),
			(1, vec![0, 2]),
			(2, vec![3, 4, 5]),
		]
		.into_iter()
		.map(VecColumn::from)
		.collect();
		let expected: PersistenceDiagram = [
			(0, Infinity),
			(1, Finite(3)),
			(2, Finite(4)),
			(5, Finite(6)),
		]
		.into_iter()
		.collect();
		for maintain_v in [false, true] {
			let options = LoPhatOptions {
				maintain_v,
				..Default::default()
			};
			let decomposition = SerialAlgorithm::init(Some(options))
				.add_cols(matrix.clone().into_iter())
				.decompose();
			assert_eq!(
				PersistenceDiagram::from_decomposition(&decomposition),
				expected
			);
			assert_eq!(decomposition.diagram(), expected);
			let transposed = SerialAlgorithm::init(Some(options))
				.add_cols(anti_transpose(&matrix).into_iter())
				.decompose();
			assert_eq!(transposed.diagram().anti_transpose(matrix.len()), expected);
		}
	}

	#[test]
	fn empty_and_zero_column_decompositions() {
		for maintain_v in [false, true] {
			let options = LoPhatOptions {
				maintain_v,
				..Default::default()
			};
			let empty = SerialAlgorithm::<VecColumn>::init(Some(options)).decompose();
			assert_eq!(
				empty.diagram().anti_transpose(0),
				PersistenceDiagram::default()
			);
			let zero = SerialAlgorithm::init(Some(options))
				.add_cols(std::iter::once(VecColumn::from((0, vec![]))))
				.decompose();
			assert_eq!(zero.diagram(), [(0, Infinity)].into_iter().collect());
		}
	}

	#[test]
	fn infinity_orders_after_all_finite_indices() {
		assert!(Finite(0) < Finite(1));
		assert!(Finite(usize::MAX) < Infinity);
		assert_eq!(Infinity.cmp(&Infinity), std::cmp::Ordering::Equal);
		assert_eq!(Finite(usize::MAX).to_string(), usize::MAX.to_string());
		assert_eq!(Infinity.to_string(), "Inf");
	}

	#[test]
	fn mapping_and_display_preserve_finite_and_essential_intervals() {
		let original: PersistenceDiagram = [(0, Finite(3)), (1, Infinity)].into_iter().collect();
		let mapped = original.clone().map_idxs(|idx| idx + 5);
		assert_eq!(
			mapped,
			[(5, Finite(8)), (6, Infinity)].into_iter().collect()
		);
		assert_eq!(mapped.to_string(), "{5: 8, 6: Inf}");
		assert_eq!(original[&0], Finite(3));
	}
}
