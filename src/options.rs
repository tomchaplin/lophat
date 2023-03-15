/// A simple struct for specifying options for R=DV decompositions
///
/// * `maintain_v` - if true, returns full R=DV decomposition,
///   otherwise returns [`RVDecomposition`](crate::RVDecomposition) with field `v` set to `None`.
/// * `n_threads` - number of threads to use in thread pool; ignored by serial algorithms.
///   see [`num_threads`](rayon::ThreadPoolBuilder::num_threads) for more details.
#[derive(Default, Clone, Copy)]
pub struct LoPhatOptions {
    pub maintain_v: bool,
    pub num_threads: usize,
}
