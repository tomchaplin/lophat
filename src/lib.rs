//! # LoPHAT
//! LoPHAT implements algorithms for computing persistent homology, in parallel, in a lockfree fashion.
//! The primary interface with this crate is the [`rv_decompose_lock_free`] function.
//! We also implement the traditional, serial algorithm in [`rv_decompose`].
//!
//! Both functions accept an iterator over columns.
//! These can have any representation in memory, so long as they impelement the [`Column`] trait.
//! For many applications, [`VecColumn`] is a good choice for your columns.
//!
//! Both functions return a [`RVDecomposition`], from which you can read off the persistence diagram.

mod column;
mod decomposition;
mod lock_free;

pub use column::{Column, VecColumn};
pub use decomposition::{rv_decompose, PersistenceDiagram, RVDecomposition};
pub use lock_free::rv_decompose_lock_free;
