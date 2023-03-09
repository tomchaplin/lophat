use crate::Column;
use rayon::prelude::IntoParallelIterator;

pub trait Matrix<C: Column>: IntoIterator<Item = C> + IntoParallelIterator<Item = C> {
    fn get_col(&self, index: usize) -> C;
    fn set_col(&mut self, index: usize, col: C);
    fn push_col(&mut self, col: C);
    fn len(&self) -> usize;
}

impl<C> Matrix<C> for Vec<C>
where
    C: Column,
{
    fn get_col(&self, index: usize) -> C {
        self[index].clone()
    }

    fn set_col(&mut self, index: usize, col: C) {
        self[index] = col;
    }

    fn push_col(&mut self, col: C) {
        self.push(col);
    }

    fn len(&self) -> usize {
        self.len()
    }
}
