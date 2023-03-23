use crate::Column;

/// Unused.
pub trait IndexableMatrix<C: Column> {
    fn col(&self, index: usize) -> &C;
    fn set_col(&mut self, index: usize, col: C);
    fn push_col(&mut self, col: C);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub struct VecMatrix<C> {
    pub cols: Vec<C>,
    height: usize,
}

impl<C: Column> IndexableMatrix<C> for VecMatrix<C> {
    fn col(&self, index: usize) -> &C {
        &self.cols[index]
    }

    fn set_col(&mut self, index: usize, col: C) {
        self.cols[index] = col;
    }

    fn push_col(&mut self, col: C) {
        self.cols.push(col);
    }

    fn width(&self) -> usize {
        self.cols.len()
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<C: Column> From<Vec<C>> for VecMatrix<C> {
    fn from(cols: Vec<C>) -> Self {
        Self {
            height: cols.len(),
            cols,
        }
    }
}

impl<C: Column> From<(Vec<C>, Option<usize>)> for VecMatrix<C> {
    fn from((cols, height): (Vec<C>, Option<usize>)) -> Self {
        match height {
            Some(height) => Self { height, cols },
            None => Self {
                height: cols.len(),
                cols,
            },
        }
    }
}
