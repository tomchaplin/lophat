use crate::Column;
use crate::LoPhatOptions;
use crate::RVDecomposition;

use crossbeam::atomic::AtomicCell;
use pinboard::NonEmptyPinboard;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

// Implements do while loop lines 6-9
fn get_col_with_pivot<C: Column>(
    l: usize,
    matrix: &Vec<NonEmptyPinboard<(C, Option<C>)>>,
    pivots: &Vec<AtomicCell<Option<usize>>>,
) -> Option<(usize, (C, Option<C>))> {
    loop {
        let piv = pivots[l].load();
        if let Some(piv) = piv {
            let cols = matrix[piv].read();
            if cols.0.pivot() != Some(l) {
                // Got a column but it now has the wrong pivot; loop again.
                continue;
            };
            // Get column with correct pivot, return to caller.
            return Some((piv, cols));
        } else {
            // There is not yet a column with this pivot, inform caller.
            return None;
        }
    }
}

fn reduce_column<C: Column>(
    j: usize,
    matrix: &Vec<NonEmptyPinboard<(C, Option<C>)>>,
    pivots: &Vec<AtomicCell<Option<usize>>>,
) {
    let mut working_j = j;
    'outer: loop {
        let mut curr_column = matrix[working_j].read();
        while let Some(l) = (&curr_column).0.pivot() {
            let piv_with_column_opt = get_col_with_pivot(l, &matrix, &pivots);
            if let Some((piv, piv_column)) = piv_with_column_opt {
                // Lines 17-24
                if piv < working_j {
                    curr_column.0.add_col(&piv_column.0);
                    // Only add V columns if we need to
                    if let Some(curr_v_col) = curr_column.1.as_mut() {
                        curr_v_col.add_col(&piv_column.1.unwrap());
                    }
                } else if piv > working_j {
                    matrix[working_j].set(curr_column);
                    if pivots[l]
                        .compare_exchange(Some(piv), Some(working_j))
                        .is_ok()
                    {
                        working_j = piv;
                    }
                    continue 'outer;
                } else {
                    panic!()
                }
            } else {
                // piv = -1 case
                matrix[working_j].set(curr_column);
                if pivots[l].compare_exchange(None, Some(working_j)).is_ok() {
                    return;
                } else {
                    continue 'outer;
                }
            }
        }
        // Lines 25-27 (curr_column = 0 clause)
        if (&curr_column.0).pivot().is_none() {
            matrix[working_j].set(curr_column);
            return;
        }
    }
}

/// Decomposes the input matrix, using the lockfree, parallel algoirhtm of Morozov and Nigmetov.
///
/// * `matrix` - iterator over columns of the matrix you wish to decompose.
/// * `options` - additional options to control decompositon, see [`LoPhatOptions`].
pub fn rv_decompose_lock_free<C: Column + 'static>(
    matrix: impl Iterator<Item = C>,
    options: LoPhatOptions,
) -> RVDecomposition<C> {
    let matrix: Vec<_> = matrix
        .enumerate()
        .map(|(idx, r_col)| {
            if options.maintain_v {
                let mut v_col = C::default();
                v_col.add_entry(idx);
                NonEmptyPinboard::new((r_col, Some(v_col)))
            } else {
                NonEmptyPinboard::new((r_col, None))
            }
        })
        .collect();
    let column_height = options.column_height.unwrap_or(matrix.len());
    let pivots: Vec<_> = (0..column_height).map(|_| AtomicCell::new(None)).collect();
    // Setup thread pool
    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(options.num_threads)
        .build()
        .expect("Failed to build thread pool");
    // Reduce matrix
    // TODO: Can we advice rayon to split work in chunks?
    thread_pool.install(|| {
        (0..matrix.len())
            .into_par_iter()
            .with_min_len(options.min_chunk_len)
            .for_each(|j| reduce_column(j, &matrix, &pivots));
    });
    // Wrap into RV decomposition
    let (r, v) = if options.maintain_v {
        let (r_sub, v_sub) = matrix
            .into_iter()
            .map(|pinboard| pinboard.read())
            .map(|(r_col, v_col)| (r_col, v_col.unwrap()))
            .unzip();
        (r_sub, Some(v_sub))
    } else {
        (
            matrix
                .into_iter()
                .map(|pinboard| pinboard.read())
                .map(|(r_col, _v_col)| r_col)
                .collect(),
            None,
        )
    };
    RVDecomposition { r, v }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::column::VecColumn;
    use crate::rv_decompose_serial;
    use proptest::collection::hash_set;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn lockfree_agrees_with_serial( matrix in sut_matrix(100) ) {
            let options = LoPhatOptions { maintain_v: false, column_height: None, num_threads: 0, min_chunk_len: 1 };
            let serial_dgm = rv_decompose_serial(matrix.iter().cloned(), options).diagram();
            let parallel_dgm = rv_decompose_lock_free(matrix.into_iter(), options).diagram();
            assert_eq!(serial_dgm, parallel_dgm);
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
            VecColumn::from(col)
        })
    }
}
