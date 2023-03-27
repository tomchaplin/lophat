use crate::{Column, PersistenceDiagram};

/// Re-indexes a persistence diagram, assuming that it was produced from an anti-transposed matrix.
/// * `diagram` - the diagram to reindex.
/// * `matrix_size` - the size of the decomposed matrix, assumed to be square.
pub fn anti_transpose_diagram(
    mut diagram: PersistenceDiagram,
    matrix_size: usize,
) -> PersistenceDiagram {
    let new_paired = diagram
        .paired
        .into_iter()
        .map(|(b, d)| (matrix_size - 1 - d, matrix_size - 1 - b))
        .collect();
    let new_unpaired = diagram
        .unpaired
        .into_iter()
        .map(|idx| matrix_size - 1 - idx)
        .collect();
    diagram.paired = new_paired;
    diagram.unpaired = new_unpaired;
    diagram
}

/// Anti-transposes the input matrix (e.g. to compute cohomology).
/// * `matrix` - a reference to a collected matrix (vector of columns).
/// Assumes that input matrix is square.
pub fn anti_transpose<C: Column>(matrix: &Vec<C>) -> Vec<C> {
    let matrix_width = matrix.len();
    let max_dim = matrix.iter().map(|col| col.dimension()).max().unwrap();
    let mut return_matrix: Vec<_> = matrix
        .iter()
        .rev()
        .map(|col| C::new_with_dimension(max_dim - col.dimension()))
        .collect();
    for (j, col) in matrix.iter().enumerate() {
        for i in col.boundary().iter() {
            return_matrix[matrix_width - 1 - i].add_entry(matrix_width - 1 - j);
        }
    }
    return_matrix
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::VecColumn;

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
    use proptest::collection::hash_set;
    use proptest::prelude::*;

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
