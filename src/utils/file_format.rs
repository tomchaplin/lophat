use std::{cell::Cell, ops::Deref};

use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::{
	algorithms::{Decomposition, NoVMatrixError},
	columns::{Column, VecColumn},
};

#[macro_export]
/// Implement [`Serialize`](serde::Serialize) for a generic decomposition type.
///
/// The struct must be generic over the column type and implement
/// [`Decomposition`](crate::algorithms::Decomposition). As a fallback, you may
/// wish to use [`serialize_algo`] to implement [`Serialize`](serde::Serialize)
/// yourself.
///
/// The expansion uses the caller's `Column` trait import and introduces
/// `Serialize` and `serialize_algo` imports in the invocation's scope. Invoke
/// once per scope, with a type generic over a single column parameter.
/// Deserialization is provided through [`DecompositionFileFormat`] rather
/// than through the original algorithm-specific type.
///
/// # Example usage
///
/// ```ignore
/// use lophat::impl_rvd_serialize;
/// use lophat::columns::Column;
/// use lophat::algorithms::Decomposition;
///
/// struct MyAlgo<C: Column> { ... }
///
/// impl<C:Column> Decomposition<C> for MyAlgo<C> { ... }
///
/// impl_rvd_serialize!(MyAlgo);
/// ```
macro_rules! impl_rvd_serialize {
	($struct:ident) => {
		use serde::Serialize;
		use $crate::utils::serialize_algo;
		impl<C: Column> Serialize for $struct<C> {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				serialize_algo(self, serializer)
			}
		}
	};
}

/// Owned R and optional V matrices in the common decomposition file format.
///
/// Stores columns as [`VecColumn`] values and implements both Serde
/// serialization and deserialization. Typically obtained by serializing an
/// algorithm's [`Decomposition`] and reading it into this type, or by calling
/// [`clone_to_file_format`]. The [`Decomposition`] methods provide column
/// access and diagram extraction after loading.
///
/// The canonical empty representation has no R columns and `v: None`.
/// Serialization normalizes every empty decomposition to this representation,
/// so its original `maintain_v` setting cannot be recovered. Construction and
/// deserialization do not validate matrix consistency or reducedness.
///
/// # Example
/// ```
/// // Use CBOR file format to store serialization
/// use ciborium::{de::from_reader, ser::into_writer};
/// use lophat::{
/// 	algorithms::{DecompositionAlgo, LockFreeAlgorithm},
/// 	columns::VecColumn,
/// 	utils::DecompositionFileFormat,
/// };
///
/// let matrix = vec![
/// 	(0, vec![]),
/// 	(0, vec![]),
/// 	(0, vec![]),
/// 	(1, vec![0, 1]),
/// 	(1, vec![0, 2]),
/// 	(1, vec![1, 2]),
/// 	(2, vec![3, 4, 5]),
/// ]
/// .into_iter()
/// .map(VecColumn::from);
///
/// let decomp = LockFreeAlgorithm::init(None).add_cols(matrix).decompose();
/// // Serialize into bytes (could write to file here instead)
/// let mut bytes: Vec<u8> = vec![];
/// into_writer(&decomp, &mut bytes).unwrap();
/// // Deserialize to file format
/// let rvdff: DecompositionFileFormat = from_reader(bytes.as_slice()).unwrap();
/// ```
#[derive(Deserialize, PartialEq, Debug)]
pub struct DecompositionFileFormat {
	r: Vec<VecColumn>,
	v: Option<Vec<VecColumn>>,
}

impl DecompositionFileFormat {
	/// Construct from owned R columns and optional owned V columns.
	///
	/// No consistency checks are performed. If V is supplied, it must contain
	/// one column per R column for V access and serialization to work
	/// correctly. The caller is responsible for reducedness and native
	/// column invariants.
	///
	/// `Self::new(Vec::new(), None)` is the canonical empty representation. An
	/// empty R matrix cannot reveal whether V was maintained; serialization
	/// normalizes such a value to an empty R matrix and absent V.
	pub fn new(r: Vec<VecColumn>, v: Option<Vec<VecColumn>>) -> Self { Self { r, v } }
}

