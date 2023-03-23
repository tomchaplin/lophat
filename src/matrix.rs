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

pub fn anti_transpose<'a, C: Column, M: IndexableMatrix<C>>(
    matrix: &'a M,
) -> impl Iterator<Item = C> + 'a {
    let matrix_width = matrix.width();
    (0..matrix.height()).map(move |j| {
        // Need to produce column j for antitranspose
        let mut internal = C::default();
        for i in 0..matrix_width {
            if matrix
                .col(matrix_width - i)
                .has_entry(&(matrix.height() - j))
            {
                internal.add_entry(i)
            }
        }
        internal
    })
}
