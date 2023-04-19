//! Utility functions and structs, including persistence diagrams and matrix anti-transposition.

mod anti_transpose;
mod diagram;

pub use anti_transpose::anti_transpose;
pub use diagram::PersistenceDiagram;

use crate::columns::{Column, ColumnMode};

/// Helper function to set mode of both columns
pub(crate) fn set_mode_of_pair<C: Column>(column_pair: &mut (C, Option<C>), mode: ColumnMode) {
    column_pair.0.set_mode(mode);
    if let Some(c) = column_pair.1.as_mut() {
        c.set_mode(mode);
    }
}
