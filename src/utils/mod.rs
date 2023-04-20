//! Utility functions and structs, including persistence diagrams and matrix anti-transposition.

mod anti_transpose;
mod diagram;
#[cfg(feature = "serde")]
mod file_format;

pub use anti_transpose::anti_transpose;
pub use diagram::PersistenceDiagram;

#[cfg(feature = "serde")]
pub use file_format::{clone_to_file_format, clone_to_veccolumn, serialize_algo, RVDFileFormat};

use crate::columns::{Column, ColumnMode};

/// Helper function to set mode of both columns
pub(crate) fn set_mode_of_pair<C: Column>(column_pair: &mut (C, Option<C>), mode: ColumnMode) {
    column_pair.0.set_mode(mode);
    if let Some(c) = column_pair.1.as_mut() {
        c.set_mode(mode);
    }
}
