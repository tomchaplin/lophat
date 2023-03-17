use pyo3::prelude::*;

/// A simple struct for specifying options for R=DV decompositions
///
/// * `maintain_v` - if true, returns full R=DV decomposition,
///   otherwise returns [`RVDecomposition`](crate::RVDecomposition) with field `v` set to `None`.
/// * `n_threads` - number of threads to use in thread pool; ignored by serial algorithms.
///   see [`num_threads`](rayon::ThreadPoolBuilder::num_threads) for more details.
///   Only relevant for lockfree algorithm.
/// * `column_height` - an optional hint to the height of the columns.
///   If `None`, assumed to be `matrix.collect().len()`.
///   All indices must lie in the range `0..column_height`.
///   Only relevant for lockfree algorithm.
/// * `min_chunk_len` - When splitting work, ensure that each thread gets at least this many columns to work on at a time.
///   Only relevant for lockfree algorithm.
#[pyclass]
#[derive(Default, Clone, Copy)]
pub struct LoPhatOptions {
    #[pyo3(get, set)]
    pub maintain_v: bool,
    #[pyo3(get, set)]
    pub num_threads: usize,
    #[pyo3(get, set)]
    pub column_height: Option<usize>,
    #[pyo3(get, set)]
    pub min_chunk_len: usize,
}

#[pymethods]
impl LoPhatOptions {
    #[new]
    #[pyo3(signature = (maintain_v=false, num_threads=0, column_height=None, min_chunk_len=1))]
    fn new(
        maintain_v: bool,
        num_threads: usize,
        column_height: Option<usize>,
        min_chunk_len: usize,
    ) -> Self {
        LoPhatOptions {
            maintain_v,
            num_threads,
            column_height,
            min_chunk_len,
        }
    }
}