impl Decomposition<VecColumn> for DecompositionFileFormat {
	type RColRef<'a>
		= &'a VecColumn
	where
		Self: 'a;
	type VColRef<'a>
		= &'a VecColumn
	where
		Self: 'a;

	fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a> { &self.r[index] }

	fn get_v_col<'a>(&'a self, index: usize) -> Result<Self::VColRef<'a>, NoVMatrixError> {
		if self.n_cols() == 0 {
			return Err(NoVMatrixError::EmptyDecompositionError);
		}
		Ok(&self
			.v
			.as_ref()
			.ok_or(NoVMatrixError::VMatrixDiscardedError)?[index])
	}

	fn n_cols(&self) -> usize { self.r.len() }
}

/// Copy a column's degree and nonzero coefficients into sorted vector storage.
///
/// Populates a new [`VecColumn`] by toggling the indices yielded by
/// [`Column::entries`]. Does not modify the input and does not require its
/// iterator to be sorted.
pub fn clone_to_veccolumn<C: Column>(col: &C) -> VecColumn {
	let mut output = VecColumn::new_with_degree(col.degree());
	output.add_entries(col.entries());
	output
}

/// Serialize any decomposition in the common R/V file format.
///
/// Serializes R and retained V as sequences of [`VecColumn`] values. Empty
/// inputs and decompositions without V are written with `v: None`. The original
/// `maintain_v` setting of an empty input is not preserved. Deserialize the
/// result into [`DecompositionFileFormat`]. Columns are copied individually
/// while serializing rather than cloning the complete decomposition first.
///
/// # Errors
///
/// Propagates errors reported by the supplied serializer.
///
/// # Panics
///
/// May panic if a custom [`Decomposition`] reports V as available but fails to
/// provide a V column at a valid input index.
pub fn serialize_algo<C, Algo, S>(algo: &Algo, serializer: S) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
	C: Column,
	Algo: Decomposition<C>,
{
	// Taken from https://users.rust-lang.org/t/how-to-serialize-an-iterator-to-json/59272
	// We wrap the iterator in a cell so that we can implement Serialize on it
	// We also wrap it in an option because in order to call Cell::take() and
	// accept ownership of the iterator we must leave behind a default
	// value.
	struct IteratorWrapper<T>(Cell<Option<T>>);

	impl<T> IteratorWrapper<T> {
		fn new(value: T) -> Self { IteratorWrapper(Cell::new(Some(value))) }
	}
	impl<I, J> Serialize for IteratorWrapper<I>
	where
		I: IntoIterator<Item = J>,
		J: Serialize,
	{
		fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: serde::Serializer,
		{
			serializer.collect_seq(self.0.take().unwrap())
		}
	}

	// Set up struct
	let mut rvdff = serializer.serialize_struct("DecompositionFileFormat", 2)?;

	// Serialize R
	let r_col_iter = (0..algo.n_cols()).map(|idx| {
		let col = algo.get_r_col(idx);
		clone_to_veccolumn(col.deref())
	});
	let r_col_iter = IteratorWrapper::new(r_col_iter);
	rvdff.serialize_field("r", &r_col_iter)?;

	// Serialize V
	let v_col_iter_opt = match algo.has_v() {
		Ok(()) => {
			let v_col_iter = (0..algo.n_cols()).map(|idx| {
				// Can safely unwrap everything because V was maintained
				let col = algo.get_v_col(idx).unwrap();
				clone_to_veccolumn(col.deref())
			});
			Some(IteratorWrapper::new(v_col_iter))
		},
		Err(NoVMatrixError::EmptyDecompositionError | NoVMatrixError::VMatrixDiscardedError) => {
			None
		},
	};
	rvdff.serialize_field("v", &v_col_iter_opt)?;
	rvdff.end()
}

// We do not derive directly because we want all algorithms to use the same
// serialize function.
impl Serialize for DecompositionFileFormat {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serialize_algo(self, serializer)
	}
}

