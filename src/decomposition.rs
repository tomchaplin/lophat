use crate::Column;
use hashbrown::HashSet;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

#[pyclass]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct PersistenceDiagram {
    #[pyo3(get)]
    pub unpaired: HashSet<usize>,
    #[pyo3(get)]
    pub paired: HashSet<(usize, usize)>,
}

#[derive(Debug, Default)]
pub struct RVDecomposition<C: Column> {
    pub r: Vec<C>,
    pub v: Vec<C>,
}

fn col_idx_with_same_low<C: Column>(col: &C, low_inverse: &HashMap<usize, usize>) -> Option<usize> {
    let pivot = col.pivot()?;
    low_inverse.get(&pivot).copied()
}

impl<C: Column> RVDecomposition<C> {
    // Receives column, reduces it with left-to-right addition from R
    // Adds reduction to self
    pub fn reduce_column(&mut self, mut column: C, low_inverse: &mut HashMap<usize, usize>) {
        // v_col tracks how the final reduced column is built up
        // Currently column contains 1 lot of the latest column in D
        let mut v_col = C::default();
        v_col.add_entry(self.r.len());
        // Reduce the column, keeping track of how we do this in V
        while let Some(col_idx) = col_idx_with_same_low(&column, &low_inverse) {
            column.add_col(&self.r[col_idx]);
            v_col.add_col(&self.v[col_idx]);
        }
        // Update low inverse
        let final_pivot = column.pivot();
        if let Some(final_pivot) = final_pivot {
            // This column has a lowest 1 and is being inserted at the end of R
            low_inverse.insert(final_pivot, self.r.len());
        }
        // Push to decomposition
        self.r.push(column);
        self.v.push(v_col);
    }

    pub fn diagram(&self) -> PersistenceDiagram {
        let paired: HashSet<(usize, usize)> = self
            .r
            .par_iter()
            .enumerate()
            .filter_map(|(idx, col)| {
                let lowest_idx = col.pivot()?;
                Some((lowest_idx, idx))
            })
            .collect();
        let mut unpaired: HashSet<usize> = (0..self.r.len()).collect();
        for (birth, death) in paired.iter() {
            unpaired.remove(birth);
            unpaired.remove(death);
        }
        PersistenceDiagram { unpaired, paired }
    }
}

pub fn rv_decompose<C: Column>(matrix: impl Iterator<Item = C>) -> RVDecomposition<C> {
    let mut low_inverse = HashMap::new();
    matrix.fold(RVDecomposition::default(), |mut accum, next_col| {
        accum.reduce_column(next_col, &mut low_inverse);
        accum
    })
}

#[cfg(test)]
mod tests {
    use crate::column::VecColumn;

    use super::*;

    fn build_sphere_triangulation() -> impl Iterator<Item = VecColumn> {
        vec![
            vec![],
            vec![],
            vec![],
            vec![],
            vec![0, 1],
            vec![0, 2],
            vec![1, 2],
            vec![0, 3],
            vec![1, 3],
            vec![2, 3],
            vec![4, 7, 8],
            vec![5, 7, 9],
            vec![6, 8, 9],
            vec![4, 5, 6],
        ]
        .into_iter()
        .map(|internal| VecColumn { internal })
    }

    #[test]
    fn sphere_triangulation_correct() {
        let matrix = build_sphere_triangulation();
        let correct_diagram = PersistenceDiagram {
            unpaired: HashSet::from_iter(vec![0, 13]),
            paired: HashSet::from_iter(vec![(1, 4), (2, 5), (3, 7), (6, 12), (8, 10), (9, 11)]),
        };
        let computed_diagram = rv_decompose(matrix).diagram();
        assert_eq!(computed_diagram, correct_diagram)
    }
}
