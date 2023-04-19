//! Options for all algorithms.
//! Soon to be deprecated in favour of an option struct per algorithm.

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// A simple struct for specifying options for R=DV decompositions.
/// Soon to be deprecated in favour of an option struct per algorithm.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Copy, Clone)]
pub struct LoPhatOptions {
    /// If true, returns full R=DV decomposition, otherwise the resulign decomposiiton will always return `None` from [`get_v_col`](crate::algorithms::RVDecomposition::get_v_col).
    pub maintain_v: bool,
    ///  Number of threads to use in thread pool; ignored by serial algorithms.
    ///   see [`num_threads`](rayon::ThreadPoolBuilder::num_threads) for more details.
    ///   Only relevant for lockfree algorithm.
    pub num_threads: usize,
    ///  An optional hint to the height of the columns.
    ///   If `None`, assumed to be `matrix.collect().len()`.
    ///   All indices must lie in the range `0..column_height`.
    ///   Only relevant for lockfree algorithm.
    pub column_height: Option<usize>,
    ///  When splitting work, don't reduce chunks to smaller than this size.
    ///   Only relevant for lockfree algorithm.
    pub min_chunk_len: usize,
    ///  Whether to employ the clearing optimisation.
    ///   Note, if input matrix is not square then can't use this optimisation since it assumes D*D = 0.
    ///   Only relevant for lockfree algorithm.
    pub clearing: bool,
}

#[cfg(feature = "python")]
#[pymethods]
impl LoPhatOptions {
    #[new]
    #[pyo3(signature = (maintain_v=false, num_threads=0, column_height=None, min_chunk_len=1, clearing=true))]
    fn new(
        maintain_v: bool,
        num_threads: usize,
        column_height: Option<usize>,
        min_chunk_len: usize,
        clearing: bool,
    ) -> Self {
        LoPhatOptions {
            maintain_v,
            num_threads,
            column_height,
            min_chunk_len,
            clearing,
        }
    }
}

impl Default for LoPhatOptions {
    fn default() -> Self {
        Self {
            maintain_v: false,
            num_threads: 0,
            column_height: None,
            min_chunk_len: 1,
            clearing: true,
        }
    }
}