/// Copy a decomposition into the standard R/V file format in memory.
///
/// Copies every R column and, when available, every V column into [`VecColumn`]
/// storage. An empty decomposition produces an empty R matrix and `v: None`,
/// regardless of its original `maintain_v` setting. Use [`serialize_algo`] or
/// [`impl_rvd_serialize`](crate::impl_rvd_serialize) to serialize without first
/// allocating a complete second decomposition.
///
/// # Panics
///
/// May panic if a custom [`Decomposition`] reports V as available but fails to
/// provide a V column at a valid input index.
pub fn clone_to_file_format<C: Column, Algo: Decomposition<C>>(
	algo: &Algo,
) -> DecompositionFileFormat {
	let r = (0..algo.n_cols())
		.map(|idx| {
			let col = algo.get_r_col(idx);
			clone_to_veccolumn(col.deref())
		})
		.collect();
	let v = match algo.has_v() {
		Ok(()) => {
			Some(
				(0..algo.n_cols())
					.map(|idx| {
						let col = algo.get_v_col(idx).unwrap();
						clone_to_veccolumn(col.deref())
					})
					.collect(),
			)
		},
		Err(NoVMatrixError::EmptyDecompositionError | NoVMatrixError::VMatrixDiscardedError) => {
			None
		},
	};
	DecompositionFileFormat::new(r, v)
}

#[cfg(test)]
mod tests {
	use ciborium::{de::from_reader, ser::into_writer};
	use serde::Serialize;

	use super::{DecompositionFileFormat, clone_to_file_format};
	use crate::{
		algorithms::{
			Decomposition,
			DecompositionAlgo,
			LockFreeAlgorithm,
			LockingAlgorithm,
			NoVMatrixError,
			SerialAlgorithm,
		},
		columns::VecColumn,
		options::LoPhatOptions,
	};

	fn check_serialization_and_clone<D: Decomposition<VecColumn> + Serialize>(
		decomp: &D,
		expected: &DecompositionFileFormat,
	) {
		assert_eq!(&clone_to_file_format(decomp), expected);
		let mut bytes = Vec::new();
		into_writer(decomp, &mut bytes).unwrap();
		let restored: DecompositionFileFormat = from_reader(bytes.as_slice()).unwrap();
		assert_eq!(&restored, expected);
		assert_eq!(restored.has_v(), expected.has_v());
	}

	fn check_algorithm_serialization<A: DecompositionAlgo<VecColumn, Options = LoPhatOptions>>()
	where
		A::Decomposition: Serialize,
	{
		for maintain_v in [false, true] {
			let options = LoPhatOptions {
				maintain_v,
				clearing: false,
				num_threads: 1,
				..Default::default()
			};
			let empty = A::init(Some(options)).decompose();
			check_serialization_and_clone(&empty, &DecompositionFileFormat::new(vec![], None));

			let zero_column = A::init(Some(options))
				.add_cols(std::iter::once(VecColumn::from((0, vec![]))))
				.decompose();
			let expected = DecompositionFileFormat::new(
				vec![VecColumn::from((0, vec![]))],
				maintain_v.then(|| vec![VecColumn::from((0, vec![0]))]),
			);
			check_serialization_and_clone(&zero_column, &expected);

			let simplex = A::init(Some(options)).add_cols(get_matrix()).decompose();
			check_serialization_and_clone(&simplex, &get_rvdff(maintain_v));
		}
	}

	#[test]
	fn serialize_and_clone_serial_edge_cases() {
		check_algorithm_serialization::<SerialAlgorithm<VecColumn>>();
	}

	#[test]
	fn serialize_and_clone_lockfree_edge_cases() {
		check_algorithm_serialization::<LockFreeAlgorithm<VecColumn>>();
	}

	#[test]
	fn serialize_and_clone_locking_edge_cases() {
		check_algorithm_serialization::<LockingAlgorithm<VecColumn>>();
	}

	#[test]
	fn empty_fileformat_is_canonicalized() {
		for v in [None, Some(vec![])] {
			let decomp = DecompositionFileFormat::new(vec![], v);
			assert_eq!(decomp.has_v(), Err(NoVMatrixError::EmptyDecompositionError));
			assert_eq!(
				decomp.get_v_col(0).err(),
				Some(NoVMatrixError::EmptyDecompositionError)
			);
			check_serialization_and_clone(&decomp, &DecompositionFileFormat::new(vec![], None));
		}
	}

