use crate::columns::Column;

/// Anti-transpose a square matrix, reversing row and column order.
///
/// For an `n`-column input, entry `(i, j)` moves to `(n - 1 - j, n - 1 - i)`.
/// The output column corresponding to an input column of degree `d` is assigned
/// `max_degree - d`, where `max_degree` is the greatest input degree. The input
/// is borrowed unchanged and the output contains newly constructed columns.
/// An empty matrix produces an empty vector.
///
/// This transformation can be used to compute pairings via cohomology. Map the
/// resulting diagram back with
/// [`PersistenceDiagram::anti_transpose`](crate::utils::PersistenceDiagram::anti_transpose).
///
///
/// # Panics
///
/// Every nonzero row index must be less than `matrix.len()`. Invalid indices
/// may cause subtraction overflow or an out-of-bounds access.
pub fn anti_transpose<C: Column>(matrix: &[C]) -> Vec<C> {
	let matrix_width = matrix.len();
	let max_dim = matrix.iter().map(|col| col.degree()).max().unwrap_or(0);
	let mut return_matrix: Vec<_> = matrix
		.iter()
		.rev()
		.map(|col| C::new_with_degree(max_dim - col.degree()))
		.collect();
	for (j, col) in matrix.iter().enumerate() {
		for i in col.entries() {
			return_matrix[matrix_width - 1 - i].add_entry(matrix_width - 1 - j);
		}
	}
	return_matrix
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::columns::VecColumn;

	fn build_sphere_triangulation() -> Vec<VecColumn> {
		vec![
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(1, vec![0, 1]),
			(1, vec![0, 2]),
			(1, vec![1, 2]),
			(1, vec![0, 3]),
			(1, vec![1, 3]),
			(1, vec![2, 3]),
			(2, vec![4, 7, 8]),
			(2, vec![5, 7, 9]),
			(2, vec![6, 8, 9]),
			(2, vec![4, 5, 6]),
		]
		.into_iter()
		.map(|col| col.into())
		.collect()
	}

	fn build_sphere_triangulation_at() -> Vec<VecColumn> {
		vec![
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(0, vec![]),
			(1, vec![1, 2]),
			(1, vec![1, 3]),
			(1, vec![2, 3]),
			(1, vec![0, 1]),
			(1, vec![0, 2]),
			(1, vec![0, 3]),
			(2, vec![4, 5, 6]),
			(2, vec![4, 7, 8]),
			(2, vec![5, 7, 9]),
			(2, vec![6, 8, 9]),
		]
		.into_iter()
		.map(|col| col.into())
		.collect()
	}

	#[test]
	fn sphere_triangulation_at() {
		let matrix = build_sphere_triangulation();
		let matrix_at = build_sphere_triangulation_at();
		let at: Vec<VecColumn> = anti_transpose(&matrix);
		assert_eq!(at, matrix_at);
	}
	use proptest::{collection::hash_set, prelude::*};

	proptest! {
		#[test]
		fn at_at_is_identity( matrix in sut_matrix(100) ) {
			let at: Vec<VecColumn> = anti_transpose(&matrix);
			let at_at: Vec<VecColumn> = anti_transpose(&at);
			assert_eq!(matrix, at_at);
		}
	}

	// Generates a strict upper triangular matrix of VecColumns with given size
	fn sut_matrix(size: usize) -> impl Strategy<Value = Vec<VecColumn>> {
		let mut matrix = vec![];
		for i in 1..size {
			matrix.push(veccolum_with_idxs_below(i));
		}
		matrix
	}

	fn veccolum_with_idxs_below(mut max_idx: usize) -> impl Strategy<Value = VecColumn> {
		// Avoid empty range problem
		// Always returns empty Vec because size is in 0..1 == { 0 }
		if max_idx == 0 {
			max_idx = 1;
		}
		hash_set(0..max_idx, 0..max_idx).prop_map(|set| {
			let mut col: Vec<_> = set.into_iter().collect();
			col.sort();
			VecColumn::from((0, col))
		})
	}
}
