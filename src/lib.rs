//! # LoPHAT
//! LoPHAT implements mutliple algorithms for computing persistent homology; in particular, the parallel, lockfree algorithm of Morozov and Nigmetov.
//! The algorithms implemented are:
//!
//! * [`SerialAlgorithm`](algorithms::SerialAlgorithm) - the standard left-to-right column additional algorithm of [Edelsbrunner et al.](https://doi.org/10.1109/SFCS.2000.892133).
//! * [`LockFreeAlgorithm`](algorithms::LockFreeAlgorithm) - the algorithm introduced by [Morozov and Nigmetov](https://doi.org/10.1145/3350755.3400244).
//! * [`LockingAlgorithm`](algorithms::LockingAlgorithm) - a locking variant of the above, in which each column is stored behind a [`RwLock`](std::sync::RwLock).
//!
//! The primary interface to each of these algorithms is the the [`decompose`](algorithms::RVDecomposition::decompose) method of the [`RVDecomposition`](algorithms::RVDecomposition) trait.
//! This method accepts your boundary matrix (as well as an options struct) and decomposes it via the corresponding algorithm.
//! The output is a struct which implements [`RVDecomposition`](algorithms::RVDecomposition) -- a trait which contains methods for querying the resulting R=DV decomposition.
//! In particular, you can compute the persistence pairings via the [`diagram`](algorithms::RVDecomposition::diagram) method.
//!
//! Each algorithm is generic over the underlying column representation.
//! A number of representations are available in the [`columns`] module.
//! For many applications, [`VecColumn`](columns::VecColumn) is a good choice.
//!
//! # Example
//!
//! ```
//! // Import the algorithm you want to use as well as the decomposition trait
//! use lophat::algorithms::{LockFreeAlgorithm, RVDecomposition};
//! // Import the column representation we want to use
//! use lophat::columns::VecColumn;
//! use lophat::utils::PersistenceDiagram;
//! use hashbrown::HashSet;
//!
//! fn build_sphere_triangulation() -> impl Iterator<Item = VecColumn> {
//!     vec![
//!         (0, vec![]),
//!         (0, vec![]),
//!         (0, vec![]),
//!         (0, vec![]),
//!         (1, vec![0, 1]),
//!         (1, vec![0, 2]),
//!         (1, vec![1, 2]),
//!         (1, vec![0, 3]),
//!         (1, vec![1, 3]),
//!         (1, vec![2, 3]),
//!         (2, vec![4, 7, 8]),
//!         (2, vec![5, 7, 9]),
//!         (2, vec![6, 8, 9]),
//!         (2, vec![4, 5, 6]),
//!     ]
//!     .into_iter()
//!     .map(|col| col.into())
//! }
//!
//! // Build a simplicial representation of the 2-sphere
//! let matrix = build_sphere_triangulation();
//! // Decompose with the lockfree algorithm, using the default options
//! let decomposition = LockFreeAlgorithm::decompose(matrix, None);
//! // Compute the persistence diagram
//! let computed_diagram = decomposition.diagram();
//! // Ensure we get the correct pairings
//! let correct_diagram = PersistenceDiagram {
//!     unpaired: HashSet::from_iter(vec![0, 13]),
//!     paired: HashSet::from_iter(vec![(1, 4), (2, 5), (3, 7), (6, 12), (8, 10), (9, 11)]),
//! };
//! assert_eq!(computed_diagram, correct_diagram)
//! ```

// TODO:
// 1. Document everyting
// 2. Use set_mode in columns
// 3. Change options struct for each algorithm
// 4. Decide on Python bindings

/// Structs implementing various algorithms for computing persistent homology.
pub mod algorithms;
/// Structs representing columns of a Z_2 matrix, complying to a common interface.
pub mod columns;
/// To be deprecated.
pub mod options;
/// Utility functions and structs, including persistence diagrams and matrix anti-transposition.
pub mod utils;

#[cfg(feature = "python")]
mod bindings;
