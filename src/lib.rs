//! # LoPHAT
//! LoPHAT implements algorithms for computing persistent homology, in parallel, in a lockfree fashion.
//! The primary interface with this crate is the [`rv_decompose`] function.
//! This function accepts an iterator over columns.
//! These can have any representation in memory, so long as they impelement the [`Column`] trait.
//! For many applications, [`VecColumn`] is a good choice for your columns.
//! The function returns a [`RVDecomposition`], from which you can read off the persistence diagram.
//! Additional options can be set by providing a [`LoPhatOptions`] struct.
//!
//! If you would like to force the use of the serial or parallel algorithm,
//! use [`rv_decompose_serial`] or [`rv_decompose_lock_free`] respectively.

pub mod algorithms;
pub mod columns;
pub mod options;
pub mod utils;

#[cfg(feature = "python")]
mod bindings;
