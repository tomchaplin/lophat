use std::ops::Deref;

use hashbrown::HashSet;

use crate::{columns::Column, utils::PersistenceDiagram};

pub trait RVDecomposition<C>
where
    C: Column + Sync,
{
    type Options;

    fn decompose(matrix: impl Iterator<Item = C>, options: Self::Options) -> Self;

    type RColRef<'a>: Deref<Target = C> + 'a
    where
        Self: 'a;
    fn get_r_col<'a>(&'a self, index: usize) -> Self::RColRef<'a>;

    type VColRef<'a>: Deref<Target = C> + 'a
    where
        Self: 'a;
    fn get_v_col<'a>(&'a self, index: usize) -> Option<Self::VColRef<'a>>;

    fn n_cols(&self) -> usize;

    fn diagram(&self) -> PersistenceDiagram {
        let r_col_iter = (0..self.n_cols()).map(|idx| self.get_r_col(idx));
        let paired: HashSet<(usize, usize)> = r_col_iter
            .enumerate()
            .filter_map(|(idx, col)| {
                let lowest_idx = col.pivot()?;
                Some((lowest_idx, idx))
            })
            .collect();
        let mut unpaired: HashSet<usize> = (0..self.n_cols()).collect();
        for (birth, death) in paired.iter() {
            unpaired.remove(birth);
            unpaired.remove(death);
        }
        PersistenceDiagram { unpaired, paired }
    }
}
