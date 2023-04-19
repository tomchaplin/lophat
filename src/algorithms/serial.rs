use crate::{
    algorithms::RVDecomposition,
    columns::{Column, ColumnMode},
    options::LoPhatOptions,
};

use std::collections::HashMap;

/// Implements the standard left-to-right column additional algorithm of [Edelsbrunner et al.](https://doi.org/10.1109/SFCS.2000.892133).
/// No optimisations have been implemented.
#[derive(Debug)]
pub struct SerialAlgorithm<C: Column> {
    r: Vec<C>,
    v: Option<Vec<C>>,
    low_inverse: HashMap<usize, usize>,
}

impl<C: Column> SerialAlgorithm<C> {
    fn new(options: LoPhatOptions) -> Self {
        Self {
            r: vec![],
            v: options.maintain_v.then_some(vec![]),
            low_inverse: HashMap::new(),
        }
    }

    fn col_idx_with_same_low(&self, col: &C) -> Option<usize> {
        let pivot = col.pivot()?;
        self.low_inverse.get(&pivot).copied()
    }
    /// Uses the decomposition so far to reduce the next column of D with left-to-right columns addition.
    pub fn reduce_column(&mut self, mut column: C) {
        column.set_mode(ColumnMode::Working);
        // v_col tracks how the final reduced column is built up
        // Currently column contains 1 lot of the latest column in D
        let maintain_v = self.v.is_some();
        let mut v_col: Option<C> = None;
        if maintain_v {
            let mut v_col_internal = C::new_with_dimension(column.dimension());
            v_col_internal.set_mode(ColumnMode::Working);
            v_col_internal.add_entry(self.r.len());
            v_col = Some(v_col_internal);
        }
        // Reduce the column, keeping track of how we do this in V
        while let Some(col_idx) = self.col_idx_with_same_low(&column) {
            column.add_col(&self.r[col_idx]);
            if maintain_v {
                v_col
                    .as_mut()
                    .unwrap()
                    .add_col(&self.v.as_mut().unwrap()[col_idx]);
            }
        }
        // Update low inverse
        let final_pivot = column.pivot();
        if let Some(final_pivot) = final_pivot {
            // This column has a lowest 1 and is being inserted at the end of R
            self.low_inverse.insert(final_pivot, self.r.len());
        }
        // Push to decomposition
        column.set_mode(ColumnMode::Storage);
        self.r.push(column);
        if maintain_v {
            let mut v_col = v_col.unwrap();
            v_col.set_mode(ColumnMode::Storage);
            self.v.as_mut().unwrap().push(v_col);
        }
    }
}

impl<C: Column> RVDecomposition<C> for SerialAlgorithm<C> {
    fn get_r_col(&self, index: usize) -> &C {
        &self.r[index]
    }

    fn get_v_col(&self, index: usize) -> Option<&C> {
        Some(&self.v.as_ref()?[index])
    }

    fn n_cols(&self) -> usize {
        self.r.len()
    }

    type RColRef<'a> = &'a C where Self : 'a;

    type VColRef<'a> = &'a C where Self: 'a;

    type Options = LoPhatOptions;

    fn decompose(matrix: impl Iterator<Item = C>, options: Option<Self::Options>) -> Self {
        let options = options.unwrap_or_default();
        let algo = SerialAlgorithm::new(options);
        matrix.fold(algo, |mut accum, next_col| {
            accum.reduce_column(next_col);
            accum
        })
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashSet;

    use crate::{columns::VecColumn, utils::PersistenceDiagram};

    use super::*;

    fn build_sphere_triangulation() -> impl Iterator<Item = VecColumn> {
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
    }

    #[test]
    fn sphere_triangulation_correct() {
        let matrix = build_sphere_triangulation();
        let correct_diagram = PersistenceDiagram {
            unpaired: HashSet::from_iter(vec![0, 13]),
            paired: HashSet::from_iter(vec![(1, 4), (2, 5), (3, 7), (6, 12), (8, 10), (9, 11)]),
        };
        let options = LoPhatOptions::default();
        let computed_diagram = SerialAlgorithm::decompose(matrix, Some(options)).diagram();
        assert_eq!(computed_diagram, correct_diagram)
    }

    #[test]
    fn test_v_maintain() {
        let matrix = build_sphere_triangulation();
        let mut options = LoPhatOptions::default();
        options.maintain_v = true;
        let correct_diagram = PersistenceDiagram {
            unpaired: HashSet::from_iter(vec![0, 13]),
            paired: HashSet::from_iter(vec![(1, 4), (2, 5), (3, 7), (6, 12), (8, 10), (9, 11)]),
        };
        let decomp = SerialAlgorithm::decompose(matrix, Some(options));
        let computed_diagram = decomp.diagram();
        for col in decomp.v.unwrap() {
            println!("{:?}", col);
        }
        assert_eq!(computed_diagram, correct_diagram)
    }
}