	#[test]
	fn nonempty_fileformat_reports_v_presence() {
		for maintain_v in [false, true] {
			let decomp = DecompositionFileFormat::new(
				vec![VecColumn::from((0, vec![]))],
				maintain_v.then(|| vec![VecColumn::from((0, vec![0]))]),
			);
			let expected = if maintain_v {
				Ok(())
			} else {
				Err(NoVMatrixError::VMatrixDiscardedError)
			};
			assert_eq!(decomp.has_v(), expected);
			assert_eq!(decomp.get_v_col(0).map(|_| ()), expected);
			check_serialization_and_clone(&decomp, &decomp);
		}
	}

	fn get_matrix() -> impl Iterator<Item = VecColumn> {
		vec![
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(1, vec![0, 1]),
			(1, vec![0, 2]),
			(1, vec![1, 2]),
			(2, vec![3, 4, 5]),
		]
		.into_iter()
		.map(VecColumn::from)
	}

	fn get_rvdff(with_g: bool) -> DecompositionFileFormat {
		let correct_r = vec![
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(1, vec![0, 1]),
			(1, vec![0, 2]),
			(1, vec![]),
			(2, vec![3, 4, 5]),
		]
		.into_iter()
		.map(VecColumn::from);
		let correct_v = vec![
			(0, vec![0]),
			(0, vec![1]),
			(0, vec![2]),
			(1, vec![3]),
			(1, vec![4]),
			(1, vec![3, 4, 5]),
			(2, vec![6]),
		]
		.into_iter()
		.map(VecColumn::from);
		if with_g {
			DecompositionFileFormat::new(correct_r.collect(), Some(correct_v.collect()))
		} else {
			DecompositionFileFormat::new(correct_r.collect(), None)
		}
	}

	#[test]
	fn serialize_fileformat_and_back() {
		// Serialize and back again with V
		let rvdff_1 = get_rvdff(true);
		let mut bytes: Vec<u8> = vec![];
		into_writer(&rvdff_1, &mut bytes).ok();
		let rvdff_2: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
		assert_eq!(rvdff_1, rvdff_2);
		// Serialize and back again without V
		let rvdff_1 = get_rvdff(false);
		let mut bytes: Vec<u8> = vec![];
		into_writer(&rvdff_1, &mut bytes).ok();
		let rvdff_2: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
		assert_eq!(rvdff_1, rvdff_2);
	}

	#[test]
	fn serialize_lfa_and_back() {
		let matrix = get_matrix();
		let correct_rvdff = get_rvdff(true);
		// Decompose via LFA
		let options = LoPhatOptions {
			maintain_v: true,
			clearing: false, // Just do normal left-to-right reduction in decreasing degrees
			num_threads: 1,  // So we can predict the output
			..Default::default()
		};
		let decomp = LockFreeAlgorithm::init(Some(options))
			.add_cols(matrix)
			.decompose();
		// Serialize into bytes
		let mut bytes: Vec<u8> = vec![];
		into_writer(&decomp, &mut bytes).ok();
		// Deserialize to file format
		let rvdff: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
		// Check all columns are correct
		assert_eq!(rvdff, correct_rvdff)
	}

	#[test]
	fn serialize_lfa_without_v() {
		let matrix = get_matrix();
		let correct_rvdff = get_rvdff(false); // Decompose via LFA
		let options = LoPhatOptions {
			maintain_v: false, // Just do normal left-to-right reduction in decreasing degrees
			clearing: false,
			num_threads: 1, // So we can predict the output
			..Default::default()
		};
		let decomp = LockFreeAlgorithm::init(Some(options))
			.add_cols(matrix)
			.decompose();
		// Serialize into bytes
		let mut bytes: Vec<u8> = vec![];
		into_writer(&decomp, &mut bytes).ok();
		// Deserialize to file format
		let rvdff: DecompositionFileFormat = from_reader(bytes.as_slice()).ok().unwrap();
		// Check all columns are correct and V is none
		assert_eq!(rvdff, correct_rvdff)
	}
}
