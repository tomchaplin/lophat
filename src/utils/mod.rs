//! Utility functions and structs, including persistence diagrams and matrix anti-transposition.

mod anti_transpose;
mod diagram;

pub use anti_transpose::anti_transpose;
pub use diagram::PersistenceDiagram;
